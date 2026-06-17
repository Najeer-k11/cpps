use std::process::Command;

use super::{CompilerInfo, CompilerType, PackageManager, Platform};

pub struct MacOsPlatform;

impl Platform for MacOsPlatform {
    fn find_compilers(&self) -> Vec<CompilerInfo> {
        let mut compilers = Vec::new();

        if let Some(info) = detect_clang() {
            compilers.push(info);
        }

        if let Some(info) = detect_gcc() {
            compilers.push(info);
        }

        compilers
    }

    fn install_command(&self, tool: &str) -> Option<Vec<String>> {
        let cmd = match tool {
            "g++" => vec!["brew", "install", "gcc"],
            "clang++" => vec!["xcode-select", "--install"],
            "cmake" => vec!["brew", "install", "cmake"],
            "ninja" => vec!["brew", "install", "ninja"],
            "vcpkg" => vec!["brew", "install", "vcpkg"],
            _ => return None,
        };
        Some(cmd.into_iter().map(String::from).collect())
    }

    fn vcpkg_triplet(&self) -> &str {
        if cfg!(target_arch = "aarch64") {
            "arm64-osx"
        } else {
            "x64-osx"
        }
    }

    fn binary_extension(&self) -> &str {
        ""
    }

    fn detect_package_manager(&self) -> Option<PackageManager> {
        if which::which("brew").is_ok() {
            Some(PackageManager::Brew)
        } else {
            None
        }
    }

    fn default_compiler_ranking(&self) -> Vec<CompilerType> {
        vec![CompilerType::Clang, CompilerType::Gcc]
    }
}

fn detect_gcc() -> Option<CompilerInfo> {
    // Try versioned gcc first, then plain g++
    let candidates = ["g++-14", "g++-13", "g++-12", "g++"];
    for candidate in candidates {
        if let Ok(path) = which::which(candidate) {
            let output = Command::new(&path).arg("--version").output().ok()?;
            let stdout = String::from_utf8_lossy(&output.stdout);
            // Skip if this is actually clang masquerading as g++
            if stdout.contains("clang") {
                continue;
            }
            if let Some(version) = parse_gcc_version(&stdout) {
                return Some(CompilerInfo {
                    name: "g++".to_string(),
                    version,
                    path,
                    compiler_type: CompilerType::Gcc,
                });
            }
        }
    }
    None
}

fn detect_clang() -> Option<CompilerInfo> {
    let path = which::which("clang++").ok()?;
    let output = Command::new(&path).arg("--version").output().ok()?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let version = parse_clang_version(&stdout)?;
    Some(CompilerInfo {
        name: "clang++".to_string(),
        version,
        path,
        compiler_type: CompilerType::Clang,
    })
}

fn parse_gcc_version(output: &str) -> Option<String> {
    let re = regex::Regex::new(r"(\d+\.\d+\.\d+)").ok()?;
    re.captures(output)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().to_string())
}

fn parse_clang_version(output: &str) -> Option<String> {
    let re = regex::Regex::new(r"version\s+(\d+\.\d+\.\d+)").ok()?;
    re.captures(output)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().to_string())
}
