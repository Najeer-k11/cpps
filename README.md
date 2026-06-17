# cpps

A cross-platform CLI tool that makes C++ development feel like Cargo or npm.

## Installation

```bash
cargo install --path .
```

## Quick Start

```bash
# Check your environment
cpps doctor

# Create a new project
cpps new hello-world
cd hello-world

# Compile and run
cpps run

# Build without running
cpps build
cpps build --release
```

## Commands

### `cpps doctor`

Checks your C++ development environment for required tools (compilers, cmake, ninja, vcpkg).

```bash
cpps doctor          # Check environment
cpps doctor --fix    # Auto-install missing tools
```

### `cpps new <name>`

Scaffolds a new project from a template.

```bash
cpps new hello-world                # Basic C++ project
cpps new my-game --template sdl     # SDL2 game project
cpps new my-app --template raylib   # Raylib project
cpps new my-lib --template lib      # Static library
cpps new my-tests --template test   # With test framework
```

### `cpps run [file]`

Compiles and immediately runs your code.

```bash
cpps run              # Project mode (reads cpps.toml)
cpps run main.cpp     # Single-file mode (no config needed)
```

### `cpps build [--release]`

Compiles without running. Outputs binary path and size.

```bash
cpps build            # Debug build
cpps build --release  # Optimized build (-O2 -DNDEBUG)
```

### `cpps add <package>`

Adds a dependency via vcpkg.

```bash
cpps add sdl2
cpps add fmt
cpps add boost-filesystem
```

## Configuration

Every project has a `cpps.toml`:

```toml
[project]
name    = "my-project"
version = "0.1.0"
std     = "c++17"

[compiler]
preferred = "auto"         # auto | gcc | clang | msvc
flags     = ["-Wall", "-O2"]

[build]
src_dir = "src"
out_dir = "build"
entry   = "src/main.cpp"

[dependencies]
sdl2 = { version = "2.28", source = "vcpkg" }
```

## Supported Platforms

- Windows (x86_64) — MSVC, GCC, Clang
- macOS (x86_64, aarch64) — Clang, GCC
- Linux (x86_64) — Clang, GCC

## License

MIT
