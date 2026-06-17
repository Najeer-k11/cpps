use std::path::PathBuf;

use crate::compiler::detect::CompilerDetector;
use crate::compiler::invoke::BuildCommand;
use crate::config::CppsConfig;
use crate::errors::{formatter, parser};
use crate::output::colors::{print_error, print_success};
use crate::output::progress;
use crate::output::OutputConfig;

pub fn execute(release: bool, output_config: &OutputConfig) -> i32 {
    // Find cpps.toml
    let config_path = find_config_file();
    let config_path = match config_path {
        Some(p) => p,
        None => {
            print_error("No cpps.toml found. Run `cpps new <name>` to create a project or create a cpps.toml manually.");
            return 1;
        }
    };

    let project_dir = config_path.parent().unwrap().to_path_buf();

    // Load config
    let config = match CppsConfig::load(&config_path) {
        Ok(c) => c,
        Err(e) => {
            print_error(&e);
            return 1;
        }
    };

    // Detect compiler
    let detector = CompilerDetector::new();
    let compilers = detector.detect_all();
    let compiler = match detector.select_compiler(&compilers, &config.compiler.preferred) {
        Ok(c) => c,
        Err(e) => {
            print_error(&e);
            return 1;
        }
    };

    // Build
    let build_cmd = match BuildCommand::from_config(
        &project_dir,
        compiler,
        &config.project.std,
        &config.compiler.flags,
        &config.build.src_dir,
        &config.build.out_dir,
        &config.build.entry,
        release,
        Vec::new(),
        Vec::new(),
        Vec::new(),
    ) {
        Ok(cmd) => cmd,
        Err(e) => {
            print_error(&e);
            return 1;
        }
    };

    let mode_label = if release { "Release build" } else { "Debug build" };
    let spinner = progress::create_spinner(output_config, &format!("{}...", mode_label));

    let result = match build_cmd.execute() {
        Ok(r) => r,
        Err(e) => {
            if let Some(s) = spinner {
                s.finish_and_clear();
            }
            print_error(&e);
            return 1;
        }
    };

    if let Some(s) = spinner {
        s.finish_and_clear();
    }

    if !result.success {
        let error_parser = parser::parser_for(&compiler.compiler_type);
        let diagnostics = error_parser.parse(&result.stderr_output);
        if diagnostics.is_empty() {
            formatter::display_raw_output(&result.stderr_output);
        } else {
            formatter::display_diagnostics(&diagnostics);
        }
        return 1;
    }

    // Print success info
    if let Some(ref binary_path) = result.binary_path {
        let abs_path = std::fs::canonicalize(binary_path)
            .unwrap_or_else(|_| binary_path.clone());
        let size_str = format_size(result.binary_size.unwrap_or(0));
        print_success(&format!(
            "Built {} ({}) in {:.2}s",
            abs_path.display(),
            size_str,
            result.duration.as_secs_f64()
        ));
    }

    0
}

fn format_size(bytes: u64) -> String {
    if bytes >= 1_048_576 {
        format!("{:.1} MB", bytes as f64 / 1_048_576.0)
    } else if bytes >= 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{} bytes", bytes)
    }
}

fn find_config_file() -> Option<PathBuf> {
    let mut dir = std::env::current_dir().ok()?;
    loop {
        let candidate = dir.join("cpps.toml");
        if candidate.exists() {
            return Some(candidate);
        }
        if !dir.pop() {
            return None;
        }
    }
}
