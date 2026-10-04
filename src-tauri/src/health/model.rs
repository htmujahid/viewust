use serde::Serialize;

#[derive(Serialize, Debug, PartialEq, Clone, Copy, PartialOrd)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Ok,
    Warn,
    Danger,
}

#[derive(Serialize, Debug)]
pub struct Check {
    pub(crate) severity: Severity,
    pub(crate) title: String,
    pub(crate) detail: String,
    /// Where to look closer, when there is somewhere
    pub(crate) link: Option<&'static str>,
}

#[derive(Serialize)]
pub struct HealthReport {
    pub(crate) problems: usize,
    pub(crate) warnings: usize,
    pub(crate) fine: usize,
    pub(crate) checks: Vec<Check>,
}
