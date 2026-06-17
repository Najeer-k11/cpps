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
    let mut has_cpp_compiler = false;

    for tool in TOOLS {
        // On Windows, g++ and clang++ are alternatives — only need one
        #[cfg(target_os = "windows")]
        {
            if tool.name == "g++" {
                // Skip g++ check entirely on Windows — clang++ from LLVM is the primary option
                continue;
            }
        }

        match check_tool(tool) {
            ToolStatus::Found { version, path } => {
                if tool.name == "clang++" || tool.name == "g++" {
                    has_cpp_compiler = true;
                }
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
                if tool.name == "clang++" || tool.name == "g++" {
                    // Only report as issue if no C++ compiler at all
                    if !has_cpp_compiler {
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
                } else {
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
        "vcpkg" => {
            let mut paths = vec![
                std::path::PathBuf::from(r"C:\vcpkg\vcpkg.exe"),
                std::path::PathBuf::from(r"C:\src\vcpkg\vcpkg.exe"),
            ];
            if let Ok(home) = std::env::var("USERPROFILE") {
                paths.push(std::path::PathBuf::from(format!("{}\\vcpkg\\vcpkg.exe", home)));
            }
            paths
        },
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
    use indicatif::{ProgressBar, ProgressStyle};
    use std::io::{BufRead, BufReader};
    use std::time::Duration;

    if let Some(cmd) = platform.install_command(tool_name) {
        // Create a spinner progress bar for the install
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .tick_chars("⣾⣽⣻⢿⡿⣟⣯⣷")
                .template("  {spinner:.cyan} {msg}")
                .unwrap(),
        );
        pb.set_message(format!("Downloading {}...", tool_name));
        pb.enable_steady_tick(Duration::from_millis(80));

        // Spawn the process with piped output
        let mut child = match Command::new(&cmd[0])
            .args(&cmd[1..])
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
        {
            Ok(c) => c,
            Err(e) => {
                pb.finish_and_clear();
                print_error(&format!("Failed to run installer for {}: {}", tool_name, e));
                return;
            }
        };

        // Stream stdout to update progress message
        let mut stdout_content = String::new();
        let mut stderr_content = String::new();

        if let Some(out) = child.stdout.take() {
            let reader = BufReader::new(out);
            for line in reader.lines().map_while(Result::ok) {
                if !line.trim().is_empty() {
                    let display = if line.len() > 55 {
                        format!("{}...", &line[..52])
                    } else {
                        line.clone()
                    };
                    pb.set_message(format!("Installing {}: {}", tool_name, display));
                }
                stdout_content.push_str(&line);
                stdout_content.push('\n');
            }
        }
        if let Some(err) = child.stderr.take() {
            let reader = BufReader::new(err);
            for line in reader.lines().map_while(Result::ok) {
                stderr_content.push_str(&line);
                stderr_content.push('\n');
            }
        }

        let status = child.wait();
        pb.finish_and_clear();

        match status {
            Ok(s) if s.success() => {
                print_success(&format!("{} installed successfully", tool_name));

                // vcpkg needs bootstrap after clone
                #[cfg(target_os = "windows")]
                if tool_name == "vcpkg" {
                    let home = std::env::var("USERPROFILE").unwrap_or_default();
                    let vcpkg_dir = format!("{}\\vcpkg", home);
                    let bootstrap = format!("{}\\bootstrap-vcpkg.bat", vcpkg_dir);
                    if std::path::Path::new(&bootstrap).exists() {
                        let bp = ProgressBar::new_spinner();
                        bp.set_style(
                            ProgressStyle::default_spinner()
                                .tick_chars("⣾⣽⣻⢿⡿⣟⣯⣷")
                                .template("  {spinner:.cyan} {msg}")
                                .unwrap(),
                        );
                        bp.set_message("Bootstrapping vcpkg...");
                        bp.enable_steady_tick(Duration::from_millis(80));

                        let bs_result = Command::new("cmd")
                            .args(["/C", &format!("\"{}\" -disableMetrics", bootstrap)])
                            .current_dir(&vcpkg_dir)
                            .output();

                        bp.finish_and_clear();
                        match bs_result {
                            Ok(out) if out.status.success() => {
                                print_success("vcpkg bootstrapped");
                            }
                            _ => {
                                print_warning("Bootstrap failed — run bootstrap-vcpkg.bat manually");
                            }
                        }
                        print_info(&format!("Add to PATH: {}", vcpkg_dir));
                    }
                }
            }
            Ok(_) => {
                let combined = format!("{}{}", stdout_content, stderr_content);
                if combined.contains("already installed") || combined.contains("No available upgrade") {
                    print_success(&format!("{} is already installed", tool_name));
                    #[cfg(target_os = "windows")]
                    {
                        if tool_name == "clang++" || tool_name == "g++" {
                            print_info("If not detected, add C:\\Program Files\\LLVM\\bin to your PATH");
                        }
                    }
                } else {
                    let error_msg = if !stderr_content.trim().is_empty() {
                        stderr_content.trim().lines().last().unwrap_or("unknown error").to_string()
                    } else if !stdout_content.trim().is_empty() {
                        stdout_content.trim().lines().last().unwrap_or("unknown error").to_string()
                    } else {
                        "unknown error".to_string()
                    };
                    print_error(&format!("Failed to install {}: {}", tool_name, error_msg));
                }
            }
            Err(e) => {
                print_error(&format!("Install process error for {}: {}", tool_name, e));
            }
        }
    }
}

fn extract_version(output: &str) -> Option<String> {
    // Try date-based version first (vcpkg uses YYYY-MM-DD format)
    let date_re = regex::Regex::new(r"(\d{4}-\d{2}-\d{2})").ok()?;
    if let Some(caps) = date_re.captures(output) {
        return Some(caps[1].to_string());
    }

    // Standard semver-like version
    let re = regex::Regex::new(r"(\d+\.\d+[\.\d]*)").ok()?;
    re.captures(output)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().to_string())
}

fn is_version_sufficient(detected: &str, minimum: &str) -> bool {
    // Handle date-based versions (YYYY-MM-DD, used by vcpkg)
    if detected.contains('-') && minimum.contains('-') {
        return detected >= minimum;
    }
    // If minimum is just a year (e.g., "2023"), check if detected starts with >= that
    if minimum.len() == 4 && detected.contains('-') {
        let det_year: u32 = detected[..4].parse().unwrap_or(0);
        let min_year: u32 = minimum.parse().unwrap_or(0);
        return det_year >= min_year;
    }

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
