use crate::priority::Priority;

/// Tag definition with metadata for completions and display
#[derive(Debug, Clone, PartialEq)]
pub struct TagDefinition {
    /// Tag name (e.g., "TODO")
    pub name: &'static str,
    /// Description for UI display
    pub description: &'static str,
    /// Priority level
    pub priority: Priority,
}

/// Default tag definitions used by todo-tree
pub const DEFAULT_TAGS: &[TagDefinition] = &[
    // Medium priority - Todo category
    TagDefinition {
        name: "TODO",
        description: "General TODO items",
        priority: Priority::Medium,
    },
    TagDefinition {
        name: "WIP",
        description: "Work in progress",
        priority: Priority::Medium,
    },
    TagDefinition {
        name: "MAYBE",
        description: "Potential future work",
        priority: Priority::Medium,
    },
    // Critical priority - Error category
    TagDefinition {
        name: "FIXME",
        description: "Items that need fixing",
        priority: Priority::Critical,
    },
    TagDefinition {
        name: "BUG",
        description: "Known bugs",
        priority: Priority::Critical,
    },
    TagDefinition {
        name: "ERROR",
        description: "Error handling needed",
        priority: Priority::Critical,
    },
    // High priority - Warn category
    TagDefinition {
        name: "HACK",
        description: "Hacky solutions",
        priority: Priority::High,
    },
    TagDefinition {
        name: "WARN",
        description: "Warnings",
        priority: Priority::High,
    },
    TagDefinition {
        name: "WARNING",
        description: "Warning about potential issues",
        priority: Priority::High,
    },
    TagDefinition {
        name: "FIX",
        description: "Quick fix needed",
        priority: Priority::High,
    },
    // Low priority - Info category
    TagDefinition {
        name: "NOTE",
        description: "Notes and documentation",
        priority: Priority::Low,
    },
    TagDefinition {
        name: "XXX",
        description: "Items requiring attention",
        priority: Priority::Low,
    },
    TagDefinition {
        name: "INFO",
        description: "Informational notes",
        priority: Priority::Low,
    },
    TagDefinition {
        name: "DOCS",
        description: "Documentation needed",
        priority: Priority::Low,
    },
    TagDefinition {
        name: "PERF",
        description: "Performance issues",
        priority: Priority::Low,
    },
    TagDefinition {
        name: "TEST",
        description: "Test-related items",
        priority: Priority::Low,
    },
    TagDefinition {
        name: "IDEA",
        description: "Ideas for future consideration",
        priority: Priority::Low,
    },
];

/// Get tag names as a vector of strings
pub fn default_tag_names() -> Vec<String> {
    DEFAULT_TAGS.iter().map(|t| t.name.to_string()).collect()
}

/// Find a tag definition by name (case-insensitive)
pub fn find_tag(name: &str) -> Option<&'static TagDefinition> {
    DEFAULT_TAGS
        .iter()
        .find(|t| t.name.eq_ignore_ascii_case(name))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_tags_count() {
        assert_eq!(DEFAULT_TAGS.len(), 17);
    }

    #[test]
    fn test_default_tags_contains_todo() {
        assert!(DEFAULT_TAGS.iter().any(|t| t.name == "TODO"));
    }

    #[test]
    fn test_default_tags_contains_fixme() {
        assert!(DEFAULT_TAGS.iter().any(|t| t.name == "FIXME"));
    }

    #[test]
    fn test_default_tags_priorities() {
        let critical_tags: Vec<_> = DEFAULT_TAGS
            .iter()
            .filter(|t| t.priority == Priority::Critical)
            .collect();
        assert_eq!(critical_tags.len(), 3); // FIXME, BUG, ERROR

        let high_tags: Vec<_> = DEFAULT_TAGS
            .iter()
            .filter(|t| t.priority == Priority::High)
            .collect();
        assert_eq!(high_tags.len(), 4); // HACK, WARN, WARNING, FIX

        let medium_tags: Vec<_> = DEFAULT_TAGS
            .iter()
            .filter(|t| t.priority == Priority::Medium)
            .collect();
        assert_eq!(medium_tags.len(), 3); // TODO, WIP, MAYBE

        let low_tags: Vec<_> = DEFAULT_TAGS
            .iter()
            .filter(|t| t.priority == Priority::Low)
            .collect();
        assert_eq!(low_tags.len(), 7); // NOTE, XXX, INFO, DOCS, PERF, TEST, IDEA
    }

    #[test]
    fn test_default_tag_names() {
        let names = default_tag_names();
        assert_eq!(names.len(), 17);
        assert!(names.contains(&"TODO".to_string()));
        assert!(names.contains(&"FIXME".to_string()));
        assert!(names.contains(&"BUG".to_string()));
    }

    #[test]
    fn test_find_tag() {
        let tag = find_tag("TODO");
        assert!(tag.is_some());
        assert_eq!(tag.unwrap().name, "TODO");
    }

    #[test]
    fn test_find_tag_case_insensitive() {
        let tag = find_tag("todo");
        assert!(tag.is_some());
        assert_eq!(tag.unwrap().name, "TODO");

        let tag = find_tag("FiXmE");
        assert!(tag.is_some());
        assert_eq!(tag.unwrap().name, "FIXME");
    }

    #[test]
    fn test_find_tag_not_found() {
        let tag = find_tag("NONEXISTENT");
        assert!(tag.is_none());
    }

    #[test]
    fn test_tag_definition_equality() {
        let tag1 = TagDefinition {
            name: "TODO",
            description: "Test",
            priority: Priority::Medium,
        };

        let tag2 = TagDefinition {
            name: "TODO",
            description: "Test",
            priority: Priority::Medium,
        };

        assert_eq!(tag1, tag2);
    }

    #[test]
    fn test_all_tags_have_descriptions() {
        for tag in DEFAULT_TAGS {
            assert!(!tag.description.is_empty());
        }
    }

    #[test]
    fn test_all_tags_have_names() {
        for tag in DEFAULT_TAGS {
            assert!(!tag.name.is_empty());
        }
    }
}
