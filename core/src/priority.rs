use serde::{Deserialize, Serialize};

/// Priority levels for different tag types
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    Low,
    Medium,
    High,
    Critical,
}

impl Priority {
    /// Infer priority from tag name
    pub fn from_tag(tag: &str) -> Self {
        match tag.to_uppercase().as_str() {
            "BUG" | "FIXME" | "ERROR" => Priority::Critical,
            "HACK" | "WARN" | "WARNING" | "FIX" => Priority::High,
            "TODO" | "WIP" | "MAYBE" => Priority::Medium,
            "NOTE" | "XXX" | "INFO" | "DOCS" | "PERF" | "TEST" | "IDEA" => Priority::Low,
            _ => Priority::Medium,
        }
    }

    /// Get emoji representation for the priority
    pub fn emoji(&self) -> &'static str {
        match self {
            Priority::Critical => "🔴",
            Priority::High => "🟡",
            Priority::Medium => "🔵",
            Priority::Low => "🟢",
        }
    }

    /// Get display name for the priority
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_priority_from_tag() {
        // Critical
        assert_eq!(Priority::from_tag("BUG"), Priority::Critical);
        assert_eq!(Priority::from_tag("FIXME"), Priority::Critical);
        assert_eq!(Priority::from_tag("ERROR"), Priority::Critical);
        // High
        assert_eq!(Priority::from_tag("HACK"), Priority::High);
        assert_eq!(Priority::from_tag("WARN"), Priority::High);
        assert_eq!(Priority::from_tag("WARNING"), Priority::High);
        assert_eq!(Priority::from_tag("FIX"), Priority::High);
        // Medium
        assert_eq!(Priority::from_tag("TODO"), Priority::Medium);
        assert_eq!(Priority::from_tag("WIP"), Priority::Medium);
        assert_eq!(Priority::from_tag("MAYBE"), Priority::Medium);
        // Low
        assert_eq!(Priority::from_tag("NOTE"), Priority::Low);
        assert_eq!(Priority::from_tag("XXX"), Priority::Low);
        assert_eq!(Priority::from_tag("INFO"), Priority::Low);
        assert_eq!(Priority::from_tag("DOCS"), Priority::Low);
        assert_eq!(Priority::from_tag("PERF"), Priority::Low);
        assert_eq!(Priority::from_tag("TEST"), Priority::Low);
        assert_eq!(Priority::from_tag("IDEA"), Priority::Low);
    }

    #[test]
    fn test_priority_from_tag_case_variations() {
        assert_eq!(Priority::from_tag("bug"), Priority::Critical);
        assert_eq!(Priority::from_tag("Bug"), Priority::Critical);
        assert_eq!(Priority::from_tag("hack"), Priority::High);
        assert_eq!(Priority::from_tag("Hack"), Priority::High);
        assert_eq!(Priority::from_tag("warn"), Priority::High);
        assert_eq!(Priority::from_tag("wip"), Priority::Medium);
        assert_eq!(Priority::from_tag("info"), Priority::Low);
    }

    #[test]
    fn test_priority_from_unknown_tag() {
        assert_eq!(Priority::from_tag("UNKNOWN"), Priority::Medium);
        assert_eq!(Priority::from_tag("CUSTOM"), Priority::Medium);
        assert_eq!(Priority::from_tag("RANDOM"), Priority::Medium);
    }

    #[test]
    fn test_priority_ordering() {
        assert!(Priority::Critical > Priority::High);
        assert!(Priority::High > Priority::Medium);
        assert!(Priority::Medium > Priority::Low);
    }

    #[test]
    fn test_priority_emoji() {
        assert_eq!(Priority::Critical.emoji(), "🔴");
        assert_eq!(Priority::High.emoji(), "🟡");
        assert_eq!(Priority::Medium.emoji(), "🔵");
        assert_eq!(Priority::Low.emoji(), "🟢");
    }

    #[test]
    fn test_priority_display_name() {
        assert_eq!(Priority::Critical.display_name(), "Critical");
        assert_eq!(Priority::High.display_name(), "High");
        assert_eq!(Priority::Medium.display_name(), "Medium");
        assert_eq!(Priority::Low.display_name(), "Low");
    }

    #[test]
    fn test_priority_display() {
        assert_eq!(format!("{}", Priority::Critical), "Critical");
        assert_eq!(format!("{}", Priority::High), "High");
        assert_eq!(format!("{}", Priority::Medium), "Medium");
        assert_eq!(format!("{}", Priority::Low), "Low");
    }

    #[test]
    fn test_priority_serialization() {
        let priority = Priority::Critical;
        let json = serde_json::to_string(&priority).unwrap();
        let deserialized: Priority = serde_json::from_str(&json).unwrap();
        assert_eq!(priority, deserialized);
    }

    #[test]
    fn test_priority_equality() {
        assert_eq!(Priority::Critical, Priority::Critical);
        assert_ne!(Priority::Critical, Priority::High);
    }
}
