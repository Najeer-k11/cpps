use crate::platform::{self, CompilerInfo, CompilerType, Platform};

use super::cache::CompilerCache;

pub struct CompilerDetector {
    platform: Box<dyn Platform>,
}

impl CompilerDetector {
    pub fn new() -> Self {
        Self {
            platform: platform::current_platform(),
        }
    }

    /// Detect all available compilers, using cache if valid
    pub fn detect_all(&self) -> Vec<CompilerInfo> {
        if let Some(cached) = CompilerCache::load_if_valid() {
            return cached;
        }

        let compilers = self.platform.find_compilers();

        // Save to cache (ignore errors)
        let _ = CompilerCache::save(&compilers);

        compilers
    }

    /// Select the best compiler based on config preference and platform defaults
    pub fn select_compiler<'a>(
        &self,
        compilers: &'a [CompilerInfo],
        preferred: &str,
    ) -> Result<&'a CompilerInfo, String> {
        if compilers.is_empty() {
            return Err(
                "No C++ compiler found. Run `cpps doctor --fix` to install one.".to_string()
            );
        }

        // If user specified a preference (not "auto"), try to find it
        if preferred != "auto" {
            let target_type = match preferred {
                "gcc" => CompilerType::Gcc,
                "clang" => CompilerType::Clang,
                "msvc" => CompilerType::Msvc,
                _ => {
                    return Err(format!("Unknown compiler preference: {}", preferred));
                }
            };

            if let Some(compiler) = compilers.iter().find(|c| c.compiler_type == target_type) {
                return Ok(compiler);
            }

            // Preferred not found — warn and fall back
            eprintln!(
                "  ⚠ Preferred compiler '{}' not found, falling back to default",
                preferred
            );
        }

        // Use platform-specific ranking
        let ranking = self.platform.default_compiler_ranking();
        for desired_type in &ranking {
            if let Some(compiler) = compilers.iter().find(|c| &c.compiler_type == desired_type) {
                return Ok(compiler);
            }
        }

        // Just return the first available
        Ok(&compilers[0])
    }

    #[allow(dead_code)]
    pub fn platform(&self) -> &dyn Platform {
        self.platform.as_ref()
    }
}
