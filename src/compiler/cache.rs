use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::platform::{CompilerInfo, CompilerType};

const CACHE_VALIDITY: Duration = Duration::from_secs(24 * 60 * 60); // 24 hours

#[derive(Serialize, Deserialize)]
pub struct CompilerCacheFile {
    pub timestamp: u64,
    pub compilers: Vec<CachedCompiler>,
}

#[derive(Serialize, Deserialize)]
pub struct CachedCompiler {
    pub name: String,
    pub version: String,
    pub path: String,
    pub compiler_type: String,
}

pub struct CompilerCache;

impl CompilerCache {
    fn cache_path() -> Option<PathBuf> {
        let home = dirs::home_dir()?;
        Some(home.join(".cpps").join("env-cache.toml"))
    }

    pub fn load_if_valid() -> Option<Vec<CompilerInfo>> {
        let path = Self::cache_path()?;
        let content = std::fs::read_to_string(&path).ok()?;
        let cache: CompilerCacheFile = toml::from_str(&content).ok()?;

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .ok()?
            .as_secs();

        if now - cache.timestamp > CACHE_VALIDITY.as_secs() {
            return None; // Cache expired
        }

        let compilers = cache
            .compilers
            .into_iter()
            .filter_map(|c| {
                let compiler_type = match c.compiler_type.as_str() {
                    "gcc" => CompilerType::Gcc,
                    "clang" => CompilerType::Clang,
                    "msvc" => CompilerType::Msvc,
                    _ => return None,
                };
                Some(CompilerInfo {
                    name: c.name,
                    version: c.version,
                    path: PathBuf::from(c.path),
                    compiler_type,
                })
            })
            .collect();

        Some(compilers)
    }

    pub fn save(compilers: &[CompilerInfo]) -> Result<(), String> {
        let path = Self::cache_path()
            .ok_or_else(|| "Could not determine home directory".to_string())?;

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create cache directory: {}", e))?;
        }

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| format!("System time error: {}", e))?
            .as_secs();

        let cache = CompilerCacheFile {
            timestamp: now,
            compilers: compilers
                .iter()
                .map(|c| CachedCompiler {
                    name: c.name.clone(),
                    version: c.version.clone(),
                    path: c.path.to_string_lossy().to_string(),
                    compiler_type: c.compiler_type.to_string(),
                })
                .collect(),
        };

        let content = toml::to_string_pretty(&cache)
            .map_err(|e| format!("Failed to serialize cache: {}", e))?;

        std::fs::write(&path, content)
            .map_err(|e| format!("Failed to write cache file: {}", e))?;

        Ok(())
    }

    #[allow(dead_code)]
    pub fn invalidate() -> Result<(), String> {
        if let Some(path) = Self::cache_path() {
            if path.exists() {
                std::fs::remove_file(&path)
                    .map_err(|e| format!("Failed to remove cache: {}", e))?;
            }
        }
        Ok(())
    }
}
