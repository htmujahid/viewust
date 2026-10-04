use std::collections::HashMap;

/// `/etc/os-release` is shell-style `KEY="value"` lines.
pub(crate) fn parse_os_release(text: &str) -> HashMap<String, String> {
    text.lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.starts_with('#') {
                return None;
            }
            let (key, value) = line.split_once('=')?;
            let value = value.trim();
            let value = value
                .strip_prefix('"')
                .and_then(|v| v.strip_suffix('"'))
                .or_else(|| value.strip_prefix('\'').and_then(|v| v.strip_suffix('\'')))
                .unwrap_or(value);
            Some((key.trim().to_owned(), value.replace("\\\"", "\"")))
        })
        .collect()
}

pub(crate) fn os_release() -> HashMap<String, String> {
    ["/etc/os-release", "/usr/lib/os-release"]
        .iter()
        .find_map(|p| std::fs::read_to_string(p).ok())
        .map(|t| parse_os_release(&t))
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    const RELEASE: &str = "\
# a comment
PRETTY_NAME=\"Ubuntu 26.04 LTS\"
NAME=\"Ubuntu\"
VERSION_ID=\"26.04\"
ID=ubuntu
ID_LIKE=debian
HOME_URL='https://www.ubuntu.com/'
EMPTY=
";

    #[test]
    fn quoted_and_bare_values_are_read() {
        let r = parse_os_release(RELEASE);
        assert_eq!(r["PRETTY_NAME"], "Ubuntu 26.04 LTS");
        assert_eq!(r["ID"], "ubuntu");
        assert_eq!(r["HOME_URL"], "https://www.ubuntu.com/");
        assert_eq!(r["EMPTY"], "");
        assert!(!r.contains_key("# a comment"));
    }
}
