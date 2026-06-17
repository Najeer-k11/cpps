use std::path::PathBuf;

#[cfg(target_os = "windows")]
pub mod windows;
#[cfg(target_os = "macos")]
pub mod macos;
#[cfg(target_os = "linux")]
pub mod linux;

#[derive(Debug, Clone, PartialEq)]
pub enum CompilerType {
    Gcc,
    Clang,
    Msvc,
}

impl std::fmt::Display for CompilerType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompilerType::Gcc => write!(f, "gcc"),
            CompilerType::Clang => write!(f, "clang"),
            CompilerType::Msvc => write!(f, "msvc"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CompilerInfo {
    pub name: String,
    pub version: String,
    pub path: PathBuf,
    pub compiler_type: CompilerType,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum PackageManager {
    Winget,
    Choco,
    Brew,
    Apt,
    Dnf,
    Pacman,
}

pub trait Platform {
    /// Find compiler binaries on the system
    fn find_compilers(&self) -> Vec<CompilerInfo>;

    /// Get the install command for a missing tool
    fn install_command(&self, tool: &str) -> Option<Vec<String>>;

    /// Get the vcpkg triplet for this platform
    fn vcpkg_triplet(&self) -> &str;

    /// Get the binary extension (.exe on Windows, empty elsewhere)
    #[allow(dead_code)]
    fn binary_extension(&self) -> &str;

    /// Detect which package manager is available
    #[allow(dead_code)]
    fn detect_package_manager(&self) -> Option<PackageManager>;

    /// Get the default compiler ranking for this platform
    fn default_compiler_ranking(&self) -> Vec<CompilerType>;
}

pub fn current_platform() -> Box<dyn Platform> {
    #[cfg(target_os = "windows")]
    {
        Box::new(windows::WindowsPlatform)
    }
    #[cfg(target_os = "macos")]
    {
        Box::new(macos::MacOsPlatform)
    }
    #[cfg(target_os = "linux")]
    {
        Box::new(linux::LinuxPlatform)
    }
}
