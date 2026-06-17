use colored::Colorize;
use is_terminal::IsTerminal;

#[allow(dead_code)]
pub struct OutputConfig {
    pub color_enabled: bool,
    pub is_tty: bool,
}

impl OutputConfig {
    pub fn detect(no_color_flag: bool) -> Self {
        let is_tty = std::io::stdout().is_terminal();
        let color_enabled = is_tty && !no_color_flag;

        if !color_enabled {
            colored::control::set_override(false);
        }

        Self {
            color_enabled,
            is_tty,
        }
    }
}

pub fn print_success(msg: &str) {
    println!("  {} {}", "✓".green(), msg);
}

pub fn print_error(msg: &str) {
    eprintln!("  {} {}", "✗".red(), msg);
}

pub fn print_warning(msg: &str) {
    eprintln!("  {} {}", "⚠".yellow(), msg);
}

pub fn print_info(msg: &str) {
    println!("  {} {}", "→".cyan(), msg);
}

pub fn print_header(msg: &str) {
    println!("\n  {}\n", msg.bold());
}
