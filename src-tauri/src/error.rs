use serde::{Serialize, Serializer};

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("{0} is not installed")]
    MissingTool(&'static str),

    #[error("Permission was declined")]
    PermissionDenied,

    #[error("The read was interrupted: {0}")]
    Interrupted(String),

    #[error("{0}")]
    Other(String),
}

impl Serialize for AppError {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

pub type Result<T> = std::result::Result<T, AppError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn errors_reach_the_frontend_as_plain_messages() {
        let json = serde_json::to_string(&AppError::MissingTool("dmidecode")).unwrap();
        assert_eq!(json, "\"dmidecode is not installed\"");
        assert_eq!(
            serde_json::to_string(&AppError::PermissionDenied).unwrap(),
            "\"Permission was declined\""
        );
    }
}
