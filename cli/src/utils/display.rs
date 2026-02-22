use colored::Color;
use todo_tree_core::Priority;

pub fn priority_to_color(priority: Priority) -> Color {
    match priority {
        Priority::Critical => Color::Red,
        Priority::High => Color::Yellow,
        Priority::Medium => Color::Cyan,
        Priority::Low => Color::Green,
    }
}
