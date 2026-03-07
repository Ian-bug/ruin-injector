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

# Run all tests
cargo test

# Run single test
cargo test test_name

# Tests with output
cargo test -- --nocapture
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

### Windows API
- All calls in `unsafe` blocks
- Handle `Result` types properly
- Always `CloseHandle()` when done
- Always `VirtualFreeEx()` when done
- Use `GetProcAddress` for addresses
- Check `.is_invalid()` on handles
- Get error: `unsafe { GetLastError() }`

### GUI/egui
- State in app struct
- Implement `Default` trait
- Implement `eframe::App`
- `egui::CentralPanel` for content
- `egui::Window` for popups
- `egui::ScrollArea` for scroll
- `ctx.request_repaint()` at end

### Unsafe Code
- Comment why unsafe needed
- Keep isolated and minimal
- Prefer safe abstractions
- Validate pointers before use

### Configuration
- Use `serde` for serialization
- Store in user config via `dirs`
- JSON via `serde_json`
- Sensible defaults

## Project Structure

```
rust-injector/
├── src/
│   ├── main.rs      # Entry, egui UI
│   ├── injector.rs  # Core injection, Windows API
│   ├── uwp.rs       # UWP permissions
│   └── config.rs   # Config persistence
├── Cargo.toml
├── build.rs
├── icon.ico
└── AGENTS.md
```

## Windows-Specific

### Process Discovery
- `CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS)`
- `Process32First` + `Process32Next` iteration
- UTF-16 conversion for names

### DLL Injection Steps
1. Get process ID by name
2. Open with `PROCESS_ALL_ACCESS`
3. `VirtualAllocEx` to allocate memory
4. `WriteProcessMemory` for DLL path
5. `CreateRemoteThread` with `LoadLibraryW`
6. `WaitForSingleObject` for completion
7. Cleanup: free memory, close handles

### UWP
- Lower privileges
- Manual DLL permission setup
- SID: `S-1-15-2-1`
- Admin privileges required

## Testing

No automated tests currently. When adding:
1. Unit tests in same file
2. Use `#[cfg(test)]` attribute
3. Mock Windows API calls
4. Integration tests on test processes

## Common Pitfalls

- **Borrow checker**: Use index-based iteration in egui UI
- **Handle leaks**: Always close handles; consider RAII
- **Null pointers**: Check `is_null()` before dereference
- **Result types**: `windows` crate returns `Result`, not raw
- **UTF-16**: Windows uses UTF-16; convert properly
