# cpps

A cross-platform CLI tool that makes C++ development feel like Cargo or npm.

cpps handles environment setup, project scaffolding, compilation, execution, and dependency management across Windows, macOS, and Linux — all from a single binary.

## Installation

### Windows (MSI Installer)

Download the latest `.msi` from [Releases](https://github.com/user/cpps/releases). Installs and adds to PATH automatically.

### From Source

```bash
cargo install --path .
```

### PowerShell Install Script

```powershell
powershell -ExecutionPolicy Bypass -File install.ps1
```

## Quick Start

```bash
# 1. Set up your C++ environment (installs compilers, cmake, ninja, vcpkg)
cpps doctor --fix

# 2. Create a project
cpps new hello-world
cd hello-world

# 3. Run it
cpps run
```

## Commands

### `cpps doctor [--fix]`

Checks your C++ development environment for compilers, build tools, and package managers.

```bash
cpps doctor          # Check environment — shows ✓/✗/⚠ for each tool
cpps doctor --fix    # Auto-install missing tools with progress bars
```

**Detected tools:** clang++, g++, MSVC (cl.exe), cmake, ninja, vcpkg

**`--fix` behavior by platform:**
| Platform | Package Manager |
|----------|----------------|
| Windows | winget (+ vcpkg via git clone) |
| macOS | brew |
| Linux | apt / dnf / pacman (auto-detected) |

---

### `cpps new <name> [--template <name>]`

Creates a new project with source files, `cpps.toml`, and VS Code IntelliSense config.

```bash
cpps new hello-world                # Basic hello world
cpps new my-game --template sdl     # SDL2 window + event loop
cpps new my-app --template raylib   # Raylib window + draw loop
cpps new my-lib --template lib      # Static library (include/ + src/)
cpps new my-tests --template test   # With doctest testing framework
```

**Auto-generates:** `cpps.toml`, `src/main.cpp`, `.vscode/c_cpp_properties.json`

**Project name rules:** alphanumeric, hyphens, underscores only; 1–64 chars; can't start with hyphen.

---

### `cpps run [file]`

Compiles and immediately runs your code.

```bash
cpps run              # Project mode — reads cpps.toml, compiles all src/
cpps run main.cpp     # Single-file mode — no config needed, uses C++17
cpps run test.cc      # Supports .cpp, .cc, .cxx extensions
```

**Project mode:** reads `cpps.toml` for compiler, flags, std, and dependencies.  
**Single-file mode:** compiles with highest-ranked compiler, C++17, `-Wall`.

---

### `cpps build [--release]`

Compiles without running. Reports binary path and size.

```bash
cpps build            # Debug build (with debug symbols)
cpps build --release  # Optimized: -O2 -DNDEBUG (smaller, faster binary)
```

---

### `cpps add <package>`

Installs a C++ library via vcpkg and updates `cpps.toml`.

```bash
cpps add sdl2                 # SDL2 multimedia library
cpps add fmt                  # Modern formatting library
cpps add boost-filesystem     # Boost.Filesystem
cpps add raylib               # Raylib game library
cpps add imgui                # Dear ImGui
```

After adding, `#include` the library headers in your code and `cpps run` — include paths are resolved automatically.

---

### `cpps uninstall [--force]`

Removes cpps and all tools installed by `cpps doctor --fix`.

```bash
cpps uninstall         # Prompts for confirmation
cpps uninstall --force # No prompt, removes everything
```

**Removes:** LLVM/clang++, ninja (via winget), vcpkg folder, `~/.cpps` cache, PATH entries.

---

## Configuration (`cpps.toml`)

Every project has a `cpps.toml` at the root:

```toml
[project]
name    = "my-project"
version = "0.1.0"
std     = "c++17"              # c++11 | c++14 | c++17 | c++20 | c++23

[compiler]
preferred = "auto"             # auto | gcc | clang | msvc
flags     = ["-Wall", "-O2"]

[build]
src_dir = "src"
out_dir = "build"
entry   = "src/main.cpp"

[dependencies]
sdl2 = { version = "2.28", source = "vcpkg" }
fmt  = { version = "10.1", source = "vcpkg" }
```

## Supported Platforms

| Platform           | Compilers        | Package Manager    | Binary |
| ------------------ | ---------------- | ------------------ | ------ |
| Windows x86_64     | MSVC, Clang, GCC | winget / choco     | `.exe` |
| macOS x86_64/arm64 | Clang, GCC       | brew               | (none) |
| Linux x86_64       | Clang, GCC       | apt / dnf / pacman | (none) |

## VS Code Integration

`cpps new` auto-generates `.vscode/c_cpp_properties.json` with:

- Your detected compiler path
- Project `src/` and `include/` directories
- vcpkg installed headers path

This eliminates IntelliSense squiggles for library includes.

## Global Flags

| Flag               | Description                                    |
| ------------------ | ---------------------------------------------- |
| `--no-color`       | Suppress colored output (useful for CI/piping) |
| `--help` / `-h`    | Show help (use with any command for details)   |
| `--version` / `-V` | Show version                                   |

## License

MIT
