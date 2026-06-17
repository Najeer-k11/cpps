use std::path::Path;

use crate::output::colors::{print_error, print_header, print_info, print_success};
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
            print_header(&format!("Created project '{}'", name));

            // Display created files
            for file in &created_files {
                if let Ok(relative) = file.strip_prefix(target_dir) {
                    print_success(&format!("{}", relative.display()));
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
