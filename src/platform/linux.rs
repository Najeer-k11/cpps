use std::process::Command;

use super::{CompilerInfo, CompilerType, PackageManager, Platform};

pub struct LinuxPlatform;

impl Platform for LinuxPlatform {
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
        let pkg_mgr = self.detect_package_manager()?;
        let (cmd, pkg) = match (&pkg_mgr, tool) {
            (PackageManager::Apt, "g++") => ("apt", "g++"),
            (PackageManager::Apt, "clang++") => ("apt", "clang"),
            (PackageManager::Apt, "cmake") => ("apt", "cmake"),
            (PackageManager::Apt, "ninja") => ("apt", "ninja-build"),
            (PackageManager::Dnf, "g++") => ("dnf", "gcc-c++"),
            (PackageManager::Dnf, "clang++") => ("dnf", "clang"),
            (PackageManager::Dnf, "cmake") => ("dnf", "cmake"),
            (PackageManager::Dnf, "ninja") => ("dnf", "ninja-build"),
            (PackageManager::Pacman, "g++") => ("pacman", "gcc"),
            (PackageManager::Pacman, "clang++") => ("pacman", "clang"),
            (PackageManager::Pacman, "cmake") => ("pacman", "cmake"),
            (PackageManager::Pacman, "ninja") => ("pacman", "ninja"),
            _ => return None,
        };
        Some(vec![
            "sudo".to_string(),
            cmd.to_string(),
            "install".to_string(),
            "-y".to_string(),
            pkg.to_string(),
        ])
    }

    fn vcpkg_triplet(&self) -> &str {
        "x64-linux"
    }

    fn binary_extension(&self) -> &str {
        ""
    }

    fn detect_package_manager(&self) -> Option<PackageManager> {
        if which::which("apt").is_ok() {
            Some(PackageManager::Apt)
        } else if which::which("dnf").is_ok() {
            Some(PackageManager::Dnf)
        } else if which::which("pacman").is_ok() {
            Some(PackageManager::Pacman)
        } else {
            None
        }
    }

    fn default_compiler_ranking(&self) -> Vec<CompilerType> {
        vec![CompilerType::Clang, CompilerType::Gcc]
    }
}

fn detect_gcc() -> Option<CompilerInfo> {
    let candidates = ["g++-14", "g++-13", "g++-12", "g++"];
    for candidate in candidates {
        if let Ok(path) = which::which(candidate) {
            let output = Command::new(&path).arg("--version").output().ok()?;
            let stdout = String::from_utf8_lossy(&output.stdout);
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
    let candidates = ["clang++-18", "clang++-17", "clang++-16", "clang++"];
    for candidate in candidates {
        if let Ok(path) = which::which(candidate) {
            let output = Command::new(&path).arg("--version").output().ok()?;
            let stdout = String::from_utf8_lossy(&output.stdout);
            if let Some(version) = parse_clang_version(&stdout) {
                return Some(CompilerInfo {
                    name: "clang++".to_string(),
                    version,
                    path,
                    compiler_type: CompilerType::Clang,
                });
            }
        }
    }
    None
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
