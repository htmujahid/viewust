//! DNS servers.

/// "DNS Servers: 1.1.1.1 8.8.8.8" (or "Current DNS Server: …") from `resolvectl status`.
pub fn dns_servers(text: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for line in text.lines().map(str::trim) {
        if let Some(v) = line
            .strip_prefix("DNS Servers:")
            .or_else(|| line.strip_prefix("Current DNS Server:"))
        {
            for s in v.split_whitespace() {
                if !out.iter().any(|x| x == s) {
                    out.push(s.to_owned());
                }
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collects_each_server_once() {
        let text = "Link 3 (wlan0)\n  Current DNS Server: 1.1.1.1\n         DNS Servers: 1.1.1.1 8.8.8.8\n";
        assert_eq!(dns_servers(text), vec!["1.1.1.1", "8.8.8.8"]);
        assert!(dns_servers("nothing relevant").is_empty());
    }
}
