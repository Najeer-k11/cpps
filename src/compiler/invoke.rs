use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

use crate::platform::CompilerInfo;

pub struct BuildCommand {
    pub compiler_path: PathBuf,
    #[allow(dead_code)]
    pub standard: String,
    pub flags: Vec<String>,
    pub sources: Vec<PathBuf>,
    pub output: PathBuf,
    pub include_paths: Vec<PathBuf>,
    pub lib_paths: Vec<PathBuf>,
    pub libraries: Vec<String>,
}

pub struct BuildResult {
    pub success: bool,
    pub binary_path: Option<PathBuf>,
    pub binary_size: Option<u64>,
    pub stderr_output: String,
    pub duration: Duration,
}

impl BuildCommand {
    /// Construct for single-file mode
    pub fn single_file(file: &Path, compiler: &CompilerInfo) -> Result<Self, String> {
        let out_dir = file.parent().unwrap_or(Path::new(".")).join("build");
        std::fs::create_dir_all(&out_dir)
            .map_err(|e| format!("Failed to create build directory: {}", e))?;

        let stem = file.file_stem().unwrap_or_default().to_string_lossy();
        let ext = if cfg!(windows) { ".exe" } else { "" };
        let output = out_dir.join(format!("{}{}", stem, ext));

        Ok(Self {
            compiler_path: compiler.path.clone(),
            standard: "-std=c++17".to_string(),
            flags: vec!["-Wall".to_string()],
            sources: vec![file.to_path_buf()],
            output,
            include_paths: Vec::new(),
            lib_paths: Vec::new(),
            libraries: Vec::new(),
        })
    }

    /// Construct from config for project mode
    pub fn from_config(
        project_dir: &Path,
        compiler: &CompilerInfo,
        std_version: &str,
        flags: &[String],
        src_dir: &str,
        out_dir: &str,
        entry: &str,
        release: bool,
        include_paths: Vec<PathBuf>,
        lib_paths: Vec<PathBuf>,
        libraries: Vec<String>,
    ) -> Result<Self, String> {
        let src_path = project_dir.join(src_dir);
        let out_path = project_dir.join(out_dir);
        let entry_path = project_dir.join(entry);

        // Verify src_dir exists
        if !src_path.exists() {
            return Err(format!(
                "Source directory '{}' does not exist",
                src_path.display()
            ));
        }

        // Verify entry file exists
        if !entry_path.exists() {
            return Err(format!(
                "Entry file '{}' does not exist",
                entry_path.display()
            ));
        }

        // Collect all .cpp files from src_dir
        let sources = collect_sources(&src_path)?;
        if sources.is_empty() {
            return Err(format!(
                "No .cpp files found in '{}'",
                src_path.display()
            ));
        }

        // Create output directory
        std::fs::create_dir_all(&out_path)
            .map_err(|e| format!("Failed to create build directory: {}", e))?;

        let project_name = entry_path
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let ext = if cfg!(windows) { ".exe" } else { "" };
        let output = out_path.join(format!("{}{}", project_name, ext));

        // Build flags
        let std_flag = format!("-std={}", std_version);
        let mut all_flags = vec![std_flag];
        if release {
            all_flags.push("-O2".to_string());
            all_flags.push("-DNDEBUG".to_string());
        } else {
            all_flags.push("-g".to_string());
        }
        all_flags.extend(flags.iter().cloned());

        Ok(Self {
            compiler_path: compiler.path.clone(),
            standard: format!("-std={}", std_version),
            flags: all_flags,
            sources,
            output,
            include_paths,
            lib_paths,
            libraries,
        })
    }

    /// Execute the build command
    pub fn execute(&self) -> Result<BuildResult, String> {
        let start = Instant::now();

        let mut cmd = Command::new(&self.compiler_path);

        // Add flags
        for flag in &self.flags {
            cmd.arg(flag);
        }

        // Add include paths
        for path in &self.include_paths {
            cmd.arg(format!("-I{}", path.display()));
        }

        // Add source files
        for source in &self.sources {
            cmd.arg(source);
        }

        // Add output
        cmd.arg("-o");
        cmd.arg(&self.output);

        // Add lib paths
        for path in &self.lib_paths {
            cmd.arg(format!("-L{}", path.display()));
        }

        // Add libraries
        for lib in &self.libraries {
            cmd.arg(format!("-l{}", lib));
        }

        let output = cmd
            .output()
            .map_err(|e| format!("Failed to execute compiler: {}", e))?;

        let duration = start.elapsed();
        let stderr_output = String::from_utf8_lossy(&output.stderr).to_string();

        if output.status.success() {
            let binary_size = std::fs::metadata(&self.output).ok().map(|m| m.len());
            Ok(BuildResult {
                success: true,
                binary_path: Some(self.output.clone()),
                binary_size,
                stderr_output,
                duration,
            })
        } else {
            Ok(BuildResult {
                success: false,
                binary_path: None,
                binary_size: None,
                stderr_output,
                duration,
            })
        }
    }
}

fn collect_sources(dir: &Path) -> Result<Vec<PathBuf>, String> {
    let mut sources = Vec::new();
    collect_sources_recursive(dir, &mut sources)?;
    Ok(sources)
}

fn collect_sources_recursive(dir: &Path, sources: &mut Vec<PathBuf>) -> Result<(), String> {
    let entries = std::fs::read_dir(dir)
        .map_err(|e| format!("Failed to read directory '{}': {}", dir.display(), e))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
        let path = entry.path();
        if path.is_dir() {
            collect_sources_recursive(&path, sources)?;
        } else if let Some(ext) = path.extension() {
            let ext = ext.to_string_lossy().to_lowercase();
            if ext == "cpp" || ext == "cc" || ext == "cxx" {
                sources.push(path);
            }
        }
    }
    Ok(())
}
