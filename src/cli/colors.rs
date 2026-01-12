use colored::*;

/// ANSI 256 color codes for terminal-friendly colors
/// Compatible with ghostty, kitty, and other modern terminals
pub struct Colors;

impl Colors {
    // Primary colors
    pub fn primary(text: &str) -> ColoredString {
        text.truecolor(51, 255, 255) // Bright cyan
    }
    
    pub fn success(text: &str) -> ColoredString {
        text.truecolor(46, 255, 87) // Bright green
    }
    
    pub fn warning(text: &str) -> ColoredString {
        text.truecolor(226, 255, 87) // Bright yellow
    }
    
    // Secondary colors
    pub fn info(text: &str) -> ColoredString {
        text.truecolor(39, 148, 255) // Bright blue
    }
    
    pub fn accent(text: &str) -> ColoredString {
        text.truecolor(201, 97, 255) // Bright magenta
    }
    
    pub fn error(text: &str) -> ColoredString {
        text.truecolor(196, 0, 0) // Bright red
    }
    
    // Neutral colors
    pub fn text(text: &str) -> ColoredString {
        text.truecolor(255, 255, 255) // White
    }
    
    pub fn muted(text: &str) -> ColoredString {
        text.truecolor(244, 244, 244) // Light gray
    }
    
    // Status labels
    pub fn label_pass(text: &str) -> ColoredString {
        format!("[{}]", text).truecolor(46, 255, 87)
    }
    
    pub fn label_fail(text: &str) -> ColoredString {
        format!("[{}]", text).truecolor(196, 0, 0)
    }
    
    pub fn label_warn(text: &str) -> ColoredString {
        format!("[{}]", text).truecolor(226, 255, 87)
    }
    
    pub fn label_info(text: &str) -> ColoredString {
        format!("[{}]", text).truecolor(39, 148, 255)
    }
    
    pub fn label_input(text: &str) -> ColoredString {
        format!("[{}]", text).truecolor(51, 255, 255)
    }
    
    pub fn label_output(text: &str) -> ColoredString {
        format!("[{}]", text).truecolor(201, 97, 255)
    }
    
    pub fn label_tip(text: &str) -> ColoredString {
        format!("[{}]", text).truecolor(226, 255, 87)
    }
    
    pub fn label_gpu(text: &str) -> ColoredString {
        format!("[{}]", text).truecolor(201, 97, 255)
    }
    
    pub fn label_cpu(text: &str) -> ColoredString {
        format!("[{}]", text).truecolor(244, 244, 244)
    }
}

/// Helper to create box drawing borders
pub struct Borders;

impl Borders {
    pub fn top(width: usize) -> String {
        format!("╔{}╗", "═".repeat(width.saturating_sub(2)))
    }
    
    pub fn bottom(width: usize) -> String {
        format!("╚{}╝", "═".repeat(width.saturating_sub(2)))
    }
    
    pub fn separator(width: usize) -> String {
        "─".repeat(width)
    }
    
    pub fn box_line(content: &str, width: usize) -> String {
        let content_len = content.chars().count();
        let padding = width.saturating_sub(content_len + 4);
        format!("║ {} {}{} ║", content, " ".repeat(padding.saturating_sub(1)), if padding > 0 { "" } else { "" })
    }
    
    pub fn box_line_left(content: &str, width: usize) -> String {
        let content_len = content.chars().count();
        let padding = width.saturating_sub(content_len + 4);
        format!("║ {}{} ║", content, " ".repeat(padding))
    }
}
