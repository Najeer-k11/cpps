use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "cpps",
    version,
    about = "C++ made simple — a cross-platform CLI for C++ development",
    long_about = "cpps is a cross-platform CLI tool that makes C++ development feel like Cargo or npm.\n\
        It handles environment setup, project scaffolding, compilation, execution,\n\
        and dependency management across Windows, macOS, and Linux.\n\n\
        QUICK START:\n\
        \x20 cpps doctor --fix    Install compilers and tools automatically\n\
        \x20 cpps new my-app      Create a new C++ project\n\
        \x20 cpps run             Compile and run your project\n\
        \x20 cpps build --release Build an optimized binary\n\
        \x20 cpps add sdl2        Add a library dependency\n\n\
        EXAMPLES:\n\
        \x20 cpps run main.cpp           Single-file mode (no config needed)\n\
        \x20 cpps new game --template sdl Create an SDL2 game project\n\
        \x20 cpps doctor                  Check what tools are installed",
    after_help = "See 'cpps <command> --help' for detailed info on each command."
)]
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
    #[command(
        long_about = "Scans your system for C++ compilers (clang++, g++, MSVC), build tools\n\
            (cmake, ninja), and package managers (vcpkg). Reports version and path for\n\
            each tool found, and suggests install commands for missing ones.\n\n\
            EXAMPLES:\n\
            \x20 cpps doctor          Check environment status\n\
            \x20 cpps doctor --fix    Auto-install all missing tools"
    )]
    Doctor {
        /// Auto-install missing tools (uses winget/brew/apt depending on OS)
        #[arg(long, help = "Auto-install missing tools with progress bars")]
        fix: bool,
    },

    /// Create a new C++ project from a template
    #[command(
        long_about = "Scaffolds a new C++ project directory with cpps.toml config, source files,\n\
            and VS Code IntelliSense configuration. Templates provide ready-to-run code.\n\n\
            TEMPLATES:\n\
            \x20 basic   — Hello world with iostream (default)\n\
            \x20 sdl     — SDL2 window with event loop and input handling\n\
            \x20 raylib  — Raylib window with draw loop\n\
            \x20 lib     — Static library layout with include/ and src/\n\
            \x20 test    — Project with doctest testing framework\n\n\
            EXAMPLES:\n\
            \x20 cpps new hello-world              Basic project\n\
            \x20 cpps new my-game --template sdl   SDL2 game\n\
            \x20 cpps new engine --template raylib  Raylib project\n\
            \x20 cpps new mylib --template lib      Library project\n\
            \x20 cpps new myapp --template test     With tests"
    )]
    New {
        /// Project name (alphanumeric, hyphens, underscores; max 64 chars)
        name: String,
        /// Project template to use
        #[arg(long, default_value = "basic", value_parser = ["basic", "sdl", "raylib", "lib", "test"])]
        template: String,
    },

    /// Compile and run your C++ code
    #[command(
        long_about = "Compiles and immediately executes your code. Works in two modes:\n\n\
            PROJECT MODE (no file argument):\n\
            \x20 Reads cpps.toml, compiles all sources in src_dir, runs the binary.\n\
            \x20 Uses configured compiler, flags, and C++ standard.\n\n\
            SINGLE-FILE MODE (with file argument):\n\
            \x20 Compiles just that file with C++17 and runs it immediately.\n\
            \x20 No cpps.toml needed — great for quick experiments.\n\n\
            EXAMPLES:\n\
            \x20 cpps run              Compile & run project (reads cpps.toml)\n\
            \x20 cpps run main.cpp     Compile & run a single file\n\
            \x20 cpps run test.cc      Supports .cpp, .cc, .cxx extensions"
    )]
    Run {
        /// Source file for single-file mode (omit for project mode)
        file: Option<String>,
    },

    /// Compile your project without running it
    #[command(
        long_about = "Compiles the project and reports the output binary path and size.\n\
            Requires cpps.toml in the current directory.\n\n\
            BUILD MODES:\n\
            \x20 Debug (default) — includes debug symbols, no optimization\n\
            \x20 Release         — optimized with -O2 -DNDEBUG, stripped\n\n\
            EXAMPLES:\n\
            \x20 cpps build            Debug build\n\
            \x20 cpps build --release  Optimized release build"
    )]
    Build {
        /// Build with optimizations (-O2 -DNDEBUG)
        #[arg(long, help = "Enable release mode: -O2 -DNDEBUG optimizations")]
        release: bool,
    },

    /// Add a dependency via vcpkg
    #[command(
        long_about = "Installs a C++ library via vcpkg and adds it to your cpps.toml.\n\
            The library's include and lib paths are automatically resolved during builds.\n\n\
            After adding a dependency, #include its headers in your code and rebuild.\n\
            vcpkg must be installed (run 'cpps doctor --fix' if missing).\n\n\
            EXAMPLES:\n\
            \x20 cpps add sdl2                 Add SDL2 library\n\
            \x20 cpps add fmt                  Add the fmt formatting library\n\
            \x20 cpps add boost-filesystem     Add Boost.Filesystem\n\
            \x20 cpps add raylib               Add Raylib game library\n\
            \x20 cpps add imgui                Add Dear ImGui"
    )]
    Add {
        /// Package name (as listed in vcpkg registry)
        package: String,
    },

    /// Uninstall cpps and all tools installed by 'cpps doctor --fix'
    #[command(
        long_about = "Removes cpps, its cache (~/.cpps), and tools that were installed by\n\
            'cpps doctor --fix'. Cleans up directories and removes PATH entries.\n\n\
            On Windows: uninstalls LLVM, Ninja via winget; removes vcpkg folder;\n\
            \x20           cleans user PATH environment variable.\n\
            On macOS:   uninstalls via brew; removes vcpkg folder.\n\
            On Linux:   removes via apt/dnf/pacman; removes vcpkg folder.\n\n\
            EXAMPLES:\n\
            \x20 cpps uninstall          Prompts for confirmation before removing\n\
            \x20 cpps uninstall --force  Remove everything without asking"
    )]
    Uninstall {
        /// Remove everything without confirmation prompt
        #[arg(long, help = "Skip confirmation prompt and remove everything")]
        force: bool,
    },
}
