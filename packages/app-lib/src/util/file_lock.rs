//! Detect processes that hold a file lock on a given path.
//!
//! Uses platform-specific lock inspection:
//! - Windows: Restart Manager API, with `handle.exe` as a fallback
//! - Linux: `fuser -v` with fallback to `lsof -t`
//! - macOS: `lsof -F pcn`
//!
//! All operations gracefully return an empty vec on any error.
#![allow(dead_code)]

use std::path::Path;

use serde::Serialize;
use tracing::warn;

#[cfg(windows)]
use std::{collections::HashSet, os::windows::ffi::OsStrExt};
#[cfg(windows)]
use windows::{
    Win32::{
        Foundation::{ERROR_MORE_DATA, ERROR_SUCCESS},
        System::RestartManager::{
            CCH_RM_SESSION_KEY, RM_PROCESS_INFO, RmEndSession, RmGetList,
            RmRegisterResources, RmStartSession,
        },
    },
    core::{PCWSTR, PWSTR},
};

/// Information about a process that has a file handle open.
#[derive(Debug, Clone, Serialize)]
pub struct LockingProcess {
    pub pid: u32,
    pub name: String,
    pub path: String,
    /// ISO 8601 timestamp of process start time, if available.
    pub start_time: Option<String>,
}

/// Detect processes currently holding the file at `file_path` open.
///
/// Returns an empty vec when no locking processes are found, the platform
/// cannot determine this, required tools are missing, or any error occurs.
pub fn get_locking_processes(file_path: &Path) -> Vec<LockingProcess> {
    let path_str = file_path.to_string_lossy().to_string();

    #[cfg(windows)]
    {
        let procs = get_locking_processes_windows(file_path, &path_str);
        if !procs.is_empty() {
            warn!(
                "Windows file lock detected on {}: {} process(es)",
                path_str,
                procs.len()
            );
        }
        procs
    }

    #[cfg(target_os = "linux")]
    {
        let procs = get_locking_processes_linux(file_path, &path_str);
        if !procs.is_empty() {
            warn!(
                "Linux file lock detected on {}: {} process(es)",
                path_str,
                procs.len()
            );
        }
        procs
    }

    #[cfg(target_os = "macos")]
    {
        let procs = get_locking_processes_macos(file_path, &path_str);
        if !procs.is_empty() {
            warn!(
                "macOS file lock detected on {}: {} process(es)",
                path_str,
                procs.len()
            );
        }
        procs
    }

    #[cfg(not(any(windows, target_os = "linux", target_os = "macos")))]
    {
        let _ = path_str;
        Vec::new()
    }
}

// ─── Windows: Restart Manager / handle.exe fallback ───────────────────────

#[cfg(windows)]
fn get_locking_processes_windows(
    file_path: &Path,
    path_str: &str,
) -> Vec<LockingProcess> {
    if let Some(processes) =
        get_locking_processes_windows_restart_manager(file_path, path_str)
    {
        return processes;
    }

    let output =
        run_subprocess("handle.exe", &["-a", &file_path.to_string_lossy()]);

    match output {
        Some(stdout) => parse_handle_exe_output(&stdout, path_str),
        None => Vec::new(),
    }
}

#[cfg(windows)]
struct RestartManagerSession(u32);

#[cfg(windows)]
impl Drop for RestartManagerSession {
    fn drop(&mut self) {
        unsafe {
            let _ = RmEndSession(self.0);
        }
    }
}

#[cfg(windows)]
fn get_locking_processes_windows_restart_manager(
    file_path: &Path,
    path_str: &str,
) -> Option<Vec<LockingProcess>> {
    let mut session_handle = 0;
    let mut session_key = [0u16; CCH_RM_SESSION_KEY as usize + 1];
    let status = unsafe {
        RmStartSession(
            &raw mut session_handle,
            None,
            PWSTR(session_key.as_mut_ptr()),
        )
    };
    if status != ERROR_SUCCESS {
        warn!(
            "Restart Manager could not start a session for {}: error {}",
            path_str, status.0
        );
        return None;
    }
    let session = RestartManagerSession(session_handle);

    let wide_path = file_path
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect::<Vec<_>>();
    let filenames = [PCWSTR(wide_path.as_ptr())];
    let status =
        unsafe { RmRegisterResources(session.0, Some(&filenames), None, None) };
    if status != ERROR_SUCCESS {
        warn!(
            "Restart Manager could not register {}: error {}",
            path_str, status.0
        );
        return None;
    }

    let mut needed = 0;
    let mut count = 0;
    let mut reboot_reasons = 0;
    let status = unsafe {
        RmGetList(
            session.0,
            &raw mut needed,
            &raw mut count,
            None,
            &raw mut reboot_reasons,
        )
    };
    if status == ERROR_SUCCESS && needed == 0 {
        return Some(Vec::new());
    }
    if status != ERROR_MORE_DATA {
        warn!(
            "Restart Manager could not query {}: error {}",
            path_str, status.0
        );
        return None;
    }

    for _ in 0..3 {
        let mut process_info =
            vec![RM_PROCESS_INFO::default(); needed as usize];
        count = needed;
        let status = unsafe {
            RmGetList(
                session.0,
                &raw mut needed,
                &raw mut count,
                Some(process_info.as_mut_ptr()),
                &raw mut reboot_reasons,
            )
        };
        if status == ERROR_MORE_DATA {
            continue;
        }
        if status != ERROR_SUCCESS {
            warn!(
                "Restart Manager could not read {}: error {}",
                path_str, status.0
            );
            return None;
        }

        process_info.truncate(count as usize);
        let mut seen_pids = HashSet::new();
        let processes = process_info
            .into_iter()
            .filter_map(|process| {
                let pid = process.Process.dwProcessId;
                seen_pids.insert(pid).then(|| {
                    let app_name = utf16_array_to_string(&process.strAppName);
                    let service_name =
                        utf16_array_to_string(&process.strServiceShortName);
                    let name = if !app_name.is_empty() {
                        app_name
                    } else if !service_name.is_empty() {
                        service_name
                    } else {
                        format!("PID {pid}")
                    };
                    LockingProcess {
                        pid,
                        name,
                        path: path_str.to_string(),
                        start_time: None,
                    }
                })
            })
            .collect();
        return Some(processes);
    }

    warn!(
        "Restart Manager process list kept changing while querying {}",
        path_str
    );
    None
}

#[cfg(windows)]
fn utf16_array_to_string(value: &[u16]) -> String {
    let length = value
        .iter()
        .position(|character| *character == 0)
        .unwrap_or(value.len());
    String::from_utf16_lossy(&value[..length])
}

/// Parse handle.exe output to extract PID + process name.
///
/// handle.exe output lines look like:
/// ```text
/// chrome.exe        pid: 1234  type: File    <path>
/// ```
#[cfg(windows)]
fn parse_handle_exe_output(
    output: &str,
    path_str: &str,
) -> Vec<LockingProcess> {
    let mut result = Vec::new();
    for line in output.lines() {
        let trimmed = line.trim();
        if let Some(pid_start) = trimmed.find("pid:") {
            let after_pid = &trimmed[pid_start + 4..];
            let pid_str = after_pid.split_whitespace().next().unwrap_or("");
            if let Ok(pid) = pid_str.parse::<u32>() {
                let name = trimmed[..pid_start].trim().to_string();
                result.push(LockingProcess {
                    pid,
                    name,
                    path: path_str.to_string(),
                    start_time: None,
                });
            }
        }
    }
    result
}

// ─── Linux: fuser / lsof ───────────────────────────────────────────────────

#[cfg(target_os = "linux")]
fn get_locking_processes_linux(
    file_path: &Path,
    path_str: &str,
) -> Vec<LockingProcess> {
    let output = run_subprocess("fuser", &["-v", &file_path.to_string_lossy()]);

    let pids: Vec<u32> = match output {
        Some(stderr) => parse_fuser_output(&stderr),
        None => {
            let out =
                run_subprocess("lsof", &["-t", &file_path.to_string_lossy()]);
            match out {
                Some(stdout) => stdout
                    .lines()
                    .filter_map(|l| l.trim().parse::<u32>().ok())
                    .collect(),
                None => return Vec::new(),
            }
        }
    };

    pids.into_iter()
        .filter_map(|pid| {
            let name = read_proc_name(pid)?;
            let start_time = read_proc_start_time(pid);
            Some(LockingProcess {
                pid,
                name,
                path: path_str.to_string(),
                start_time,
            })
        })
        .collect()
}

#[cfg(target_os = "linux")]
fn parse_fuser_output(stderr: &str) -> Vec<u32> {
    let mut pids = Vec::new();
    for line in stderr.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with("USER") {
            // Skip the "USER PID ACCESS COMMAND" header line.
            continue;
        }
        let tokens: Vec<&str> = trimmed.split_whitespace().collect();
        // `fuser -v` data lines look like "path: USER PID ACCESS COMMAND",
        // so the PID is the third token; tolerate layouts without the
        // path prefix by falling back to the second token.
        let pid_candidate = if tokens.first().is_some_and(|t| t.ends_with(':'))
        {
            tokens.get(2)
        } else {
            tokens.get(1)
        };
        if let Some(pid_str) = pid_candidate
            && let Ok(pid) = pid_str.parse::<u32>()
            && pid > 0
        {
            pids.push(pid);
        }
    }
    pids
}

#[cfg(target_os = "linux")]
fn read_proc_name(pid: u32) -> Option<String> {
    let cmdline_path = format!("/proc/{pid}/cmdline");
    let content = std::fs::read_to_string(&cmdline_path).ok()?;
    let first = content.split('\0').next()?;
    if first.is_empty() {
        return None;
    }
    Some(
        std::path::Path::new(first)
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| first.to_string()),
    )
}

#[cfg(target_os = "linux")]
fn read_proc_start_time(pid: u32) -> Option<String> {
    let stat_path = format!("/proc/{pid}/stat");
    let content = std::fs::read_to_string(&stat_path).ok()?;
    let paren_end = content.rfind(')')?;
    let after_paren = &content[paren_end + 1..];
    let fields: Vec<&str> = after_paren.split_whitespace().collect();
    // Field 22 (1-indexed) = starttime = index 19 (0-indexed).
    let starttime_str = fields.get(19)?;
    let clock_ticks: u64 = starttime_str.parse().ok()?;

    let clk_tck = 100u64;
    let seconds_since_boot = clock_ticks / clk_tck;

    let stat_content = std::fs::read_to_string("/proc/stat").ok()?;
    let btime_line = stat_content.lines().find(|l| l.starts_with("btime "))?;
    let boot_time_secs: u64 =
        btime_line.strip_prefix("btime ")?.trim().parse().ok()?;

    let start_time_secs = boot_time_secs + seconds_since_boot;
    let dur =
        std::time::UNIX_EPOCH + std::time::Duration::from_secs(start_time_secs);
    let datetime = chrono::DateTime::<chrono::Utc>::from(dur);
    Some(datetime.to_rfc3339())
}

// ─── macOS: lsof ───────────────────────────────────────────────────────────

#[cfg(target_os = "macos")]
fn get_locking_processes_macos(
    file_path: &Path,
    path_str: &str,
) -> Vec<LockingProcess> {
    let output =
        run_subprocess("lsof", &["-F", "pcn", &file_path.to_string_lossy()]);

    match output {
        Some(stdout) => parse_lsof_output(&stdout, path_str),
        None => Vec::new(),
    }
}

#[cfg(target_os = "macos")]
fn parse_lsof_output(output: &str, path_str: &str) -> Vec<LockingProcess> {
    let mut result = Vec::new();
    let mut current_pid: Option<u32> = None;
    let mut current_name: Option<String> = None;

    for line in output.lines() {
        if line.is_empty() {
            continue;
        }
        match line.chars().next() {
            Some('p') => {
                if let (Some(pid), Some(name)) =
                    (current_pid.take(), current_name.take())
                {
                    result.push(LockingProcess {
                        pid,
                        name,
                        path: path_str.to_string(),
                        start_time: None,
                    });
                }
                current_pid = line[1..].parse::<u32>().ok();
            }
            Some('c') => {
                current_name = Some(line[1..].to_string());
            }
            Some('n') => {}
            _ => {}
        }
    }

    if let (Some(pid), Some(name)) = (current_pid, current_name) {
        result.push(LockingProcess {
            pid,
            name,
            path: path_str.to_string(),
            start_time: None,
        });
    }

    result
}

// ─── Shared subprocess helper ──────────────────────────────────────────────

/// Run a subprocess with the given arguments, capturing combined output.
/// Returns `None` if the command is not found or fails.
fn run_subprocess(program: &str, args: &[&str]) -> Option<String> {
    let result = std::process::Command::new(program).args(args).output();
    match result {
        Ok(output) if output.status.success() => {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            let combined = if stdout.is_empty() { stderr } else { stdout };
            if combined.is_empty() {
                None
            } else {
                Some(combined)
            }
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_unlocked_file() {
        let dir = tempdir().expect("temp dir");
        let file_path = dir.path().join("test.txt");
        std::fs::write(&file_path, "hello").expect("write file");
        let processes = get_locking_processes(&file_path);
        assert!(
            processes
                .iter()
                .all(|process| process.pid != std::process::id())
        );
    }

    #[cfg(windows)]
    #[test]
    fn windows_restart_manager_decodes_process_name() {
        let value = [b'A' as u16, b'x' as u16, b'o' as u16, 0, b'X' as u16];
        assert_eq!(utf16_array_to_string(&value), "Axo");
    }

    #[test]
    fn test_nonexistent_file() {
        let path = Path::new("/tmp/this_file_does_not_exist_xyz.test");
        let processes = get_locking_processes(path);
        assert!(processes.is_empty());
    }

    #[test]
    fn test_directory_path() {
        let dir = tempdir().expect("temp dir");
        let processes = get_locking_processes(dir.path());
        let _ = processes;
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn fuser_output_parses_pids_after_username() {
        let output = "\
USER PID ACCESS COMMAND
/tmp/foo.jar:      user 1234 f....  minecraft
/tmp/foo.jar:      root 4321 F....  java
";
        assert_eq!(parse_fuser_output(output), vec![1234, 4321]);
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn fuser_output_without_locks_is_empty() {
        assert!(parse_fuser_output("").is_empty());
        assert!(parse_fuser_output("USER PID ACCESS COMMAND").is_empty());
    }
}
