use std::path::Path;

use crate::commands::add;
use crate::output::colors::{print_error, print_header, print_info, print_success};
use crate::platform;
use crate::template;

pub fn execute(name: &str, template_name: &str) -> i32 {
    // Validate project name
    if let Err(e) = validate_project_name(name) {
        print_error(&e);
        return 1;
    }

    // Validate template
    if !template::AVAILABLE_TEMPLATES.contains(&template_name) {
        print_error(&format!(
            "Unknown template '{}'. Available templates: {}",
            template_name,
            template::AVAILABLE_TEMPLATES.join(", ")
        ));
        return 1;
    }

    let target_dir = Path::new(name);

    // Check if directory already exists
    if target_dir.exists() {
        print_error(&format!(
            "Directory '{}' already exists. Choose a different name or remove the existing directory.",
            name
        ));
        return 1;
    }

    // Create the project directory
    if let Err(e) = std::fs::create_dir_all(target_dir) {
        print_error(&format!("Failed to create directory '{}': {}", name, e));
        return 1;
    }

    // Scaffold from template
    match template::scaffold(name, template_name, target_dir) {
        Ok(created_files) => {
            // Generate VS Code IntelliSense config
            generate_vscode_config(target_dir);

            print_header(&format!("Created project '{}'", name));

            // Display created files
            for file in &created_files {
                if let Ok(relative) = file.strip_prefix(target_dir) {
                    print_success(&format!("{}", relative.display()));
                }
            }
            print_success(".vscode/c_cpp_properties.json");

            // Auto-install dependencies for templates that need them
            let deps = template_dependencies(template_name);
            if !deps.is_empty() {
                println!();
                print_info("Installing template dependencies...");
                let output_config = crate::output::OutputConfig::detect(false);
                // Change to project dir for cpps add
                let original_dir = std::env::current_dir().ok();
                let _ = std::env::set_current_dir(target_dir);
                for dep in &deps {
                    let result = crate::commands::add::execute(dep, &output_config);
                    if result != 0 {
                        crate::output::colors::print_warning(
                            &format!("Failed to install '{}'. Run `cpps add {}` manually after setup.", dep, dep)
                        );
                    }
                }
                // Restore original dir
                if let Some(dir) = original_dir {
                    let _ = std::env::set_current_dir(dir);
                }
            }

            println!();
            print_info(&format!("cd {}", name));
            print_info("cpps run");

            0
        }
        Err(e) => {
            // Clean up partial directory on failure
            let _ = std::fs::remove_dir_all(target_dir);
            print_error(&format!("Failed to scaffold project: {}", e));
            1
        }
    }
}

fn generate_vscode_config(project_dir: &Path) {
    let vscode_dir = project_dir.join(".vscode");
    let _ = std::fs::create_dir_all(&vscode_dir);

    // Build include paths
    let plat = platform::current_platform();
    let triplet = plat.vcpkg_triplet();

    let mut include_paths = vec![
        "${workspaceFolder}/src".to_string(),
        "${workspaceFolder}/include".to_string(),
    ];

    // Add vcpkg include path if vcpkg is installed
    if let Some(vcpkg_root) = add::get_vcpkg_root() {
        let vcpkg_inc = vcpkg_root.join("installed").join(triplet).join("include");
        include_paths.push(vcpkg_inc.to_string_lossy().to_string());
    }

    let compiler_path = find_compiler_path();

    let cpp_standard = "c++17";
    let intellisense_mode = if cfg!(target_os = "windows") {
        "windows-clang-x64"
    } else if cfg!(target_os = "macos") {
        "macos-clang-arm64"
    } else {
        "linux-gcc-x64"
    };

    let include_paths_json: Vec<String> = include_paths
        .iter()
        .map(|p| format!("                \"{}\"", p.replace('\\', "/")))
        .collect();

    let config = format!(
        r#"{{
    "configurations": [
        {{
            "name": "{}",
            "includePath": [
{}
            ],
            "defines": [],
            "compilerPath": "{}",
            "cStandard": "c17",
            "cppStandard": "{}",
            "intelliSenseMode": "{}"
        }}
    ],
    "version": 4
}}"#,
        if cfg!(target_os = "windows") { "Win32" } else if cfg!(target_os = "macos") { "Mac" } else { "Linux" },
        include_paths_json.join(",\n"),
        compiler_path.replace('\\', "/"),
        cpp_standard,
        intellisense_mode,
    );

    let config_path = vscode_dir.join("c_cpp_properties.json");
    let _ = std::fs::write(&config_path, config);
}

fn find_compiler_path() -> String {
    // Try to find clang++ or g++
    if let Ok(path) = which::which("clang++") {
        return path.to_string_lossy().to_string();
    }

    // Check common locations
    #[cfg(target_os = "windows")]
    {
        let common = std::path::PathBuf::from(r"C:\Program Files\LLVM\bin\clang++.exe");
        if common.exists() {
            return common.to_string_lossy().to_string();
        }
    }

    if let Ok(path) = which::which("g++") {
        return path.to_string_lossy().to_string();
    }

    // Fallback
    "clang++".to_string()
}

fn validate_project_name(name: &str) -> Result<(), String> {
    if name.is_empty() || name.len() > 64 {
        return Err("Project name must be between 1 and 64 characters".to_string());
    }

    if name.starts_with('-') {
        return Err("Project name cannot start with a hyphen".to_string());
    }

    for c in name.chars() {
        if !c.is_alphanumeric() && c != '-' && c != '_' {
            return Err(format!(
                "Project name can only contain alphanumeric characters, hyphens, and underscores (found '{}')",
                c
            ));
        }
    }

    Ok(())
}

/// Returns the list of vcpkg packages needed for a given template
fn template_dependencies(template_name: &str) -> Vec<&'static str> {
    match template_name {
        "sdl" => vec!["sdl2"],
        "raylib" => vec!["raylib"],
        "test" => vec!["doctest"],
        _ => vec![],
    }
}
