# Ruin DLL Injector - Agent Guidelines

Guidelines for agentic coding assistants working on this repository.

## Build, Lint, and Test Commands

```bash
# Release build (optimized)
cargo build --release

# Dev build (faster)
cargo build

# Clean + rebuild
cargo clean && cargo build --release

# Fast compile check
cargo check

# Lint
cargo clippy

# Format
cargo fmt

# Check formatting without changes
cargo fmt --check

# Run all tests
cargo test

# Run single test
cargo test test_name

# Tests with output
cargo test -- --nocapture

# Auto-fix clippy warnings
cargo clippy --fix --allow-dirty --allow-staged
```

## Code Style

### Imports
- Group: std → external → internal modules
- Sort alphabetically
- Specific imports over glob
- `#![windows_subsystem = "windows"]` at top of main.rs

### Formatting
- 4-space indentation
- Max line: 100 chars (soft)
- Opening braces same line
- Trailing commas in multi-line

### Naming
- **Structs/Enums**: PascalCase (`InjectorApp`, `InjectionError`)
- **Functions**: snake_case (`inject_dll`, `get_process_id`)
- **Private fields**: trailing underscore (`config_path`)
- **Constants**: UPPER_SNAKE_CASE (`MAX_PROCESS_NAME_LENGTH`)

### Types
- `Option<T>` for nullable (`Option<PathBuf>`)
- `Result<T, E>` for fallible ops
- `String` over `&str` when owned
- `Arc<T>` for thread-safe (`Arc<Injector>`)
- `Vec<T>` for dynamic (`Vec<String>`)

### Error Handling
- Custom errors as enums
- Implement `Display` + `Error` traits
- Use `match` over `unwrap()`
- Descriptive error messages
- Log via `add_log()` method
- Clean up resources before returning errors

### Windows API
- All calls in `unsafe` blocks
- Handle `Result` types properly
- Always `CloseHandle()` when done
- Always `VirtualFreeEx()` when done
- Use `GetProcAddress` for addresses
- Check `.is_invalid()` on handles
- Get error: `unsafe { GetLastError() }`
- Check admin privileges: `OpenProcessToken()` + `GetTokenInformation()`

### GUI/egui
- State in app struct
- Implement `Default` trait
- Implement `eframe::App`
- `egui::CentralPanel` for content
- `egui::Window` for popups
- `egui::ScrollArea` for scroll
- `ctx.request_repaint()` at end
- Use index-based iteration to avoid borrow checker issues

### Unsafe Code
- Comment why unsafe needed
- Keep isolated and minimal
- Prefer safe abstractions
- Validate pointers before use
- Use type aliases for function pointers (e.g., `LoadLibraryWFn`)

### Constants
- Replace magic numbers with named constants
- Define at module or crate level
- Use descriptive names (`NEW_LOG_DURATION_FRAMES`, `MAX_PROCESS_NAME_LENGTH`)

### Configuration
- Use `serde` for serialization with `#[derive(Default)]`
- Store in user config via `dirs`
- JSON via `serde_json`
- Sensible defaults in `Default` impl

## Project Structure

```
rust-injector/
├── src/
│   ├── main.rs      # Entry, egui UI, auto-inject logic
│   ├── injector.rs  # Core injection, Windows API, admin checks
│   └── config.rs   # Config persistence
├── Cargo.toml
├── build.rs
├── icon.ico
├── AGENTS.md
└── .github/workflows/
    ├── ci.yml          # CI testing on push/PR
    ├── release.yml     # Automated releases on tags
    └── README.md       # Workflow documentation
```

## Windows-Specific

### Process Discovery
- `CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS)`
- `Process32First` + `Process32Next` iteration
- UTF-16 conversion for names

### DLL Injection Steps
1. Validate inputs (process name, DLL path, admin privileges)
2. Get process ID by name
3. Open with `PROCESS_ALL_ACCESS`
4. `VirtualAllocEx` to allocate memory
5. `WriteProcessMemory` for DLL path (UTF-16)
6. `GetModuleHandleA()` + `GetProcAddress()` to find `LoadLibraryW`
7. `CreateRemoteThread` with transmuted function pointer
8. `WaitForSingleObject` for completion
9. Cleanup: free memory with `VirtualFreeEx()`, close handles with `CloseHandle()`
10. Admin check: `OpenProcessToken()` + `GetTokenInformation()`

## Testing

Comprehensive test suite (13 tests):
- Unit tests in same file with `#[cfg(test)]`
- Test process enumeration, admin checks, validation
- Test both elevated and non-elevated scenarios

## Features

### Auto-Inject
- Checkbox in UI, persists to config
- Checks if target process is running
- Auto-injects when process detected
- Tracks injection state (`auto_injected` flag)

### Admin Verification
- Checks elevation before injection attempt
- Returns `NotElevated` error if not admin

## Common Pitfalls

- **Borrow checker**: Use index-based iteration in egui UI
- **Handle leaks**: Always close handles; RAII guards available for future use
- **Null pointers**: Check `is_null()` before dereference
- **Result types**: `windows` crate returns `Result`, not raw
- **UTF-16**: Windows uses UTF-16; convert properly with `encode_utf16()`
- **Function pointers**: Use type aliases and transmute only when necessary
- **Auto-inject**: Remember to reset `auto_injected` flag when process restarts
- **Config loading**: Handle missing config files gracefully with `Config::default()`

## Development Workflow

1. Make changes and ensure tests pass: `cargo test`
2. Check formatting: `cargo fmt`
3. Run linter: `cargo clippy` (should be clean)
4. Build release: `cargo build --release`
5. To create new release:
   - Update version in `Cargo.toml`
   - Commit and push
   - Create tag: `git tag v1.1.4 && git push origin v1.1.4`
   - GitHub Actions automatically builds and uploads release

## CI/CD

- CI runs on every push/PR: tests, clippy, fmt check
- Release runs on version tags: builds binary and uploads to GitHub Releases
- Workflows in `.github/workflows/`

