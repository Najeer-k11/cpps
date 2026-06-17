use std::path::PathBuf;

use crate::platform::CompilerType;

#[derive(Debug, Clone, PartialEq)]
pub enum Severity {
    Error,
    Warning,
    Note,
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Severity::Error => write!(f, "error"),
            Severity::Warning => write!(f, "warning"),
            Severity::Note => write!(f, "note"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub file: PathBuf,
    pub line: u32,
    pub column: Option<u32>,
    pub severity: Severity,
    pub message: String,
    pub suggestion: Option<String>,
}

pub trait ErrorParser {
    fn parse(&self, stderr: &str) -> Vec<Diagnostic>;
}

/// Parser for GCC and Clang error output
/// Format: file:line:column: severity: message
pub struct GccClangParser;

impl ErrorParser for GccClangParser {
    fn parse(&self, stderr: &str) -> Vec<Diagnostic> {
        let re = regex::Regex::new(
            r"^(.+?):(\d+):(\d+):\s*(error|warning|note):\s*(.+)$"
        )
        .unwrap();

        let mut diagnostics = Vec::new();

        for line in stderr.lines() {
            if let Some(caps) = re.captures(line) {
                let file = PathBuf::from(&caps[1]);
                let line_num: u32 = caps[2].parse().unwrap_or(0);
                let column: u32 = caps[3].parse().unwrap_or(0);
                let severity = match &caps[4] {
                    "error" => Severity::Error,
                    "warning" => Severity::Warning,
                    "note" => Severity::Note,
                    _ => Severity::Error,
                };
                let message = caps[5].to_string();

                let suggestion = super::suggestions::suggest_fix(&message);

                diagnostics.push(Diagnostic {
                    file,
                    line: line_num,
                    column: Some(column),
                    severity,
                    message,
                    suggestion,
                });
            }
        }

        diagnostics
    }
}

/// Parser for MSVC cl.exe error output
/// Format: file(line): severity Cnnnn: message
pub struct MsvcParser;

impl ErrorParser for MsvcParser {
    fn parse(&self, stderr: &str) -> Vec<Diagnostic> {
        let re = regex::Regex::new(
            r"^(.+?)\((\d+)\):\s*(error|warning|note)\s*[A-Z]\d+:\s*(.+)$"
        )
        .unwrap();

        let mut diagnostics = Vec::new();

        for line in stderr.lines() {
            if let Some(caps) = re.captures(line) {
                let file = PathBuf::from(&caps[1]);
                let line_num: u32 = caps[2].parse().unwrap_or(0);
                let severity = match &caps[3] {
                    "error" => Severity::Error,
                    "warning" => Severity::Warning,
                    "note" => Severity::Note,
                    _ => Severity::Error,
                };
                let message = caps[4].to_string();

                let suggestion = super::suggestions::suggest_fix(&message);

                diagnostics.push(Diagnostic {
                    file,
                    line: line_num,
                    column: None,
                    severity,
                    message,
                    suggestion,
                });
            }
        }

        diagnostics
    }
}

pub fn parser_for(compiler_type: &CompilerType) -> Box<dyn ErrorParser> {
    match compiler_type {
        CompilerType::Gcc | CompilerType::Clang => Box::new(GccClangParser),
        CompilerType::Msvc => Box::new(MsvcParser),
    }
}
