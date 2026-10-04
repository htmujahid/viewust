pub(crate) struct Mount {
    pub(crate) target: String,
    pub(crate) fstype: String,
    pub(crate) source: String,
}

/// The kernel writes spaces, tabs and backslashes in paths as `\040`, `\011`, `\134`.
pub(crate) fn unescape(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        let octal = bytes
            .get(i + 1..i + 4)
            .filter(|d| bytes[i] == b'\\' && d.iter().all(|c| (b'0'..=b'7').contains(c)))
            .and_then(|d| u8::from_str_radix(std::str::from_utf8(d).ok()?, 8).ok());
        match octal {
            Some(byte) => {
                out.push(byte);
                i += 4;
            }
            None => {
                out.push(bytes[i]);
                i += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

pub(crate) fn parse_mountinfo(text: &str) -> Vec<Mount> {
    text.lines()
        .filter_map(|line| {
            let words: Vec<&str> = line.split_whitespace().collect();
            let dash = words.iter().skip(6).position(|w| *w == "-")? + 6;
            Some(Mount {
                target: unescape(words.get(4)?),
                fstype: (*words.get(dash + 1)?).to_owned(),
                source: unescape(words.get(dash + 2)?),
            })
        })
        .collect()
}

/// When mounts are stacked on one path only the last is what anyone sees there; the rest are
/// covered. Keeping one per path also keeps every row's mount point unique.
pub(crate) fn visible(mounts: Vec<Mount>) -> Vec<Mount> {
    let mut seen = std::collections::HashSet::new();
    let mut out: Vec<Mount> = mounts
        .into_iter()
        .rev()
        .filter(|m| seen.insert(m.target.clone()))
        .collect();
    out.reverse();
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const MOUNTINFO: &str = "\
26 1 259:2 / / rw,relatime shared:1 - ext4 /dev/nvme0n1p2 rw,errors=remount-ro
40 26 8:17 / /media/me/My\\040Stick rw,nosuid,nodev,relatime - vfat /dev/sdb1 rw,fmask=0022
41 26 0:38 /sub /mnt/share ro master:7 shared:9 - nfs4 srv:/export ro,vers=4.2
50 26 0:30 / /run rw,nosuid - tmpfs tmpfs rw,size=1638400k
";

    #[test]
    fn escaped_characters_in_paths_are_restored() {
        assert_eq!(unescape("/media/me/My\\040Stick"), "/media/me/My Stick");
        assert_eq!(unescape("a\\134b"), "a\\b");
        assert_eq!(unescape("plain"), "plain");
        assert_eq!(unescape("odd\\9"), "odd\\9");
    }

    #[test]
    fn each_line_gives_its_mount_point_type_and_source() {
        let m = parse_mountinfo(MOUNTINFO);
        assert_eq!(m.len(), 4);
        assert_eq!((m[0].target.as_str(), m[0].fstype.as_str()), ("/", "ext4"));
        assert_eq!(m[0].source, "/dev/nvme0n1p2");
        assert_eq!(m[1].target, "/media/me/My Stick");
    }

    #[test]
    fn optional_fields_before_the_dash_are_skipped() {
        let m = parse_mountinfo(MOUNTINFO);
        assert_eq!(m[2].fstype, "nfs4");
        assert_eq!(m[2].source, "srv:/export");
        assert_eq!(m[3].fstype, "tmpfs");
    }

    #[test]
    fn broken_lines_are_ignored() {
        assert!(parse_mountinfo("garbage\n\n1 2 3").is_empty());
    }

    #[test]
    fn a_stacked_mount_shows_only_the_top_one() {
        let text = "\
20 1 0:21 / /proc/sys/fs/binfmt_misc rw - autofs systemd-1 rw
21 20 0:50 / /proc/sys/fs/binfmt_misc rw - binfmt_misc binfmt_misc rw
22 1 8:1 / /data rw - ext4 /dev/sda1 rw
";
        let m = visible(parse_mountinfo(text));
        let targets: Vec<&str> = m.iter().map(|x| x.target.as_str()).collect();
        assert_eq!(targets, ["/proc/sys/fs/binfmt_misc", "/data"]);
        assert_eq!(m[0].fstype, "binfmt_misc");
    }
}
