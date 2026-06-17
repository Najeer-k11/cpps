use std::io::Write;
use std::path::PathBuf;
use std::process::Command;

use crate::output::colors::{print_error, print_header, print_info, print_success, print_warning};

pub fn execute(force: bool) -> i32 {
    print_header("Uninstalling cpps and tools...");

    if !force {
        eprint!("  This will remove cpps, its cache, and tools installed by `cpps doctor --fix`.\n  Continue? [y/N] ");
        std::io::stderr().flush().ok();

        let mut input = String::new();
        if std::io::stdin().read_line(&mut input).is_err() {
            print_error("Failed to read input");
            return 1;
        }
        if !input.trim().eq_ignore_ascii_case("y") {
            print_info("Cancelled.");
            return 0;
        }
    }

    let mut errors = 0;

    // 1. Remove cpps cache directory (~/.cpps)
    if let Some(home) = dirs::home_dir() {
        let cpps_dir = home.join(".cpps");
        if cpps_dir.exists() {
            match std::fs::remove_dir_all(&cpps_dir) {
                Ok(_) => print_success(&format!("Removed {}", cpps_dir.display())),
                Err(e) => {
                    print_error(&format!("Failed to remove {}: {}", cpps_dir.display(), e));
                    errors += 1;
                }
            }
        }
    }

    // 2. Platform-specific uninstalls
    #[cfg(target_os = "windows")]
    {
        errors += uninstall_windows();
    }
    #[cfg(target_os = "macos")]
    {
        errors += uninstall_macos();
    }
    #[cfg(target_os = "linux")]
    {
        errors += uninstall_linux();
    }

    println!();
    if errors == 0 {
        print_success("Uninstall complete. You may need to restart your terminal.");
    } else {
        print_warning(&format!("Uninstall finished with {} error(s).", errors));
    }

    errors as i32
}

#[cfg(target_os = "windows")]
fn uninstall_windows() -> u32 {
    let mut errors = 0;

    // Remove vcpkg directory
    let home = std::env::var("USERPROFILE").unwrap_or_default();
    let vcpkg_dir = PathBuf::from(&home).join("vcpkg");
    if vcpkg_dir.exists() {
        print_info("Removing vcpkg...");
        match std::fs::remove_dir_all(&vcpkg_dir) {
            Ok(_) => print_success(&format!("Removed {}", vcpkg_dir.display())),
            Err(e) => {
                print_error(&format!("Failed to remove {}: {}", vcpkg_dir.display(), e));
                errors += 1;
            }
        }
    }

    // Uninstall LLVM via winget
    print_info("Uninstalling LLVM (clang++)...");
    let result = Command::new("winget")
        .args(["uninstall", "LLVM.LLVM", "--silent", "--accept-source-agreements"])
        .output();
    match result {
        Ok(out) if out.status.success() => print_success("LLVM uninstalled"),
        Ok(out) => {
            let msg = String::from_utf8_lossy(&out.stdout);
            if msg.contains("No installed package") {
                print_info("LLVM was not installed via winget, skipping");
            } else {
                print_warning("Could not uninstall LLVM — may need manual removal");
            }
        }
        Err(_) => print_warning("winget not available — skip LLVM uninstall"),
    }

    // Uninstall Ninja via winget
    print_info("Uninstalling Ninja...");
    let result = Command::new("winget")
        .args(["uninstall", "Ninja-build.Ninja", "--silent", "--accept-source-agreements"])
        .output();
    match result {
        Ok(out) if out.status.success() => print_success("Ninja uninstalled"),
        Ok(_) => print_info("Ninja was not installed via winget, skipping"),
        Err(_) => {}
    }

    // Remove cpps binary directory and clean PATH
    let cpps_bin = PathBuf::from(&home).join(".cpps").join("bin");
    if cpps_bin.exists() {
        let _ = std::fs::remove_dir_all(&cpps_bin);
    }

    // Clean PATH environment variable
    print_info("Cleaning PATH...");
    let paths_to_remove = vec![
        format!("{}\\.cpps\\bin", home),
        format!("{}\\vcpkg", home),
        r"C:\Program Files\LLVM\bin".to_string(),
    ];
    remove_from_user_path(&paths_to_remove);
    print_success("PATH cleaned");

    errors
}

#[cfg(target_os = "windows")]
fn remove_from_user_path(paths_to_remove: &[String]) {
    // Read current user PATH from registry
    let output = Command::new("powershell")
        .args(["-Command", "[Environment]::GetEnvironmentVariable('Path', 'User')"])
        .output();

    if let Ok(out) = output {
        let current_path = String::from_utf8_lossy(&out.stdout).trim().to_string();
        let new_parts: Vec<&str> = current_path
            .split(';')
            .filter(|p| {
                let p_lower = p.to_lowercase();
                !paths_to_remove.iter().any(|r| p_lower == r.to_lowercase())
            })
            .filter(|p| !p.is_empty())
            .collect();
        let new_path = new_parts.join(";");

        let _ = Command::new("powershell")
            .args([
                "-Command",
                &format!(
                    "[Environment]::SetEnvironmentVariable('Path', '{}', 'User')",
                    new_path.replace('\'', "''")
                ),
            ])
            .output();
    }
}

#[cfg(target_os = "macos")]
fn uninstall_macos() -> u32 {
    let mut errors = 0;

    // Remove vcpkg if installed in home
    if let Some(home) = dirs::home_dir() {
        let vcpkg_dir = home.join("vcpkg");
        if vcpkg_dir.exists() {
            print_info("Removing vcpkg...");
            match std::fs::remove_dir_all(&vcpkg_dir) {
                Ok(_) => print_success("Removed vcpkg"),
                Err(e) => {
                    print_error(&format!("Failed: {}", e));
                    errors += 1;
                }
            }
        }
    }

    // Uninstall via brew
    let brew_packages = ["llvm", "gcc", "cmake", "ninja", "vcpkg"];
    for pkg in brew_packages {
        print_info(&format!("Uninstalling {} via brew...", pkg));
        let result = Command::new("brew")
            .args(["uninstall", "--force", pkg])
            .output();
        match result {
            Ok(out) if out.status.success() => print_success(&format!("{} uninstalled", pkg)),
            Ok(_) => print_info(&format!("{} was not installed via brew", pkg)),
            Err(_) => {}
        }
    }

    // Clean shell profile
    print_info("Note: manually remove cpps PATH entries from ~/.zshrc or ~/.bash_profile if added");

    errors
}

#[cfg(target_os = "linux")]
fn uninstall_linux() -> u32 {
    let mut errors = 0;

    // Remove vcpkg if installed in home
    if let Some(home) = dirs::home_dir() {
        let vcpkg_dir = home.join("vcpkg");
        if vcpkg_dir.exists() {
            print_info("Removing vcpkg...");
            match std::fs::remove_dir_all(&vcpkg_dir) {
                Ok(_) => print_success("Removed vcpkg"),
                Err(e) => {
                    print_error(&format!("Failed: {}", e));
                    errors += 1;
                }
            }
        }

        // Remove cpps bin
        let cpps_bin = home.join(".cpps").join("bin");
        if cpps_bin.exists() {
            let _ = std::fs::remove_dir_all(&cpps_bin);
        }
        let local_bin = home.join(".local").join("bin").join("cpps");
        if local_bin.exists() {
            let _ = std::fs::remove_file(&local_bin);
        }
    }

    // Detect package manager and uninstall
    if which::which("apt").is_ok() {
        let packages = ["g++", "clang", "cmake", "ninja-build"];
        for pkg in packages {
            print_info(&format!("Removing {} via apt...", pkg));
            let _ = Command::new("sudo")
                .args(["apt", "remove", "-y", pkg])
                .output();
        }
    } else if which::which("dnf").is_ok() {
        let packages = ["gcc-c++", "clang", "cmake", "ninja-build"];
        for pkg in packages {
            print_info(&format!("Removing {} via dnf...", pkg));
            let _ = Command::new("sudo")
                .args(["dnf", "remove", "-y", pkg])
                .output();
        }
    } else if which::which("pacman").is_ok() {
        let packages = ["gcc", "clang", "cmake", "ninja"];
        for pkg in packages {
            print_info(&format!("Removing {} via pacman...", pkg));
            let _ = Command::new("sudo")
                .args(["pacman", "-Rns", "--noconfirm", pkg])
                .output();
        }
    }

    print_info("Note: manually remove cpps PATH entries from ~/.bashrc or ~/.profile if added");

    errors
}
