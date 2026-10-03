use serde::Serialize;

#[derive(Serialize, Clone)]
pub struct Detail {
    section: String,
    label: String,
    value: String,
}

pub struct Details(Vec<Detail>);

impl Details {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn add(
        &mut self,
        section: impl Into<String>,
        label: impl Into<String>,
        value: impl Into<String>,
    ) {
        let value = value.into();
        if !value.trim().is_empty() {
            self.0.push(Detail {
                section: section.into(),
                label: label.into(),
                value,
            });
        }
    }

    pub fn add_opt(
        &mut self,
        section: impl Into<String>,
        label: &str,
        value: Option<impl Into<String>>,
    ) {
        if let Some(v) = value {
            self.add(section, label, v);
        }
    }

    pub fn finish(self) -> Vec<Detail> {
        self.0
    }
}
