use std::path::PathBuf;
use std::process::Command;

use super::{CompilerInfo, CompilerType, PackageManager, Platform};

pub struct WindowsPlatform;

impl Platform for WindowsPlatform {
    fn find_compilers(&self) -> Vec<CompilerInfo> {
        let mut compilers = Vec::new();

        // Search PATH for g++
        if let Some(info) = detect_gcc() {
            compilers.push(info);
        }

        // Search PATH for clang++
        if let Some(info) = detect_clang() {
            compilers.push(info);
        }

        // Search for MSVC cl.exe
        if let Some(info) = detect_msvc() {
            compilers.push(info);
        }

        compilers
    }

    fn install_command(&self, tool: &str) -> Option<Vec<String>> {
        let cmd: Vec<String> = match tool {
            // clang++ comes from LLVM
            "g++" | "clang++" => vec![
                "winget".into(), "install".into(), "LLVM.LLVM".into(),
                "--accept-package-agreements".into(), "--accept-source-agreements".into(), "--silent".into(),
            ],
            "cmake" => vec![
                "winget".into(), "install".into(), "Kitware.CMake".into(),
                "--accept-package-agreements".into(), "--accept-source-agreements".into(), "--silent".into(),
            ],
            "ninja" => vec![
                "winget".into(), "install".into(), "Ninja-build.Ninja".into(),
                "--accept-package-agreements".into(), "--accept-source-agreements".into(), "--silent".into(),
            ],
            // vcpkg is not on winget — clone from GitHub
            "vcpkg" => {
                let home = std::env::var("USERPROFILE").unwrap_or_else(|_| r"C:\".to_string());
                vec![
                    "git".into(), "clone".into(),
                    "https://github.com/microsoft/vcpkg.git".into(),
                    format!("{}\\vcpkg", home),
                ]
            },
            _ => return None,
        };
        Some(cmd)
    }

    fn vcpkg_triplet(&self) -> &str {
        "x64-windows"
    }

    fn binary_extension(&self) -> &str {
        ".exe"
    }

    fn detect_package_manager(&self) -> Option<PackageManager> {
        if which::which("winget").is_ok() {
            Some(PackageManager::Winget)
        } else if which::which("choco").is_ok() {
            Some(PackageManager::Choco)
        } else {
            None
        }
    }

    fn default_compiler_ranking(&self) -> Vec<CompilerType> {
        vec![CompilerType::Msvc, CompilerType::Clang, CompilerType::Gcc]
    }
}

fn detect_gcc() -> Option<CompilerInfo> {
    let path = which::which("g++").ok()?;
    let output = Command::new(&path).arg("--version").output().ok()?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let version = parse_gcc_version(&stdout)?;
    Some(CompilerInfo {
        name: "g++".to_string(),
        version,
        path,
        compiler_type: CompilerType::Gcc,
    })
}

fn detect_clang() -> Option<CompilerInfo> {
    // Try PATH first
    if let Ok(path) = which::which("clang++") {
        let output = Command::new(&path).arg("--version").output().ok()?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        let version = parse_clang_version(&stdout)?;
        return Some(CompilerInfo {
            name: "clang++".to_string(),
            version,
            path,
            compiler_type: CompilerType::Clang,
        });
    }

    // Check common LLVM install locations on Windows
    let common_paths = [
        PathBuf::from(r"C:\Program Files\LLVM\bin\clang++.exe"),
        PathBuf::from(r"C:\Program Files (x86)\LLVM\bin\clang++.exe"),
    ];

    for path in &common_paths {
        if path.exists() {
            let output = Command::new(path).arg("--version").output().ok()?;
            let stdout = String::from_utf8_lossy(&output.stdout);
            let version = parse_clang_version(&stdout)?;
            return Some(CompilerInfo {
                name: "clang++".to_string(),
                version,
                path: path.clone(),
                compiler_type: CompilerType::Clang,
            });
        }
    }

    None
}

fn detect_msvc() -> Option<CompilerInfo> {
    // Try VSINSTALLDIR environment variable
    if let Ok(vs_dir) = std::env::var("VSINSTALLDIR") {
        let cl_path = PathBuf::from(&vs_dir)
            .join("VC")
            .join("Tools")
            .join("MSVC");
        if cl_path.exists() {
            // Find cl.exe in the MSVC tools directory
            if let Some(info) = find_cl_in_dir(&cl_path) {
                return Some(info);
            }
        }
    }

    // Try PATH
    if let Ok(path) = which::which("cl") {
        let output = Command::new(&path).output().ok()?;
        let stderr = String::from_utf8_lossy(&output.stderr);
        let version = parse_msvc_version(&stderr)?;
        return Some(CompilerInfo {
            name: "cl.exe".to_string(),
            version,
            path,
            compiler_type: CompilerType::Msvc,
        });
    }

    None
}

fn find_cl_in_dir(msvc_dir: &PathBuf) -> Option<CompilerInfo> {
    // Walk MSVC directory to find cl.exe
    let entries = std::fs::read_dir(msvc_dir).ok()?;
    for entry in entries.flatten() {
        let version_dir = entry.path();
        let cl_path = version_dir
            .join("bin")
            .join("Hostx64")
            .join("x64")
            .join("cl.exe");
        if cl_path.exists() {
            let output = Command::new(&cl_path).output().ok()?;
            let stderr = String::from_utf8_lossy(&output.stderr);
            let version = parse_msvc_version(&stderr)?;
            return Some(CompilerInfo {
                name: "cl.exe".to_string(),
                version,
                path: cl_path,
                compiler_type: CompilerType::Msvc,
            });
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

fn parse_msvc_version(output: &str) -> Option<String> {
    let re = regex::Regex::new(r"Version\s+(\d+\.\d+)").ok()?;
    re.captures(output)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().to_string())
}
