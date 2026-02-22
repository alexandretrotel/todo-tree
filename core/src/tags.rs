use crate::priority::Priority;

#[derive(Debug, Clone, PartialEq)]
pub struct TagDefinition {
    pub name: &'static str,
    pub description: &'static str,
    pub priority: Priority,
}

pub const DEFAULT_TAGS: &[TagDefinition] = &[
    // Medium
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
    // Critical
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
    // High
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
    // Low priority
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

pub fn default_tag_names() -> Vec<String> {
    DEFAULT_TAGS.iter().map(|t| t.name.to_string()).collect()
}

pub fn find_tag(name: &str) -> Option<&'static TagDefinition> {
    DEFAULT_TAGS
        .iter()
        .find(|t| t.name.eq_ignore_ascii_case(name))
}
