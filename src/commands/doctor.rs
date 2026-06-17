use std::process::Command;

use crate::compiler::cache::CompilerCache;
use crate::output::colors::{print_error, print_header, print_info, print_success, print_warning};
use crate::platform::{self, Platform};

struct ToolCheck {
    name: &'static str,
    binary: &'static str,
    version_flag: &'static str,
    min_version: &'static str,
}

const TOOLS: &[ToolCheck] = &[
    ToolCheck {
        name: "g++",
        binary: "g++",
        version_flag: "--version",
        min_version: "11",
    },
    ToolCheck {
        name: "clang++",
        binary: "clang++",
        version_flag: "--version",
        min_version: "14",
    },
    ToolCheck {
        name: "cmake",
        binary: "cmake",
        version_flag: "--version",
        min_version: "3.20",
    },
    ToolCheck {
        name: "ninja",
        binary: "ninja",
        version_flag: "--version",
        min_version: "1.10",
    },
    ToolCheck {
        name: "vcpkg",
        binary: "vcpkg",
        version_flag: "version",
        min_version: "2023",
    },
];

pub fn execute(fix: bool) -> i32 {
    print_header("Checking C++ environment...");

    let platform = platform::current_platform();
    let mut issues = 0;

    for tool in TOOLS {
        match check_tool(tool) {
            ToolStatus::Found { version, path } => {
                if is_version_sufficient(&version, tool.min_version) {
                    print_success(&format!(
                        "{} {} found at {}",
                        tool.name, version, path
                    ));
                } else {
                    print_warning(&format!(
                        "{} {} found (minimum: {})",
                        tool.name, version, tool.min_version
                    ));
                    issues += 1;
                }
            }
            ToolStatus::NotFound => {
                if let Some(install_cmd) = platform.install_command(tool.name) {
                    print_error(&format!(
                        "{} not found  →  install: {}",
                        tool.name,
                        install_cmd.join(" ")
                    ));
                } else {
                    print_error(&format!("{} not found", tool.name));
                }
                issues += 1;

                if fix {
                    attempt_install(&platform, tool.name);
                }
            }
        }
    }

    // Check MSVC on Windows
    #[cfg(target_os = "windows")]
    {
        match check_msvc() {
            ToolStatus::Found { version, path } => {
                print_success(&format!("MSVC cl.exe {} found at {}", version, path));
            }
            ToolStatus::NotFound => {
                print_warning("MSVC not found (Windows only — install Visual Studio Build Tools)");
            }
        }
    }

    // Cache the compiler detection results
    let compilers = platform.find_compilers();
    if let Err(e) = CompilerCache::save(&compilers) {
        print_warning(&format!("Failed to cache compiler info: {}", e));
    }

    println!();
    if issues == 0 {
        print_success("All tools found and up to date!");
        0
    } else {
        print_info(&format!(
            "{} issue{} found. Run `cpps doctor --fix` to auto-install missing tools.",
            issues,
            if issues == 1 { "" } else { "s" }
        ));
        1
    }
}

enum ToolStatus {
    Found { version: String, path: String },
    NotFound,
}

fn check_tool(tool: &ToolCheck) -> ToolStatus {
    // First try PATH
    let path_result = which::which(tool.binary);

    // On Windows, also check common install locations for certain tools
    let resolved_path = match path_result {
        Ok(p) => Some(p),
        Err(_) => {
            #[cfg(target_os = "windows")]
            {
                find_in_common_locations(tool.binary)
            }
            #[cfg(not(target_os = "windows"))]
            {
                None
            }
        }
    };

    match resolved_path {
        Some(path) => {
            let output = Command::new(&path)
                .arg(tool.version_flag)
                .output();

            match output {
                Ok(out) => {
                    let stdout = String::from_utf8_lossy(&out.stdout);
                    let stderr = String::from_utf8_lossy(&out.stderr);
                    let combined = format!("{}{}", stdout, stderr);

                    let version = extract_version(&combined)
                        .unwrap_or_else(|| "unknown".to_string());

                    ToolStatus::Found {
                        version,
                        path: path.to_string_lossy().to_string(),
                    }
                }
                Err(_) => ToolStatus::NotFound,
            }
        }
        None => ToolStatus::NotFound,
    }
}

#[cfg(target_os = "windows")]
fn find_in_common_locations(binary: &str) -> Option<std::path::PathBuf> {
    let locations: Vec<std::path::PathBuf> = match binary {
        "clang++" => vec![
            std::path::PathBuf::from(r"C:\Program Files\LLVM\bin\clang++.exe"),
            std::path::PathBuf::from(r"C:\Program Files (x86)\LLVM\bin\clang++.exe"),
        ],
        "ninja" => vec![
            std::path::PathBuf::from(r"C:\Program Files\Ninja\ninja.exe"),
        ],
        _ => vec![],
    };

    locations.into_iter().find(|p| p.exists())
}

#[cfg(target_os = "windows")]
fn check_msvc() -> ToolStatus {
    use crate::platform::windows::WindowsPlatform;
    use crate::platform::Platform;

    let platform = WindowsPlatform;
    let compilers = platform.find_compilers();
    for c in compilers {
        if c.compiler_type == crate::platform::CompilerType::Msvc {
            return ToolStatus::Found {
                version: c.version,
                path: c.path.to_string_lossy().to_string(),
            };
        }
    }
    ToolStatus::NotFound
}

fn attempt_install(platform: &Box<dyn Platform>, tool_name: &str) {
    if let Some(cmd) = platform.install_command(tool_name) {
        print_info(&format!("Attempting to install {}...", tool_name));

        let result = if cfg!(target_os = "windows") {
            Command::new(&cmd[0])
                .args(&cmd[1..])
                .output()
        } else {
            Command::new(&cmd[0])
                .args(&cmd[1..])
                .output()
        };

        match result {
            Ok(output) => {
                if output.status.success() {
                    print_success(&format!("{} installed successfully", tool_name));
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    print_error(&format!("Failed to install {}: {}", tool_name, stderr.trim()));
                }
            }
            Err(e) => {
                print_error(&format!("Failed to run installer for {}: {}", tool_name, e));
            }
        }
    }
}

fn extract_version(output: &str) -> Option<String> {
    let re = regex::Regex::new(r"(\d+\.\d+[\.\d]*)").ok()?;
    re.captures(output)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().to_string())
}

fn is_version_sufficient(detected: &str, minimum: &str) -> bool {
    let det_parts: Vec<u32> = detected
        .split('.')
        .filter_map(|s| s.parse().ok())
        .collect();
    let min_parts: Vec<u32> = minimum
        .split('.')
        .filter_map(|s| s.parse().ok())
        .collect();

    for (d, m) in det_parts.iter().zip(min_parts.iter()) {
        if d > m {
            return true;
        }
        if d < m {
            return false;
        }
    }
    det_parts.len() >= min_parts.len()
}
