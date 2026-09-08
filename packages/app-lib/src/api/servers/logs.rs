//! Console output buffering and streaming for servers.

use std::path::PathBuf;
use tokio::fs::File;
use tokio::io::{AsyncBufReadExt, AsyncSeekExt, BufReader};

use crate::Result;
use crate::api::servers::lifecycle::is_server_running;
use crate::event::emit::emit_server;
use crate::event::{ExitReason, ServerPayloadType};
use crate::state::{clear_log_buffer, push_log_line};

pub async fn get_log_buffer(server_id: &str) -> Result<Vec<String>> {
    Ok(crate::state::get_log_buffer(server_id))
}

pub async fn clear_log(server_id: &str) -> Result<()> {
    clear_log_buffer(server_id);
    Ok(())
}

pub(super) async fn stream_server_output(
    server_id: String,
    reader: impl tokio::io::AsyncRead + Unpin,
) {
    let mut buf_reader = BufReader::new(reader);
    let mut line = String::new();
    let mut jna_hint_emitted = false;
    loop {
        line.clear();
        match buf_reader.read_line(&mut line).await {
            Ok(0) | Err(_) => break,
            Ok(_) => {
                let trimmed = line.trim_end_matches(['\r', '\n']);
                let cleaned = strip_ansi(trimmed);
                if cleaned.is_empty() {
                    continue;
                }
                // The server also echoes its log4j output (timestamped lines) to
                // stdout/stderr in a console format that duplicates every line
                // already delivered losslessly by `tail_server_log_file` from
                // `logs/latest.log`. Drop those here so the file tail stays the
                // single source of truth; only non-logged process output
                // (bootstrap, patcher progress, JVM warnings) is streamed.
                if is_timestamped_log_line(&cleaned) {
                    continue;
                }
                // The server echoes entered commands to its own log (e.g.
                // "> time set 0"), which the file tailer already streams. Skip
                // those here so they are not duplicated by the stdout pipe.
                if cleaned.starts_with("> ") {
                    continue;
                }
                push_log_line(&server_id, cleaned.clone());
                emit_server(
                    &server_id,
                    ServerPayloadType::Log { line: cleaned },
                )
                .await
                .ok();
                if !jna_hint_emitted && is_jna_macos_assertion(trimmed) {
                    jna_hint_emitted = true;
                    for hint in JNA_CRASH_HINT_LINES {
                        push_log_line(&server_id, hint.to_string());
                        emit_server(
                            &server_id,
                            ServerPayloadType::Log {
                                line: hint.to_string(),
                            },
                        )
                        .await
                        .ok();
                    }
                }
            }
        }
    }
}

/// Streams the server's `logs/latest.log` file into the console buffer. The
/// Minecraft/Fabric log4j console output is frequently not delivered through
/// the process stdout pipe (it goes to the log file instead), so tailing this
/// file is the authoritative, lossless source of the server's own logs. Lines
/// already present in the buffer (e.g. delivered via the stdout/stderr pipes)
/// are skipped to avoid duplicates.
pub(super) async fn tail_server_log_file(server_id: String, dir: PathBuf) {
    let log_path = dir.join("logs").join("latest.log");
    let mut reader = loop {
        if !is_server_running(&server_id) {
            return;
        }
        match File::open(&log_path).await {
            Ok(file) => break BufReader::new(file),
            // The file only appears once the server starts logging; poll until then.
            Err(_) => {
                tokio::time::sleep(std::time::Duration::from_millis(250)).await;
            }
        }
    };

    let mut line = String::new();
    loop {
        if !is_server_running(&server_id) {
            return;
        }
        line.clear();
        match reader.read_line(&mut line).await {
            Ok(0) => {
                // Caught up. Detect log rotation (file replaced/truncated) and
                // otherwise wait for more output to be appended.
                if let Ok(meta) = tokio::fs::metadata(&log_path).await
                    && let Ok(pos) = reader.stream_position().await
                    && meta.len() < pos
                    && let Ok(file) = File::open(&log_path).await
                {
                    reader = BufReader::new(file);
                }
                tokio::time::sleep(std::time::Duration::from_millis(200)).await;
            }
            Ok(_) => {
                let trimmed = line.trim_end_matches(['\r', '\n']);
                let cleaned = strip_ansi(trimmed);
                let already_present =
                    crate::state::get_log_buffer(&server_id).contains(&cleaned);
                if !cleaned.is_empty() && !already_present {
                    push_log_line(&server_id, cleaned.clone());
                    emit_server(
                        &server_id,
                        ServerPayloadType::Log { line: cleaned },
                    )
                    .await
                    .ok();
                }
            }
            Err(_) => break,
        }
    }
}

/// Matches the native abort of the known JNA (< 5.13.0) macOS bug (JNA issue
/// #1452): a failed library load overflows JNA's fixed error buffer and the
/// JVM dies with SIGABRT before any Java-level exception can be reported.
fn is_jna_macos_assertion(line: &str) -> bool {
    line.contains("Assertion failed:")
        && line.contains("snprintf() output has been truncated")
        && line.contains("dispatch.c")
}

/// Detects a server log4j line by its leading `[HH:MM:SS]` timestamp, covering
/// both console (`[HH:MM:SS INFO]:`) and file (`[HH:MM:SS] [Thread/INFO]:`)
/// formats. Used to suppress the process-pipe echo of server logs so they are
/// not duplicated by `tail_server_log_file`.
fn is_timestamped_log_line(line: &str) -> bool {
    let b = line.as_bytes();
    b.first() == Some(&b'[')
        && b.get(1).is_some_and(|c| c.is_ascii_digit())
        && b.get(2).is_some_and(|c| c.is_ascii_digit())
        && b.get(3) == Some(&b':')
        && b.get(4).is_some_and(|c| c.is_ascii_digit())
        && b.get(5).is_some_and(|c| c.is_ascii_digit())
        && b.get(6) == Some(&b':')
        && b.get(7).is_some_and(|c| c.is_ascii_digit())
        && b.get(8).is_some_and(|c| c.is_ascii_digit())
}

/// How many lines at the end of a server's output are inspected when
/// classifying why it exited.
const EXIT_ANALYSIS_TAIL_LINES: usize = 50;

/// Classifies why a server exited on its own by scanning the tail of its
/// console output, newest lines first. Returns `None` when nothing matches:
/// no guess is better than a wrong one, and unmatched exits simply behave as
/// before.
pub(super) fn analyze_exit_reason(lines: &[String]) -> Option<ExitReason> {
    lines
        .iter()
        .rev()
        .take(EXIT_ANALYSIS_TAIL_LINES)
        .find_map(|line| is_eula_refusal(line).then_some(ExitReason::Eula))
}

/// Matches the vanilla server's refusal to boot before the EULA has been
/// accepted; the process then writes `eula.txt` and exits immediately.
fn is_eula_refusal(line: &str) -> bool {
    line.contains("need to agree to the EULA")
}

const JNA_CRASH_HINT_LINES: [&str; 3] = [
    "[WispLauncher] This crash matches a known JNA bug on macOS (java-native-access#1452):",
    "[WispLauncher] mods bundling JNA below 5.13.0 abort when a native library fails to load.",
    "[WispLauncher] Update or remove the affected mod, or ask the modpack author to bump JNA to 5.13.0+.",
];

/// Removes ANSI escape sequences (SGR colors, cursor control, OSC titles) that
/// servers emit when they assume an interactive terminal is attached.
fn strip_ansi(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let mut chars = input.char_indices().peekable();
    while let Some((_, character)) = chars.next() {
        if character != '\u{1b}' {
            output.push(character);
            continue;
        }
        match chars.peek().map(|&(_, c)| c) {
            // CSI sequence: parameter bytes, then a final byte in @..~
            Some('[') => {
                chars.next();
                for (_, c) in chars.by_ref() {
                    if ('\u{40}'..='\u{7e}').contains(&c) {
                        break;
                    }
                }
            }
            // OSC sequence: terminated by BEL or ST (ESC \)
            Some(']') => {
                chars.next();
                let mut saw_escape = false;
                for (_, c) in chars.by_ref() {
                    if c == '\u{7}' || (saw_escape && c == '\\') {
                        break;
                    }
                    saw_escape = c == '\u{1b}';
                }
            }
            // Stray escape byte without a recognized sequence
            _ => {}
        }
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_ansi_escape_sequences_from_server_output() {
        let line = "[16:02:30 INFO]: \u{1b}[38;2;255;170;0m/mspt: \u{1b}[38;2;255;255;255mView server tick times\u{1b}[0m";
        assert_eq!(
            strip_ansi(line),
            "[16:02:30 INFO]: /mspt: View server tick times"
        );

        assert_eq!(strip_ansi("\u{1b}]0;Server console\u{7}ready"), "ready");
        assert_eq!(strip_ansi("\u{1b}]0;Server console\u{1b}\\done"), "done");
        assert_eq!(strip_ansi("plain text stays"), "plain text stays");
        assert_eq!(strip_ansi("h\u{e9}llo \u{1b}[31mred"), "h\u{e9}llo red");
    }

    #[test]
    fn detects_timestamped_log_lines() {
        // Console format (no thread) — duplicated by Paper's stdout echo.
        assert!(is_timestamped_log_line(
            "[11:13:49 INFO]: [bootstrap] Running Java 25"
        ));
        // File format (with thread) — authoritative source from latest.log.
        assert!(is_timestamped_log_line(
            "[11:13:49] [ServerMain/INFO]: [bootstrap] Running"
        ));
        assert!(is_timestamped_log_line(
            "[11:13:49] [Server thread/INFO]: Stopped IO worker!"
        ));
        // Non-logged process output must NOT be suppressed.
        assert!(!is_timestamped_log_line("Downloading mojang_26.2.jar"));
        assert!(!is_timestamped_log_line("Applying patches"));
        assert!(!is_timestamped_log_line(
            "Starting org.bukkit.craftbukkit.Main"
        ));
        assert!(!is_timestamped_log_line(
            "WARNING: A terminally deprecated method in sun.misc.Unsafe"
        ));
        assert!(!is_timestamped_log_line(
            "2026-08-29T03:13:49.279070900Z ServerMain WARN Advanced terminal features",
        ));
    }

    #[test]
    fn detects_jna_macos_assertion() {
        let line = "Assertion failed: (count <= len && \"snprintf() output has been truncated\"), function LOAD_ERROR, file dispatch.c, line 74.";
        assert!(is_jna_macos_assertion(line));
        assert!(!is_jna_macos_assertion(
            "Assertion failed: something else, file other.c, line 1."
        ));
        assert!(!is_jna_macos_assertion("regular log output"));
    }

    #[test]
    fn classifies_eula_refusal_from_output_tail() {
        let eula_line = "[15:26:09] [main/INFO]: You need to agree to the EULA in order to run the server. Go to eula.txt for more info.".to_string();
        let mut lines = vec![
            "[15:26:09] [main/INFO]: Starting minecraft server version 26.2"
                .to_string(),
            eula_line.clone(),
        ];
        assert_eq!(analyze_exit_reason(&lines), Some(ExitReason::Eula));

        // Detected even when buried under later shutdown chatter.
        lines.push(
            "[16:44:23] [Server thread/INFO]: Stopped IO worker!".to_string(),
        );
        assert_eq!(analyze_exit_reason(&lines), Some(ExitReason::Eula));

        // A normal shutdown matches nothing and stays unclassified.
        let normal = vec![
            "[16:44:18] [Server thread/INFO]: Stopping the server".to_string(),
            "[16:44:23] [Server thread/INFO]: Stopped IO worker!".to_string(),
        ];
        assert_eq!(analyze_exit_reason(&normal), None);
        assert_eq!(analyze_exit_reason(&[]), None);

        // Only the tail is inspected; ancient history does not classify a
        // much-later exit.
        let mut old = vec![eula_line];
        old.resize(EXIT_ANALYSIS_TAIL_LINES + 10, "noise".to_string());
        assert_eq!(analyze_exit_reason(&old), None);
    }
}
