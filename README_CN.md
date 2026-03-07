# Ruin DLL 注入器

基于 Rust 和 egui 构建的现代化轻量级 DLL 注入器，专为 Windows 应用程序设计，包括 UWP（通用 Windows 平台）进程。受 [FateInjector](https://github.com/fligger/FateInjector) 启发。

![截图](screenshot.png)

## ✨ 功能特性

- **现代 GUI**: 使用 egui 构建的简洁响应式界面
- **进程浏览器**: 从运行中的进程中进行可视化选择，支持搜索功能
- **自动配置**: 在会话之间保存设置（DLL 路径、进程偏好）
- **实时日志**: 监控注入状态和详细错误消息
- **无控制台**: 纯 GUI 应用程序 - 无黑色终端窗口
- **UWP 就绪**: 设计用于通用 Windows 平台应用程序
- **轻量级**: 约 4.5 MB 可执行文件，依赖最少

## 📋 系统要求

- Windows 10/11 (64 位)
- 管理员权限（大多数进程需要，尤其是 UWP）
- 要注入的 DLL 文件

## 🚀 快速开始

### 编译

```bash
cargo build --release
```

编译后的可执行文件位于 `target/release/ruin-injector.exe`。

### 添加图标

将 `.ico` 文件命名为 `icon.ico` 放在项目根目录，然后重新编译。详细说明请参阅 [ICON.md](ICON.md)。

## 📖 使用方法

1. **运行应用程序**
   ```bash
   .\ruin-injector.exe
   ```
   （右键 → "以管理员身份运行" 推荐）

2. **选择 DLL 文件**
   - 点击 "Browse..." 按钮
   - 导航到并选择 DLL 文件

3. **选择目标进程**
   - 点击 "📋 Select Process" 按钮
   - 使用搜索框快速过滤进程
   - 点击所需的进程

4. **注入**
   - 点击 "Inject" 按钮
   - 监控日志区域查看成功/错误消息

### 功能详情

#### 进程选择
- **实时进程列表**: 显示所有当前运行的进程
- **搜索/过滤**: 输入以快速查找特定进程
- **进程信息**: 显示进程名和 PID（进程 ID）

#### 注入选项
- **自动注入**: 检测到目标进程时自动注入（功能占位符）
- **手动注入**: 点击注入按钮进行立即注入

## 🎯 UWP 进程注入

UWP（通用 Windows 平台）应用程序在沙盒环境中运行，权限受限。要注入到 UWP 进程：

### 先决条件

1. **管理员权限**: 以管理员身份运行注入器
2. **DLL 文件权限**: DLL 必须可被 UWP 进程访问
   - 右键点击 DLL 文件
   - 属性 → 安全
   - 添加 "All Application Packages" 组
   - 授予 "读取" 和 "执行" 权限

### 常见 UWP 进程名

- **Minecraft**: `Minecraft.Windows.exe`
- **Microsoft Edge**: `MicrosoftEdge.exe`
- **Windows Store**: `WinStore.App.exe`
- **计算器**: `CalculatorApp.exe`
- **照片**: `Microsoft.Photos.exe`

**注意**: UWP 进程名区分大小写。

## 🏗️ 架构

```
rust-injector/
├── src/
│   ├── main.rs          # 应用入口、egui UI、状态管理
│   ├── injector.rs      # 核心注入逻辑、Windows API 调用
│   ├── uwp.rs          # UWP 权限处理
│   └── config.rs       # 配置持久化（JSON）
├── Cargo.toml           # 项目依赖和元数据
├── build.rs             # Windows 资源编译（图标嵌入）
├── icon.ico             # 应用图标（可选，自动嵌入）
├── README.md            # 本文件（英文版）
├── README_CN.md        # 本文件（中文版）
├── ICON.md             # 图标使用说明
└── AGENTS.md           # AI 编程助手指南
```

## ⚙️ 技术实现

### DLL 注入流程

1. **进程发现**
   - 使用 `CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS)` 枚举进程
   - 使用 `Process32First` 和 `Process32Next` 迭代

2. **进程访问**
   - 通过 `OpenProcess(PROCESS_ALL_ACCESS, ...)` 打开目标进程
   - 处理 Windows 权限模型

3. **内存分配**
   - 通过 `VirtualAllocEx(...)` 在目标进程中分配内存
   - 将内存保护设置为 `PAGE_READWRITE`

4. **DLL 路径注入**
   - 使用 `WriteProcessMemory(...)` 将 DLL 路径字符串写入分配的内存
   - 将路径转换为 UTF-16 宽字符串格式

5. **远程线程创建**
   - 通过 `CreateRemoteThread(...)` 在目标进程中创建线程
   - 线程入口点是 `LoadLibraryW`（通过 `GetProcAddress` 获取）

6. **清理**
   - 使用 `WaitForSingleObject(...)` 等待线程完成
   - 使用 `VirtualFreeEx(...)` 释放分配的内存
   - 正确关闭所有句柄

### 使用的核心 Windows API

| API | 用途 |
|-----|---------|
| `CreateToolhelp32Snapshot` | 创建进程快照 |
| `Process32First/Next` | 枚举进程 |
| `OpenProcess` | 访问目标进程 |
| `VirtualAllocEx` | 在目标中分配内存 |
| `WriteProcessMemory` | 将 DLL 路径写入目标 |
| `GetProcAddress` | 获取函数地址 |
| `CreateRemoteThread` | 创建远程执行线程 |
| `LoadLibraryW` | 在目标进程中加载 DLL |
| `CloseHandle` | 释放资源 |

### 错误处理

所有操作都包含全面的错误处理：

```rust
pub enum InjectionError {
    ProcessNotFound(String),
    OpenProcessFailed(String),
    MemoryAllocationFailed(String),
    WriteMemoryFailed(String),
    CreateRemoteThreadFailed(String),
}
```

错误在 UI 日志中显示带有描述性上下文的消息。

## 🛠️ 开发

### 构建命令

```bash
# Release 构建（优化）
cargo build --release

# Development 构建（更快）
cargo build

# 清理并重新构建
cargo clean && cargo build --release

# 检查编译（快速）
cargo check
```

### 代码检查和格式化

```bash
# 运行 clippy 代码检查器
cargo clippy

# 使用 rustfmt 格式化代码
cargo fmt

# 检查格式化而不修改
cargo fmt --check
```

### 对于 AI 助手

请参阅 [AGENTS.md](AGENTS.md) 获取关于以下方面的详细指南：
- 代码风格约定
- 构建/测试命令
- 项目结构
- 常见陷阱
- 测试策略

## ⚠️ 重要说明

### 安全性考虑

- **杀毒软件检测**: DLL 注入是杀毒软件监控的常见技术
- **管理员访问**: 注入大多数进程需要
- **权限模型**: UWP 应用程序具有受限权限

### 最佳实践

- **备份 DLL**: 修改原始 DLL 时请保留备份
- **安全环境测试**: 首先在非关键应用程序上测试注入
- **监控日志**: 始终检查日志输出中的错误或警告
- **关闭句柄**: 所有 Windows 句柄都正确关闭以防止泄漏

### 限制

- **UWP 权限**: 可能需要手动 DLL 权限设置
- **进程保护**: 系统保护的进程（如 `csrss.exe`、`lsass.exe`）无法注入
- **杀毒软件干扰**: 实时保护可能会阻止注入尝试

## 📄 许可证

本项目**按原样**提供，**仅供教育和开发目的使用**。

### 使用指南

- 仅将 DLL 注入到您拥有或具有明确修改权限的进程中
- 本工具不应用于恶意目的
- 用户有责任遵守适用的法律和法规
- 作者不对本软件的任何滥用负责

## 🙏 致谢

- **启发来源**: [FateInjector](https://github.com/fligger/FateInjector) - 原始 C++ 实现
- **GUI 框架**: [egui](https://github.com/emilk/egui) - 即时模式 GUI 库
- **窗口框架**: [eframe](https://github.com/emilk/egui) - egui 框架集成
- **Windows 绑定**: [windows-rs](https://github.com/microsoft/windows-rs) - 微软官方 Rust 绑定
- **文件对话框**: [rfd](https://github.com/PolyMeow/rfd) - Rust 文件对话框
- **序列化**: [serde](https://github.com/serde-rs/serde) - 序列化框架
- **配置目录**: [dirs](https://github.com/dirs-dev/dirs-rs) - 跨平台配置目录
- **Windows 资源**: [winres](https://github.com/mxre/winres) - Windows 资源编译器

## 📞 贡献

欢迎贡献！请随时：
- 通过 issue 报告错误
- 建议新功能
- 提交拉取请求
- 改进文档

贡献时，请遵循 [AGENTS.md](AGENTS.md) 中的指南。

## 🔗 链接

- [GitHub 仓库](https://github.com/Ian-bug/ruin-injector)
- [问题跟踪器](https://github.com/Ian-bug/ruin-injector/issues)
- [发布说明](https://github.com/Ian-bug/ruin-injector/releases)

---

**用 Rust ❤️ 制作**
