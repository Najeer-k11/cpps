use std::path::{Path, PathBuf};
use std::process::Command;

use crate::commands::build::{resolve_vcpkg_deps, resolve_dep_cflags, resolve_subsystem};
use crate::compiler::detect::CompilerDetector;
use crate::compiler::invoke::BuildCommand;
use crate::config::CppsConfig;
use crate::errors::{formatter, parser};
use crate::output::colors::print_error;
use crate::output::progress;
use crate::output::OutputConfig;
use crate::platform;

pub fn execute(file: Option<&str>, output_config: &OutputConfig) -> i32 {
    match file {
        Some(file_path) => run_single_file(file_path, output_config),
        None => run_project_mode(output_config),
    }
}

fn run_single_file(file_path: &str, output_config: &OutputConfig) -> i32 {
    let path = Path::new(file_path);

    // Validate file exists
    if !path.exists() {
        print_error(&format!("File '{}' not found", file_path));
        return 1;
    }

    // Validate extension
    let ext = path
        .extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .unwrap_or_default();

    if !["cpp", "cc", "cxx"].contains(&ext.as_str()) {
        print_error(&format!(
            "File '{}' is not a recognized C++ source file (.cpp, .cc, .cxx)",
            file_path
        ));
        return 1;
    }

    // Detect compiler
    let detector = CompilerDetector::new();
    let compilers = detector.detect_all();
    let compiler = match detector.select_compiler(&compilers, "auto") {
        Ok(c) => c,
        Err(e) => {
            print_error(&e);
            return 1;
        }
    };

    // Build
    let build_cmd = match BuildCommand::single_file(path, compiler) {
        Ok(cmd) => cmd,
        Err(e) => {
            print_error(&e);
            return 1;
        }
    };

    let spinner = progress::create_spinner(output_config, "Compiling...");

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

    // Execute the binary
    let binary_path = result.binary_path.unwrap();
    run_binary(&binary_path)
}

fn run_project_mode(output_config: &OutputConfig) -> i32 {
    // Find cpps.toml
    let config_path = find_config_file();
    let config_path = match config_path {
        Some(p) => p,
        None => {
            print_error("No cpps.toml found. Run `cpps new <name>` to create a project, or provide a file: `cpps run main.cpp`");
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

    // Resolve vcpkg dependency paths
    let plat = platform::current_platform();
    let triplet = plat.vcpkg_triplet();
    let (include_paths, lib_paths, link_flags) = resolve_vcpkg_deps(&config, triplet);
    let dep_cflags = resolve_dep_cflags(&config);
    let subsystem = resolve_subsystem(&config);

    // Merge dep cflags with user flags
    let mut all_flags: Vec<String> = config.compiler.flags.clone();
    all_flags.extend(dep_cflags);

    // Build
    let build_cmd = match BuildCommand::from_config(
        &project_dir,
        compiler,
        &config.project.std,
        &all_flags,
        &config.build.src_dir,
        &config.build.out_dir,
        &config.build.entry,
        false,
        include_paths,
        lib_paths,
        link_flags,
        subsystem,
    ) {
        Ok(cmd) => cmd,
        Err(e) => {
            print_error(&e);
            return 1;
        }
    };

    let spinner = progress::create_spinner(output_config, "Compiling...");

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

    // Execute the binary
    let binary_path = result.binary_path.unwrap();
    run_binary(&binary_path)
}

fn run_binary(binary_path: &Path) -> i32 {
    let status = Command::new(binary_path)
        .stdin(std::process::Stdio::inherit())
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .status();

    match status {
        Ok(s) => {
            let code = s.code().unwrap_or(1);
            if code != 0 {
                eprintln!("\n  Process exited with code {}", code);
            }
            code
        }
        Err(e) => {
            print_error(&format!("Failed to execute binary: {}", e));
            1
        }
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
