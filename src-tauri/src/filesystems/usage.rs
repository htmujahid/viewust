use std::collections::HashMap;

#[derive(Clone, Copy, Default, PartialEq, Debug)]
pub(crate) struct Usage {
    pub(crate) size: Option<u64>,
    pub(crate) used: Option<u64>,
    pub(crate) available: Option<u64>,
    pub(crate) inodes_total: Option<u64>,
    pub(crate) inodes_used: Option<u64>,
}

/// Columns requested from `df`, with the mount point last because it may contain spaces.
pub(crate) const DF_COLUMNS: &str = "--output=size,used,avail,itotal,iused,target";

/// A pseudo filesystem reports zero: that means "no space to speak of", not "full".
fn real(value: Option<u64>) -> Option<u64> {
    value.filter(|v| *v > 0)
}

/// `df` prints "-" for what a filesystem can't tell, which parses to `None`.
pub(crate) fn parse_df(text: &str) -> HashMap<String, Usage> {
    text.lines()
        .skip(1)
        .filter_map(|line| {
            let mut rest = line.trim_start();
            let mut n = [None; 5];
            for slot in &mut n {
                let end = rest.find(char::is_whitespace)?;
                *slot = rest[..end].parse::<u64>().ok();
                rest = rest[end..].trim_start();
            }
            let [size, used, available, inodes_total, inodes_used] = n;
            (!rest.is_empty()).then(|| {
                let sized = real(size).is_some();
                (
                    rest.to_owned(),
                    Usage {
                        size: real(size),
                        used: used.filter(|_| sized),
                        available: available.filter(|_| sized),
                        inodes_total: real(inodes_total),
                        inodes_used: inodes_used.filter(|_| real(inodes_total).is_some()),
                    },
                )
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const DF: &str = "\
1B-blocks        Used       Avail Inodes  IUsed Mounted on
 490000000000 210000000000 255000000000 32000000 900000 /
           0           0           0       0      0 /proc
     1638400        4096     1634304 4000000     60 /run/user/1000
 1000000000000 400000000000 600000000000       -      - /media/me/My Stick
";

    #[test]
    fn a_real_filesystem_reports_space_and_inodes() {
        let u = parse_df(DF)["/"];
        assert_eq!(u.size, Some(490_000_000_000));
        assert_eq!(u.used, Some(210_000_000_000));
        assert_eq!(u.available, Some(255_000_000_000));
        assert_eq!(
            (u.inodes_total, u.inodes_used),
            (Some(32_000_000), Some(900_000))
        );
    }

    #[test]
    fn a_pseudo_filesystem_has_no_numbers() {
        assert_eq!(parse_df(DF)["/proc"], Usage::default());
    }

    #[test]
    fn missing_inode_counts_and_spaces_in_the_path_are_handled() {
        let u = parse_df(DF)["/media/me/My Stick"];
        assert_eq!(u.size, Some(1_000_000_000_000));
        assert_eq!((u.inodes_total, u.inodes_used), (None, None));
    }
}
