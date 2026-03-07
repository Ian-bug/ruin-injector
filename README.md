# Ruin DLL Injector

A modern, lightweight DLL injector built with Rust and egui, designed for Windows applications including UWP (Universal Windows Platform) processes. Inspired by [FateInjector](https://github.com/fligger/FateInjector).

![Screenshot](screenshot.png)

## ✨ Features

- **Modern GUI**: Clean, responsive interface built with egui
- **Process Browser**: Visual selection from running processes with search functionality
- **Auto Configuration**: Saves your settings (DLL path, process preferences) between sessions
- **Real-time Logging**: Monitor injection status with detailed error messages
- **No Console**: Pure GUI application - no black terminal windows
- **UWP Ready**: Designed to work with Universal Windows Platform applications
- **Lightweight**: ~4.5 MB executable with minimal dependencies

## 📋 System Requirements

- Windows 10/11 (64-bit)
- Administrator privileges (required for most processes, especially UWP)
- The DLL file you want to inject

## 🚀 Quick Start

### Building

```bash
cargo build --release
```

The compiled executable will be at `target/release/ruin-injector.exe`.

### Adding an Icon

Place your `.ico` file as `icon.ico` in the project root directory and rebuild. See [ICON.md](ICON.md) for detailed instructions.

## 📖 Usage

1. **Run the Application**
   ```bash
   .\ruin-injector.exe
   ```
   (Right-click → "Run as administrator" recommended)

2. **Select DLL File**
   - Click "Browse..." button
   - Navigate to and select your DLL file

3. **Choose Target Process**
   - Click "📋 Select Process" button
   - Use the search box to filter processes
   - Click on the desired process

4. **Inject**
   - Click "Inject" button
   - Monitor the log section for success/error messages

### Features in Detail

#### Process Selection
- **Live Process List**: Shows all currently running processes
- **Search/Filter**: Type to quickly find specific processes
- **Process Info**: Displays process name and PID (Process ID)

#### Injection Options
- **Auto Inject**: Automatically inject when the target process is detected (feature placeholder)
- **Manual Inject**: Click the Inject button for immediate injection

## 🎯 UWP Process Injection

UWP (Universal Windows Platform) applications run in a sandboxed environment with restricted permissions. To inject into UWP processes:

### Prerequisites

1. **Administrator Privileges**: Run the injector as administrator
2. **DLL File Permissions**: The DLL must be accessible to UWP processes
   - Right-click the DLL file
   - Properties → Security
   - Add "All Application Packages" group
   - Grant "Read" and "Execute" permissions

### Common UWP Process Names

- **Minecraft**: `Minecraft.Windows.exe`
- **Microsoft Edge**: `MicrosoftEdge.exe`
- **Windows Store**: `WinStore.App.exe`
- **Calculator**: `CalculatorApp.exe`
- **Photos**: `Microsoft.Photos.exe`

**Note**: UWP process names are case-sensitive.

## 🏗️ Architecture

```
rust-injector/
├── src/
│   ├── main.rs          # Application entry, egui UI, state management
│   ├── injector.rs      # Core injection logic, Windows API calls
│   ├── uwp.rs          # UWP permission handling
│   └── config.rs       # Configuration persistence (JSON)
├── Cargo.toml           # Project dependencies and metadata
├── build.rs             # Windows resource compilation (icon embedding)
├── icon.ico             # Application icon (optional, auto-embedded)
├── README.md            # This file
├── README_CN.md        # Chinese documentation
├── ICON.md             # Icon usage instructions
└── AGENTS.md           # Guidelines for AI coding assistants
```

## ⚙️ Technical Implementation

### DLL Injection Process

1. **Process Discovery**
   - Uses `CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS)` to enumerate processes
   - Iterates with `Process32First` and `Process32Next`

2. **Process Access**
   - Opens target process with `OpenProcess(PROCESS_ALL_ACCESS, ...)`
   - Handles Windows permission models

3. **Memory Allocation**
   - Allocates memory in target process via `VirtualAllocEx(...)`
   - Sets memory protection to `PAGE_READWRITE`

4. **DLL Path Injection**
   - Writes DLL path string to allocated memory using `WriteProcessMemory(...)`
   - Converts path to UTF-16 wide string format

5. **Remote Thread Creation**
   - Creates a thread in the target process via `CreateRemoteThread(...)`
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

### Error Handling

All operations include comprehensive error handling:

```rust
pub enum InjectionError {
    ProcessNotFound(String),
    OpenProcessFailed(String),
    MemoryAllocationFailed(String),
    WriteMemoryFailed(String),
    CreateRemoteThreadFailed(String),
}
```

Errors are displayed in the UI log with descriptive context messages.

## 🛠️ Development

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

### For AI Agents

See [AGENTS.md](AGENTS.md) for detailed guidelines on:
- Code style conventions
- Build/test commands
- Project structure
- Common pitfalls
- Testing strategies

## ⚠️ Important Notes

### Security Considerations

- **Antivirus Detection**: DLL injection is a common technique monitored by antivirus software
- **Administrator Access**: Required for injecting into most processes
- **Permission Models**: UWP apps have restricted permissions

### Best Practices

- **Backup DLLs**: Keep backups of original DLLs when modifying
- **Test in Safe Environment**: First test injection on non-critical applications
- **Monitor Logs**: Always check log output for errors or warnings
- **Close Handles**: All Windows handles are properly closed to prevent leaks

### Limitations

- **UWP Permissions**: Manual DLL permission setup may be required
- **Process Protection**: System-protected processes (e.g., `csrss.exe`, `lsass.exe`) cannot be injected
- **Antivirus Interference**: Real-time protection may block injection attempts

## 📄 License

This project is provided **as-is** for **educational and development purposes only**.

### Usage Guidelines

- Only inject DLLs into processes you own or have explicit permission to modify
- This tool should not be used for malicious purposes
- Users are responsible for complying with applicable laws and regulations
- The authors are not responsible for any misuse of this software

## 🙏 Acknowledgments

- **Inspired by**: [FateInjector](https://github.com/fligger/FateInjector) - Original C++ implementation
- **GUI Framework**: [egui](https://github.com/emilk/egui) - Immediate mode GUI library
- **Windowing**: [eframe](https://github.com/emilk/egui) - egui framework integration
- **Windows Bindings**: [windows-rs](https://github.com/microsoft/windows-rs) - Microsoft official Rust bindings
- **File Dialog**: [rfd](https://github.com/PolyMeow/rfd) - Rust file dialogs
- **Serialization**: [serde](https://github.com/serde-rs/serde) - Serialization framework
- **Config Dir**: [dirs](https://github.com/dirs-dev/dirs-rs) - Cross-platform config directories
- **Windows Resources**: [winres](https://github.com/mxre/winres) - Windows resource compiler

## 📞 Contributing

Contributions are welcome! Please feel free to:
- Report bugs via issues
- Suggest new features
- Submit pull requests
- Improve documentation

When contributing, please follow the guidelines in [AGENTS.md](AGENTS.md).

## 🔗 Links

- [GitHub Repository](https://github.com/yourusername/ruin-injector)
- [Issue Tracker](https://github.com/yourusername/ruin-injector/issues)
- [Release Notes](https://github.com/yourusername/ruin-injector/releases)

---

**Made with ❤️ in Rust**
