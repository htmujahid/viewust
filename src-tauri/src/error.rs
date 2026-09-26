//! The one error type every command returns.
//!
//! It serialises to its message, which is what the frontend shows.

use serde::{Serialize, Serializer};

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    /// A tool this feature depends on isn't installed.
    #[error("{0} is not installed")]
    MissingTool(&'static str),

    /// The user declined an administrator prompt, or it failed.
    #[error("Permission was declined")]
    PermissionDenied,

    /// The background thread doing the work panicked or was cancelled.
    #[error("The read was interrupted: {0}")]
    Interrupted(String),

    /// Anything else, already phrased for a person.
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
