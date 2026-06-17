use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CppsConfig {
    pub project: ProjectConfig,
    #[serde(default)]
    pub compiler: CompilerConfig,
    #[serde(default)]
    pub build: BuildConfig,
    #[serde(default)]
    pub dependencies: BTreeMap<String, DependencySpec>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProjectConfig {
    pub name: String,
    #[serde(default = "default_version")]
    pub version: String,
    #[serde(default = "default_std")]
    pub std: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CompilerConfig {
    #[serde(default = "default_preferred")]
    pub preferred: String,
    #[serde(default)]
    pub flags: Vec<String>,
}

impl Default for CompilerConfig {
    fn default() -> Self {
        Self {
            preferred: default_preferred(),
            flags: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BuildConfig {
    #[serde(default = "default_src_dir")]
    pub src_dir: String,
    #[serde(default = "default_out_dir")]
    pub out_dir: String,
    #[serde(default = "default_entry")]
    pub entry: String,
}

impl Default for BuildConfig {
    fn default() -> Self {
        Self {
            src_dir: default_src_dir(),
            out_dir: default_out_dir(),
            entry: default_entry(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DependencySpec {
    pub version: String,
    #[serde(default = "default_source")]
    pub source: String,
}

fn default_version() -> String {
    "0.1.0".to_string()
}
fn default_std() -> String {
    "c++17".to_string()
}
fn default_preferred() -> String {
    "auto".to_string()
}
fn default_src_dir() -> String {
    "src".to_string()
}
fn default_out_dir() -> String {
    "build".to_string()
}
fn default_entry() -> String {
    "src/main.cpp".to_string()
}
fn default_source() -> String {
    "vcpkg".to_string()
}

const VALID_STDS: &[&str] = &["c++11", "c++14", "c++17", "c++20", "c++23"];
const VALID_PREFERRED: &[&str] = &["auto", "gcc", "clang", "msvc"];

impl CppsConfig {
    pub fn load(path: &Path) -> Result<Self, String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;

        let config: CppsConfig = toml::from_str(&content).map_err(|e| {
            let msg = e.message();
            if let Some(span) = e.span() {
                // Calculate line number from byte offset
                let line = content[..span.start].chars().filter(|c| *c == '\n').count() + 1;
                format!("Parse error in {} at line {}: {}", path.display(), line, msg)
            } else {
                format!("Parse error in {}: {}", path.display(), msg)
            }
        })?;

        config.validate()?;
        Ok(config)
    }

    pub fn save(&self, path: &Path) -> Result<(), String> {
        let content =
            toml::to_string_pretty(self).map_err(|e| format!("Failed to serialize config: {}", e))?;
        std::fs::write(path, content)
            .map_err(|e| format!("Failed to write {}: {}", path.display(), e))?;
        Ok(())
    }

    pub fn validate(&self) -> Result<(), String> {
        if !VALID_STDS.contains(&self.project.std.as_str()) {
            return Err(format!(
                "Invalid C++ standard '{}'. Valid values: {}",
                self.project.std,
                VALID_STDS.join(", ")
            ));
        }
        if !VALID_PREFERRED.contains(&self.compiler.preferred.as_str()) {
            return Err(format!(
                "Invalid compiler preference '{}'. Valid values: {}",
                self.compiler.preferred,
                VALID_PREFERRED.join(", ")
            ));
        }
        Ok(())
    }

    #[allow(dead_code)]
    pub fn new(name: &str) -> Self {
        Self {
            project: ProjectConfig {
                name: name.to_string(),
                version: default_version(),
                std: default_std(),
            },
            compiler: CompilerConfig::default(),
            build: BuildConfig::default(),
            dependencies: BTreeMap::new(),
        }
    }
}
