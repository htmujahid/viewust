use super::model::EnvVar;

const SECRET_WORDS: &[&str] = &[
    "TOKEN",
    "SECRET",
    "PASSWORD",
    "PASSWD",
    "KEY",
    "CREDENTIAL",
    "COOKIE",
    "PRIVATE",
    "AUTH",
];

/// Names that suggest a secret. Their values never leave the backend.
pub(crate) fn is_sensitive(key: &str) -> bool {
    let key = key.to_uppercase();
    SECRET_WORDS.iter().any(|w| key.contains(w))
}

pub(crate) fn from_pairs(pairs: impl IntoIterator<Item = (String, String)>) -> Vec<EnvVar> {
    let mut vars: Vec<EnvVar> = pairs
        .into_iter()
        .map(|(key, value)| {
            let hidden = is_sensitive(&key);
            EnvVar {
                value: if hidden { String::new() } else { value },
                key,
                hidden,
            }
        })
        .collect();
    vars.sort_by(|a, b| a.key.cmp(&b.key));
    vars
}

pub(crate) fn list() -> Vec<EnvVar> {
    from_pairs(std::env::vars_os().map(|(k, v)| {
        (
            k.to_string_lossy().into_owned(),
            v.to_string_lossy().into_owned(),
        )
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_that_look_secret_are_recognised() {
        for k in [
            "GITHUB_TOKEN",
            "aws_secret_access_key",
            "DB_PASSWORD",
            "SSH_AUTH_SOCK",
            "API_KEY",
        ] {
            assert!(is_sensitive(k), "{k}");
        }
        for k in ["PATH", "HOME", "LANG", "XDG_SESSION_TYPE", "DISPLAY"] {
            assert!(!is_sensitive(k), "{k}");
        }
    }

    #[test]
    fn a_secrets_value_is_dropped_and_the_rest_sorted() {
        let v = from_pairs([
            ("PATH".to_owned(), "/bin".to_owned()),
            ("API_TOKEN".to_owned(), "abc123".to_owned()),
            ("HOME".to_owned(), "/home/me".to_owned()),
        ]);
        let keys: Vec<&str> = v.iter().map(|e| e.key.as_str()).collect();
        assert_eq!(keys, ["API_TOKEN", "HOME", "PATH"]);
        assert_eq!((v[0].hidden, v[0].value.as_str()), (true, ""));
        assert_eq!(v[2].value, "/bin");
    }
}
