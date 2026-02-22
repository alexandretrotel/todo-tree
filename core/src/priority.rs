use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    Low,
    Medium,
    High,
    Critical,
}

impl Priority {
    pub fn from_tag(tag: &str) -> Self {
        match tag.to_uppercase().as_str() {
            "BUG" | "FIXME" | "ERROR" => Priority::Critical,
            "HACK" | "WARN" | "WARNING" | "FIX" => Priority::High,
            "TODO" | "WIP" | "MAYBE" => Priority::Medium,
            "NOTE" | "XXX" | "INFO" | "DOCS" | "PERF" | "TEST" | "IDEA" => Priority::Low,
            _ => Priority::Medium,
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Priority::Critical => "Critical",
            Priority::High => "High",
            Priority::Medium => "Medium",
            Priority::Low => "Low",
        }
    }
}

impl std::fmt::Display for Priority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}
