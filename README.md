# Ruin DLL Injector

A modern, lightweight DLL injector built with Rust and egui, designed for Windows applications. Inspired by [FateInjector](https://github.com/fligger/FateInjector).

![Screenshot](screenshot.png)

## Features

- **Modern GUI**: Clean, responsive interface built with egui with smooth animations
- **Animated Transitions**: Fade-in, slide-in, and window scaling animations
- **Process Browser**: Visual selection from running processes with search functionality
- **Auto Configuration**: Saves your settings (DLL path, process preferences) between sessions
- **Auto Inject**: Automatically inject when target process is detected
- **Injection History**: Tracks last 10 injections with timestamps
- **Real-time Logging**: Monitor injection status with animated log entries
- **No Console**: Pure GUI application - no black terminal windows
- **Lightweight**: ~4.5 MB executable with minimal dependencies

## System Requirements

- Windows 10/11 (64-bit)
- Administrator privileges (required for some processes, not all)
- The DLL file you want to inject

## Quick Start

### Building

```bash
cargo build --release
```

The compiled executable will be at `target/release/ruin-injector.exe`.

### Adding an Icon

Place your `.ico` file as `icon.ico` in the project root directory and rebuild. See [ICON.md](ICON.md) for detailed instructions.

## Usage

1. **Run Application**
    ```bash
    .\ruin-injector.exe
    ```
    (Right-click -> "Run as administrator" for maximum compatibility)

2. **Select DLL File**
    - Click "Browse" button
    - Navigate to and select your DLL file

3. **Choose Target Process**
    - Click "List" button
    - Use search box to filter processes
    - Click on desired process

4. **Inject**
    - Click "Inject DLL" button
    - Monitor log section for success/error messages

### Features in Detail

#### Process Selection
- **Live Process List**: Shows all currently running processes
- **Search/Filter**: Type to quickly find specific processes
- **Process Info**: Displays process name and PID (Process ID)
- **Animated Window**: Process list opens with smooth scale animation

#### Injection Options
- **Auto Inject**: Automatically inject when target process is detected
  - Enable via checkbox in UI
  - Settings persist across sessions
  - Detects process start and injects automatically
  - Shows "Active" indicator when enabled
- **Manual Inject**: Click Inject button for immediate injection

#### Visual Enhancements
- **Fade-in Animation**: Title fades in smoothly on startup
- **Slide Animation**: Content slides in from below
- **Log Animation**: New log entries fade in as they're added
- **Window Scaling**: Process selector window scales in/out smoothly

## Architecture

```
rust-injector/
├── src/
│   ├── main.rs          # Application entry, egui UI, animations, state management
│   ├── injector.rs      # Core injection logic, Windows API calls
│   └── config.rs       # Configuration persistence (JSON)
├── Cargo.toml           # Project dependencies and metadata
├── build.rs             # Windows resource compilation (icon embedding)
├── icon.ico             # Application icon (optional, auto-embedded)
├── README.md            # This file
├── README_CN.md        # Chinese documentation
├── ICON.md             # Icon usage instructions
└── AGENTS.md           # Guidelines for AI coding assistants
```

## Technical Implementation

### DLL Injection Process

1. **Process Discovery**
   - Uses `CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS)` to enumerate processes
   - Iterates with `Process32First` and `Process32Next`

2. **Process Access**
   - Opens target process with `OpenProcess(PROCESS_ALL_ACCESS, ...)`
   - Handles Windows permission models
   - Injection will fail gracefully if insufficient permissions

3. **Memory Allocation**
   - Allocates memory in target process via `VirtualAllocEx(...)`
   - Sets memory protection to `PAGE_READWRITE`

4. **DLL Path Injection**
   - Writes DLL path string to allocated memory using `WriteProcessMemory(...)`
   - Converts path to UTF-16 wide string format

5. **Remote Thread Creation**
   - Creates a thread in target process via `CreateRemoteThread(...)`
   - Thread entry point is `LoadLibraryW` (obtained via `GetProcAddress`)

6. **Cleanup**
   - Waits for thread completion with `WaitForSingleObject(...)`
   - Frees allocated memory with `VirtualFreeEx(...)`
   - Closes all handles properly

### Core Windows APIs Used

| API | Purpose |
|-----|---------|
| `CreateToolhelp32Snapshot` | Create process snapshot |
| `Process32First/Next` | Enumerate processes |
| `OpenProcess` | Access target process |
| `VirtualAllocEx` | Allocate memory in target |
| `WriteProcessMemory` | Write DLL path to target |
| `GetProcAddress` | Get function address |
| `CreateRemoteThread` | Create remote execution thread |
| `LoadLibraryW` | Load DLL in target process |
| `CloseHandle` | Release resources |

### Animation System

The application uses a custom animation system:

- **Linear Interpolation**: `lerp()` function for smooth transitions
- **Fade Animation**: Alpha value interpolation for UI elements
- **Slide Animation**: Y-offset interpolation for panel movement
- **Scale Animation**: Window scale interpolation for dialogs
- **Log Alpha**: Per-log entry transparency for smooth fade-in

**Animation Constants**:
```rust
const ANIMATION_SPEED: f32 = 0.15;  // 15% approach per frame
const NEW_LOG_DURATION_FRAMES: usize = 120;  // 2 seconds at 60fps
```

### Error Handling

All operations include comprehensive error handling:

```rust
pub enum InjectionError {
    ProcessNotFound(String),
    OpenProcessFailed(String),
    MemoryAllocationFailed(String),
    WriteMemoryFailed(String),
    CreateRemoteThreadFailed(String),
    InvalidPath(String),
    InvalidProcessName(String),
}
```

Errors are displayed in UI log with descriptive context messages. Proper resource cleanup is performed in all error paths.

**Note**: Administrator check removed - injection now attempts with current permissions and fails gracefully if elevation is required.

## Development

### Build Commands

```bash
# Release build (optimized)
cargo build --release

# Development build (faster)
cargo build

# Clean and rebuild
cargo clean && cargo build --release

# Check compilation (fast)
cargo check
```

### Linting and Formatting

```bash
# Run clippy linter
cargo clippy

# Format code with rustfmt
cargo fmt

# Check formatting without changes
cargo fmt --check
```

### Testing

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_name
```

The project includes comprehensive unit and integration tests covering:
- Process enumeration
- Input validation
- Error handling
- Configuration management
- UI logging functionality

### For AI Agents

See [AGENTS.md](AGENTS.md) for detailed guidelines on:
- Code style conventions
- Build/test commands
- Project structure
- Common pitfalls
- Testing strategies

### Recent Improvements

Major updates to the codebase:

1. **Modern UI Redesign**: Clean, simple layout with consistent spacing and typography
2. **Animation System**: Added fade-in, slide-in, and window scaling animations throughout the UI
3. **Log Animations**: Each new log entry fades in smoothly using alpha interpolation
4. **Injection History**: Tracks last 10 successful injections with timestamps
5. **Removed Admin Check**: Injection now works with available permissions, fails gracefully if admin required
6. **Updated Constants**: Added font size constants and animation speed constants
7. **Improved Window**: Larger window (700x700) for better content display
8. **Fixed Unicode Arrow**: Changed to ASCII arrow character for better compatibility
9. **Clean Code**: Simplified UI code, removed complex styling for better maintainability

## Important Notes

### Security Considerations

- **Antivirus Detection**: DLL injection is a common technique monitored by antivirus software
- **Permission Model**: Injection works with current permissions - some processes may require administrator access
- **Process Protection**: System-protected processes cannot be injected

### Best Practices

- **Backup DLLs**: Keep backups of original DLLs when modifying
- **Test in Safe Environment**: First test injection on non-critical applications
- **Monitor Logs**: Always check log output for errors or warnings
- **Close Handles**: All Windows handles are properly closed to prevent leaks

### Limitations

- **Process Protection**: System-protected processes (e.g., `csrss.exe`, `lsass.exe`) cannot be injected
- **Antivirus Interference**: Real-time protection may block injection attempts
- **Permission Dependent**: Some processes may require administrator access for successful injection

## License

This project is provided **as-is** for **educational and development purposes only**.

### Usage Guidelines

- Only inject DLLs into processes you own or have explicit permission to modify
- This tool should not be used for malicious purposes
- Users are responsible for complying with applicable laws and regulations
- The authors are not responsible for any misuse of this software

## Acknowledgments

- **Inspired by**: [FateInjector](https://github.com/fligger/FateInjector) - Original C++ implementation
- **Dependencies**: [egui](https://github.com/emilk/egui), [windows-rs](https://github.com/microsoft/windows-rs), [rfd](https://github.com/PolyMeow/rfd), [serde](https://github.com/serde-rs/serde), [dirs](https://github.com/dirs-dev/dirs-rs), [winres](https://github.com/mxre/winres), [chrono](https://github.com/chronotope/chrono)

## Contributing

Contributions are welcome! Please feel free to:
- Report bugs via issues
- Suggest new features
- Submit pull requests
- Improve documentation

When contributing, please follow guidelines in [AGENTS.md](AGENTS.md).

## Links

- [GitHub Repository](https://github.com/Ian-bug/ruin-injector)
- [Issue Tracker](https://github.com/Ian-bug/ruin-injector/issues)
- [Release Notes](https://github.com/Ian-bug/ruin-injector/releases)

## Code Quality

This project has undergone comprehensive code review and quality improvements:

- All code passes `cargo clippy` linting
- Comprehensive test coverage (13 tests)
- Proper error handling and resource cleanup
- Named constants replacing magic numbers
- Smooth animations and modern UI
- Auto-inject functionality implemented
- RAII patterns for resource management
- Well-documented codebase with AGENTS.md guidelines

---

**Made with Rust**
