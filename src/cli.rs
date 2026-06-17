use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "cpps", version, about = "C++ made simple — a cross-platform CLI for C++ development")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Suppress color output
    #[arg(long, global = true)]
    pub no_color: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Check your C++ development environment
    Doctor {
        /// Auto-install missing tools
        #[arg(long)]
        fix: bool,
    },
    /// Create a new C++ project
    New {
        /// Project name
        name: String,
        /// Project template (basic, sdl, raylib, lib, test)
        #[arg(long, default_value = "basic")]
        template: String,
    },
    /// Compile and run
    Run {
        /// Source file (single-file mode) or omit for project mode
        file: Option<String>,
    },
    /// Compile without running
    Build {
        /// Build with optimizations (-O2 -DNDEBUG)
        #[arg(long)]
        release: bool,
    },
    /// Add a dependency via vcpkg
    Add {
        /// Package name
        package: String,
    },
    /// Uninstall cpps and all tools installed by cpps doctor --fix
    Uninstall {
        /// Remove everything without confirmation
        #[arg(long)]
        force: bool,
    },
}
