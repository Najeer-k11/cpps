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

    // Find vcpkg binary (PATH + common locations)
    let vcpkg_bin = find_vcpkg();
    let vcpkg_bin = match vcpkg_bin {
        Some(p) => p,
        None => {
            print_error("vcpkg not found. Run `cpps doctor --fix` to install it.");
            return 1;
        }
    };

    // Get platform triplet
    let plat = platform::current_platform();
    let triplet = plat.vcpkg_triplet();

    // Install via vcpkg
    let install_arg = format!("{}:{}", package, triplet);
    print_info(&format!("Installing {} (triplet: {})...", package, triplet));

    let spinner = progress::create_spinner(output_config, &format!("Installing {}...", package));

    let output = Command::new(&vcpkg_bin)
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
                    suggest_similar_packages(&vcpkg_bin, package);
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

            let dep_info = get_known_link_info(package);
            config.dependencies.insert(
                package.to_string(),
                DependencySpec {
                    version: version.clone(),
                    source: "vcpkg".to_string(),
                    link: dep_info.link,
                    cflags: dep_info.cflags,
                    subsystem: dep_info.subsystem,
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

/// Find vcpkg binary — checks PATH first, then common install locations
pub fn find_vcpkg() -> Option<PathBuf> {
    // Check PATH
    if let Ok(path) = which::which("vcpkg") {
        return Some(path);
    }

    // Check common locations
    let candidates = get_vcpkg_candidates();
    candidates.into_iter().find(|p| p.exists())
}

fn get_vcpkg_candidates() -> Vec<PathBuf> {
    let mut candidates = Vec::new();

    if let Ok(home) = std::env::var("USERPROFILE") {
        candidates.push(PathBuf::from(&home).join("vcpkg").join("vcpkg.exe"));
    }
    if let Ok(home) = std::env::var("HOME") {
        candidates.push(PathBuf::from(&home).join("vcpkg").join("vcpkg"));
    }

    candidates.push(PathBuf::from(r"C:\vcpkg\vcpkg.exe"));
    candidates.push(PathBuf::from(r"C:\src\vcpkg\vcpkg.exe"));
    candidates.push(PathBuf::from("/usr/local/bin/vcpkg"));
    candidates.push(PathBuf::from("/opt/vcpkg/vcpkg"));

    candidates
}

/// Get the vcpkg installed directory (where packages are installed)
pub fn get_vcpkg_root() -> Option<PathBuf> {
    let vcpkg_bin = find_vcpkg()?;
    vcpkg_bin.parent().map(|p| p.to_path_buf())
}

fn extract_version_from_output(output: &str) -> Option<String> {
    let re = regex::Regex::new(r"(\d+\.\d+[\.\d]*)").ok()?;
    for line in output.lines().rev() {
        if let Some(caps) = re.captures(line) {
            return Some(caps[1].to_string());
        }
    }
    None
}

fn suggest_similar_packages(vcpkg_bin: &PathBuf, package: &str) {
    let output = Command::new(vcpkg_bin)
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

/// Known linker requirements for popular packages
struct KnownDepInfo {
    link: Vec<String>,
    cflags: Vec<String>,
    subsystem: Option<String>,
}

fn get_known_link_info(package: &str) -> KnownDepInfo {
    match package {
        "sdl2" => {
            if cfg!(target_os = "windows") {
                KnownDepInfo {
                    link: vec![
                        "-lSDL2".into(),
                        "-lshell32".into(), "-lole32".into(),
                        "-loleaut32".into(), "-limm32".into(),
                        "-lversion".into(), "-lwinmm".into(),
                        "-lsetupapi".into(), "-lcfgmgr32".into(),
                    ],
                    cflags: vec![],
                    subsystem: Some("console".into()),
                }
            } else if cfg!(target_os = "macos") {
                KnownDepInfo {
                    link: vec![
                        "-lSDL2main".into(), "-lSDL2".into(),
                        "-framework".into(), "Cocoa".into(),
                        "-framework".into(), "IOKit".into(),
                        "-framework".into(), "CoreVideo".into(),
                    ],
                    cflags: vec![],
                    subsystem: None,
                }
            } else {
                KnownDepInfo {
                    link: vec!["-lSDL2main".into(), "-lSDL2".into()],
                    cflags: vec![],
                    subsystem: None,
                }
            }
        }
        "raylib" => {
            if cfg!(target_os = "windows") {
                KnownDepInfo {
                    link: vec![
                        "-lraylib".into(), "-lopengl32".into(),
                        "-lgdi32".into(), "-lwinmm".into(),
                        "-lshell32".into(),
                    ],
                    cflags: vec![],
                    subsystem: Some("console".into()),
                }
            } else if cfg!(target_os = "macos") {
                KnownDepInfo {
                    link: vec![
                        "-lraylib".into(),
                        "-framework".into(), "OpenGL".into(),
                        "-framework".into(), "Cocoa".into(),
                        "-framework".into(), "IOKit".into(),
                        "-framework".into(), "CoreVideo".into(),
                    ],
                    cflags: vec![],
                    subsystem: None,
                }
            } else {
                KnownDepInfo {
                    link: vec![
                        "-lraylib".into(), "-lGL".into(),
                        "-lm".into(), "-lpthread".into(),
                        "-ldl".into(), "-lrt".into(),
                    ],
                    cflags: vec![],
                    subsystem: None,
                }
            }
        }
        "fmt" => KnownDepInfo {
            link: vec!["-lfmt".into()],
            cflags: vec![],
            subsystem: None,
        },
        "imgui" => KnownDepInfo {
            link: vec!["-limgui".into()],
            cflags: vec![],
            subsystem: None,
        },
        "boost-filesystem" => {
            if cfg!(target_os = "windows") {
                // On Windows, boost libs have decorated names — we'll use auto-detection
                KnownDepInfo {
                    link: vec![],  // Auto-detected from vcpkg lib dir at build time
                    cflags: vec![],
                    subsystem: None,
                }
            } else {
                KnownDepInfo {
                    link: vec!["-lboost_filesystem".into()],
                    cflags: vec![],
                    subsystem: None,
                }
            }
        },
        "glfw3" => {
            if cfg!(target_os = "windows") {
                KnownDepInfo {
                    link: vec!["-lglfw3".into(), "-lopengl32".into(), "-lgdi32".into()],
                    cflags: vec![],
                    subsystem: None,
                }
            } else {
                KnownDepInfo {
                    link: vec!["-lglfw".into(), "-lGL".into()],
                    cflags: vec![],
                    subsystem: None,
                }
            }
        }
        _ => KnownDepInfo {
            link: vec![],
            cflags: vec![],
            subsystem: None,
        },
    }
}
