use std::path::Path;

pub(crate) fn is_incomplete_browser_download(file_name: &str) -> bool {
    let lower = file_name.to_ascii_lowercase();
    lower.ends_with(".crdownload")
        || lower.ends_with(".part")
        || lower.ends_with(".partial")
        || lower.ends_with(".tmp")
        || lower.ends_with(".download")
}

pub(crate) fn browser_download_file_name_matches(
    actual: &str,
    expected: &str,
) -> bool {
    if is_incomplete_browser_download(actual) {
        return false;
    }

    if browser_download_file_name_matches_without_localized_prefix(
        actual, expected,
    ) {
        return true;
    }

    let Some(actual) = actual.strip_prefix('[') else {
        return false;
    };
    let Some((title, actual)) = actual.split_once(']') else {
        return false;
    };
    if title.trim().is_empty() || actual.is_empty() {
        return false;
    }

    browser_download_file_name_matches_without_localized_prefix(
        actual, expected,
    )
}

fn browser_download_file_name_matches_without_localized_prefix(
    actual: &str,
    expected: &str,
) -> bool {
    if actual.eq_ignore_ascii_case(expected) {
        return true;
    }
    let actual = Path::new(actual);
    let expected = Path::new(expected);
    if actual
        .extension()
        .and_then(|extension| extension.to_str())
        .zip(
            expected
                .extension()
                .and_then(|extension| extension.to_str()),
        )
        .is_none_or(|(actual, expected)| !actual.eq_ignore_ascii_case(expected))
    {
        return false;
    }
    let Some(actual_stem) = actual.file_stem().and_then(|stem| stem.to_str())
    else {
        return false;
    };
    let Some(expected_stem) =
        expected.file_stem().and_then(|stem| stem.to_str())
    else {
        return false;
    };
    let actual_stem = actual_stem.to_lowercase();
    let expected_stem = expected_stem.to_lowercase();
    let Some(suffix) = actual_stem.strip_prefix(&expected_stem) else {
        return false;
    };
    let suffix = suffix.trim();
    suffix
        .strip_prefix('(')
        .and_then(|suffix| suffix.strip_suffix(')'))
        .is_some_and(|number| {
            !number.is_empty()
                && number.bytes().all(|byte| byte.is_ascii_digit())
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn browser_duplicates_match_but_incomplete_files_do_not() {
        for actual in [
            "example-mod.jar",
            "EXAMPLE-MOD.JAR",
            "example-mod (1).jar",
            "example-mod(2).jar",
            "example-mod (123).JAR",
        ] {
            assert!(
                browser_download_file_name_matches(actual, "example-mod.jar"),
                "expected {actual:?} to match"
            );
        }
        for actual in [
            "example-mod-fabric.jar",
            "example-mod (copy).jar",
            "example-mod ().jar",
            "example-mod (1) copy.jar",
            "example-mod (1).zip",
            "other-mod (1).jar",
        ] {
            assert!(
                !browser_download_file_name_matches(actual, "example-mod.jar"),
                "expected {actual:?} not to match"
            );
        }
        for suffix in ["crdownload", "part", "partial", "tmp", "download"] {
            assert!(!browser_download_file_name_matches(
                &format!("example-mod.jar.{suffix}"),
                "example-mod.jar"
            ));
            assert!(!browser_download_file_name_matches(
                &format!("example-mod.jar.{}", suffix.to_ascii_uppercase()),
                "example-mod.jar"
            ));
        }
    }

    #[test]
    fn browser_localized_prefix_matches() {
        for actual in [
            "[测试]example-mod.jar",
            "[中文标题]example-mod.jar",
            "[午餐肉乐园]example-mod.jar",
            "[Test]EXAMPLE-MOD.JAR",
            "[测试]example-mod (1).jar",
            "[测试]example-mod(2).jar",
            "[测试]example-mod (123).JAR",
        ] {
            assert!(
                browser_download_file_name_matches(actual, "example-mod.jar"),
                "expected {actual:?} to match"
            );
        }

        for actual in [
            "abc-example-mod.jar",
            "foo-example-mod.jar",
            "abcEXAMPLE-MOD.jar",
            "[]example-mod.jar",
            "[ ]example-mod.jar",
            "[testexample-mod.jar",
            "test]example-mod.jar",
            "prefix[test]example-mod.jar",
            "[test]abc-example-mod.jar",
            "[test]foo-example-mod.jar",
            "[test]other-mod.jar",
            "[test]example-mod (copy).jar",
            "[test]example-mod ().jar",
            "[test]example-mod (1) copy.jar",
            "[test]example-mod (1).zip",
            "[a][b]example-mod.jar",
        ] {
            assert!(
                !browser_download_file_name_matches(actual, "example-mod.jar"),
                "expected {actual:?} not to match"
            );
        }

        for suffix in ["crdownload", "part", "partial", "tmp", "download"] {
            assert!(!browser_download_file_name_matches(
                &format!("[测试]example-mod.jar.{suffix}"),
                "example-mod.jar"
            ));
            assert!(!browser_download_file_name_matches(
                &format!(
                    "[测试]example-mod.jar.{}",
                    suffix.to_ascii_uppercase()
                ),
                "example-mod.jar"
            ));
        }
    }
}
