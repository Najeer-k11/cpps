use colored::Colorize;
use std::path::Path;

use super::parser::{Diagnostic, Severity};

/// Format and display a list of diagnostics
pub fn display_diagnostics(diagnostics: &[Diagnostic]) {
    if diagnostics.is_empty() {
        return;
    }

    for diag in diagnostics {
        display_single(diag);
    }

    let error_count = diagnostics
        .iter()
        .filter(|d| d.severity == Severity::Error)
        .count();
    let warning_count = diagnostics
        .iter()
        .filter(|d| d.severity == Severity::Warning)
        .count();

    eprintln!();
    if error_count > 0 {
        eprintln!(
            "  {} {}",
            "✗".red(),
            format!(
                "{} error{}, {} warning{}",
                error_count,
                if error_count == 1 { "" } else { "s" },
                warning_count,
                if warning_count == 1 { "" } else { "s" }
            )
        );
    }
}

/// Display raw compiler output when parsing fails
pub fn display_raw_output(stderr: &str) {
    eprintln!(
        "  {} {}",
        "⚠".yellow(),
        "Note: structured parsing failed, showing raw compiler output:".yellow()
    );
    eprintln!();
    for line in stderr.lines() {
        eprintln!("    {}", line);
    }
}

fn display_single(diag: &Diagnostic) {
    let severity_str = match diag.severity {
        Severity::Error => format!("{}", "error".red().bold()),
        Severity::Warning => format!("{}", "warning".yellow().bold()),
        Severity::Note => format!("{}", "note".cyan()),
    };

    let location = if let Some(col) = diag.column {
        format!("{}:{}:{}", diag.file.display(), diag.line, col)
    } else {
        format!("{}:{}", diag.file.display(), diag.line)
    };

    eprintln!("  {}: {}", severity_str, diag.message);
    eprintln!("    {} {}", "-->".blue(), location);

    // Try to show the source line with caret
    if let Some(source_line) = read_source_line(&diag.file, diag.line) {
        eprintln!("     |");
        eprintln!("  {:>3} | {}", diag.line, source_line);
        if let Some(col) = diag.column {
            let padding = " ".repeat(col.saturating_sub(1) as usize);
            eprintln!("     | {}{}", padding, "^".red().bold());
        }
        eprintln!("     |");
    }

    // Show suggestion if available
    if let Some(ref suggestion) = diag.suggestion {
        eprintln!("  {} {}", "→".cyan(), suggestion.cyan());
    }

    eprintln!();
}

fn read_source_line(file: &Path, line: u32) -> Option<String> {
    let content = std::fs::read_to_string(file).ok()?;
    content
        .lines()
        .nth((line.saturating_sub(1)) as usize)
        .map(|s| s.to_string())
}
