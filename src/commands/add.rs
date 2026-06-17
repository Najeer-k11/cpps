use std::path::PathBuf;
use std::process::Command;

use crate::config::model::{CppsConfig, DependencySpec};
use crate::output::colors::{print_error, print_info, print_success};
use crate::output::progress;
use crate::output::OutputConfig;
use crate::platform;

pub fn execute(package: &str, output_config: &OutputConfig) -> i32 {
    // Find cpps.toml
    let config_path = find_config_file();
    let config_path = match config_path {
        Some(p) => p,
        None => {
            print_error("No cpps.toml found. Run `cpps new <name>` to create a project or create a cpps.toml manually.");
            return 1;
        }
    };

    // Check vcpkg is available
    if which::which("vcpkg").is_err() {
        print_error("vcpkg not found. Run `cpps doctor --fix` to install it.");
        return 1;
    }

    // Get platform triplet
    let plat = platform::current_platform();
    let triplet = plat.vcpkg_triplet();

    // Install via vcpkg
    let install_arg = format!("{}:{}", package, triplet);
    print_info(&format!("Installing {} (triplet: {})...", package, triplet));

    let spinner = progress::create_spinner(output_config, &format!("Installing {}...", package));

    let output = Command::new("vcpkg")
        .args(["install", &install_arg])
        .output();

    if let Some(s) = spinner {
        s.finish_and_clear();
    }

    match output {
        Ok(result) => {
            let stdout = String::from_utf8_lossy(&result.stdout);
            let stderr = String::from_utf8_lossy(&result.stderr);

            if !result.status.success() {
                let combined = format!("{}\n{}", stdout, stderr);

                // Check if package not found
                if combined.contains("does not exist") || combined.contains("not found") {
                    print_error(&format!("Package '{}' not found in vcpkg registry.", package));
                    suggest_similar_packages(package);
                } else {
                    print_error(&format!(
                        "vcpkg install failed: {}",
                        stderr.trim()
                    ));
                }
                return 1;
            }

            // Parse version from output
            let version = extract_version_from_output(&stdout)
                .unwrap_or_else(|| "latest".to_string());

            // Update cpps.toml
            let mut config = match CppsConfig::load(&config_path) {
                Ok(c) => c,
                Err(e) => {
                    print_error(&e);
                    return 1;
                }
            };

            let was_existing = config.dependencies.contains_key(package);

            config.dependencies.insert(
                package.to_string(),
                DependencySpec {
                    version: version.clone(),
                    source: "vcpkg".to_string(),
                },
            );

            if let Err(e) = config.save(&config_path) {
                print_error(&e);
                return 1;
            }

            if was_existing {
                print_success(&format!(
                    "Updated {} to version {}",
                    package, version
                ));
            } else {
                print_success(&format!(
                    "Added {} version {} to dependencies",
                    package, version
                ));
            }

            0
        }
        Err(e) => {
            print_error(&format!("Failed to run vcpkg: {}", e));
            1
        }
    }
}

fn extract_version_from_output(output: &str) -> Option<String> {
    // vcpkg output often contains: "package-name:triplet version#port-version"
    let re = regex::Regex::new(r"(\d+\.\d+[\.\d]*)").ok()?;
    for line in output.lines().rev() {
        if let Some(caps) = re.captures(line) {
            return Some(caps[1].to_string());
        }
    }
    None
}

fn suggest_similar_packages(package: &str) {
    // Try vcpkg search to find similar packages
    let output = Command::new("vcpkg")
        .args(["search", package])
        .output();

    if let Ok(result) = output {
        let stdout = String::from_utf8_lossy(&result.stdout);
        let suggestions: Vec<&str> = stdout
            .lines()
            .take(5)
            .filter(|l| !l.is_empty())
            .collect();

        if !suggestions.is_empty() {
            print_info("Similar packages:");
            for suggestion in suggestions {
                let name = suggestion.split_whitespace().next().unwrap_or(suggestion);
                print_info(&format!("  {}", name));
            }
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
