use crate::cli::colors::Colors;

pub fn display_banner() {
    // Create a colorful gradient banner with alternating colors
    let border_color = Colors::primary;
    let cursed_lines = vec![
        "   ██████╗██╗   ██╗██████╗ ███████╗███████╗██████╗            ",
        "  ██╔════╝██║   ██║██╔══██╗██╔════╝██╔════╝██╔══██╗           ",
        "  ██║     ██║   ██║██████╔╝███████╗█████╗  ██║  ██║           ",
        "  ██║     ██║   ██║██╔══██╗╚════██║██╔══╝  ██║  ██║           ",
        "  ╚██████╗╚██████╔╝██║  ██║███████║███████╗██████╔╝           ",
        "   ╚═════╝ ╚═════╝ ╚═╝  ╚═╝╚══════╝╚══════╝╚═════╝            ",
    ];
    
    let coddy_lines = vec![
        "         ██████╗  ██████╗ ██████╗ ██╗   ██╗                   ",
        "        ██╔════╝ ██╔═══██╗██╔══██╗╚██╗ ██╔╝                   ",
        "        ██║      ██║   ██║██║  ██║ ╚████╔╝                    ",
        "        ██║      ██║   ██║██║  ██║  ╚██╔╝                     ",
        "        ╚██████╗ ╚██████╔╝██████╔╝   ██║                      ",
        "         ╚═════╝  ╚═════╝ ╚═════╝    ╚═╝                      ",
    ];
    
    // Top border
    println!("{}", border_color("╔══════════════════════════════════════════════════════════════╗"));
    println!("{}", border_color("║                                                              ║"));
    
    // "CURSED" text with gradient: cyan -> magenta -> yellow
    for (i, line) in cursed_lines.iter().enumerate() {
        let color = match i % 3 {
            0 => Colors::primary,   // Cyan
            1 => Colors::accent,     // Magenta
            _ => Colors::warning,   // Yellow
        };
        print!("{}", border_color("║"));
        print!("{}", color(line));
        println!("{}", border_color("║"));
    }
    
    println!("{}", border_color("║                                                              ║"));
    
    // "CODYY" text with gradient: green -> blue -> cyan
    for (i, line) in coddy_lines.iter().enumerate() {
        let color = match i % 3 {
            0 => Colors::success,   // Green
            1 => Colors::info,       // Blue
            _ => Colors::primary,    // Cyan
        };
        print!("{}", border_color("║"));
        print!("{}", color(line));
        println!("{}", border_color("║"));
    }
    
    println!("{}", border_color("║                                                              ║"));
    println!("{}", Colors::accent("║              CLI Coding Education Platform                   ║"));
    println!("{}", border_color("║                                                              ║"));
    println!("{}", border_color("╚══════════════════════════════════════════════════════════════╝"));
    println!();
}
