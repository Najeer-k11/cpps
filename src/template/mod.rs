use rust_embed::Embed;
use std::path::{Path, PathBuf};

#[derive(Embed)]
#[folder = "src/template/templates/"]
struct TemplateAssets;

pub const AVAILABLE_TEMPLATES: &[&str] = &["basic", "sdl", "raylib", "lib", "test", "fmt", "boost-fs"];

/// Scaffold a project from a template
pub fn scaffold(
    project_name: &str,
    template_name: &str,
    target_dir: &Path,
) -> Result<Vec<PathBuf>, String> {
    if !AVAILABLE_TEMPLATES.contains(&template_name) {
        return Err(format!(
            "Unknown template '{}'. Available templates: {}",
            template_name,
            AVAILABLE_TEMPLATES.join(", ")
        ));
    }

    let prefix = format!("{}/", template_name);
    let mut created_files = Vec::new();

    for file_path in TemplateAssets::iter() {
        let file_path_str = file_path.as_ref();
        if !file_path_str.starts_with(&prefix) {
            continue;
        }

        // Get the relative path within the template
        let relative = &file_path_str[prefix.len()..];

        // Apply project name substitution to paths
        let target_relative = relative
            .replace("project.h", &format!("{}.h", project_name))
            .replace("project.cpp", &format!("{}.cpp", project_name));

        // Determine target file path
        // Template files retain their relative structure within the project dir
        // Special cases:
        //   - cpps.toml → project root
        //   - test_main.cpp → tests/
        //   - main.cpp (without subdir) → src/main.cpp
        //   - Everything else keeps its relative path from the template
        let target_file = if relative == "cpps.toml" {
            target_dir.join("cpps.toml")
        } else if relative == "test_main.cpp" {
            target_dir.join("tests").join("test_main.cpp")
        } else if relative == "main.cpp" {
            // Top-level main.cpp in template goes to src/
            target_dir.join("src").join("main.cpp")
        } else {
            // Files with subdirectories (include/, src/) keep their structure
            target_dir.join(&target_relative)
        };

        // Read template content
        let asset = TemplateAssets::get(file_path_str)
            .ok_or_else(|| format!("Template file not found: {}", file_path_str))?;
        let content = String::from_utf8_lossy(&asset.data);

        // Substitute placeholders
        let content = content.replace("{{project_name}}", project_name);

        // Create parent directories
        if let Some(parent) = target_file.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                format!("Failed to create directory '{}': {}", parent.display(), e)
            })?;
        }

        // Write file
        std::fs::write(&target_file, content.as_bytes())
            .map_err(|e| format!("Failed to write '{}': {}", target_file.display(), e))?;

        created_files.push(target_file);
    }

    if created_files.is_empty() {
        return Err(format!("No template files found for '{}'", template_name));
    }

    Ok(created_files)
}
