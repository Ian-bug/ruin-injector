# Ruin DLL 注入器

基于 Rust 和 egui 构建的现代化轻量级 DLL 注入器，专为 Windows 应用程序设计。受 [FateInjector](https://github.com/fligger/FateInjector) 启发。

![截图](screenshot.png)

## 功能特性

- **现代 GUI**: 使用 egui 构建的简洁响应式界面，带有流畅动画
- **动画过渡**: 淡入、滑入和窗口缩放动画
- **进程浏览器**: 从运行中的进程中进行可视化选择，支持搜索功能
- **自动配置**: 在会话之间保存设置（DLL 路径、进程偏好）
- **自动注入**: 检测到目标进程时自动注入
- **注入历史**: 记录最近 10 次注入及其时间戳
- **实时日志**: 监控注入状态，带有动画日志条目
- **无控制台**: 纯 GUI 应用程序 - 无黑色终端窗口
- **轻量级**: 约 4.5 MB 可执行文件，依赖最少

## 系统要求

- Windows 10/11 (64 位)
- 管理员权限（某些进程需要，并非所有）
- 要注入的 DLL 文件

## 快速开始

### 编译

```bash
cargo build --release
```

编译后的可执行文件位于 `target/release/ruin-injector.exe`。

### 添加图标

将 `.ico` 文件命名为 `icon.ico` 放在项目根目录，然后重新编译。详细说明请参阅 [ICON.md](ICON.md)。

## 使用方法

1. **运行应用程序**
    ```bash
    .\ruin-injector.exe
    ```
    （右键 -> "以管理员身份运行" 以获得最大兼容性）

2. **选择 DLL 文件**
    - 点击 "Browse" 按钮
    - 导航到并选择 DLL 文件

3. **选择目标进程**
    - 点击 "List" 按钮
    - 使用搜索框快速过滤进程
    - 点击所需的进程

4. **注入**
    - 点击 "Inject DLL" 按钮
    - 监控日志区域查看成功/错误消息

### 功能详情

#### 进程选择
- **实时进程列表**: 显示所有当前运行的进程
- **搜索/过滤**: 输入以快速查找特定进程
- **进程信息**: 显示进程名和 PID（进程 ID）
- **动画窗口**: 进程列表以平滑缩放动画打开

#### 注入选项
- **自动注入**: 检测到目标进程时自动注入
  - 通过 UI 中的复选框启用
  - 设置在会话之间持久化
  - 检测进程启动并自动注入
  - 启用时显示 "Active" 指示器
- **手动注入**: 点击注入按钮进行立即注入

#### 视觉增强
- **淡入动画**: 标题在启动时平滑淡入
- **滑动动画**: 内容从下方滑入
- **日志动画**: 新日志条目在添加时淡入
- **窗口缩放**: 进程选择器窗口平滑缩进/缩出

## 架构

```
rust-injector/
├── src/
│   ├── main.rs          # 应用入口、egui UI、动画、状态管理
│   ├── injector.rs      # 核心注入逻辑、Windows API 调用
│   └── config.rs       # 配置持久化（JSON）
├── Cargo.toml           # 项目依赖和元数据
├── build.rs             # Windows 资源编译（图标嵌入）
├── icon.ico             # 应用图标（可选，自动嵌入）
├── README.md            # 英文文档
├── README_CN.md        # 本文件（中文版）
├── ICON.md             # 图标使用说明
└── AGENTS.md           # AI 编程助手指南
```

## 技术实现

### DLL 注入流程

1. **进程发现**
   - 使用 `CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS)` 枚举进程
   - 使用 `Process32First` 和 `Process32Next` 迭代

2. **进程访问**
   - 通过 `OpenProcess(PROCESS_ALL_ACCESS, ...)` 打开目标进程
   - 处理 Windows 权限模型
   - 如果权限不足则优雅失败

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

### 动画系统

应用程序使用自定义动画系统：

- **线性插值**: `lerp()` 函数用于平滑过渡
- **淡入动画**: UI 元素的 Alpha 值插值
- **滑动动画**: 面板移动的 Y 偏移插值
- **缩放动画**: 对话框的窗口缩放插值
- **日志 Alpha**: 每条日志的透明度用于平滑淡入

**动画常量**：
```rust
const ANIMATION_SPEED: f32 = 0.15;  // 每帧接近 15%
const NEW_LOG_DURATION_FRAMES: usize = 120;  // 60fps 下 2 秒
```

### 错误处理

所有操作都包含全面的错误处理：

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

错误在 UI 日志中显示带有描述性上下文的消息。在所有错误路径中都会进行适当的资源清理。

**注意**: 已移除管理员检查 - 注入现在使用当前权限尝试，如果需要提升权限则优雅失败。

## 开发

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

### 测试

```bash
# 运行所有测试
cargo test

# 带输出运行测试
cargo test -- --nocapture

# 运行特定测试
cargo test test_name
```

项目包含全面的单元和集成测试，涵盖：
- 进程枚举
- 输入验证
- 错误处理
- 配置管理
- UI 日志功能

### 对于 AI 助手

请参阅 [AGENTS.md](AGENTS.md) 获取关于以下方面的详细指南：
- 代码风格约定
- 构建/测试命令
- 项目结构
- 常见陷阱
- 测试策略

### 最近改进

代码库的主要更新：

1. **现代 UI 重设计**: 简洁、简单的布局，具有一致的间距和排版
2. **动画系统**: 在整个 UI 中添加了淡入、滑入和窗口缩放动画
3. **日志动画**: 每条新日志使用 alpha 插值平滑淡入
4. **注入历史**: 跟踪最近 10 次成功注入及其时间戳
5. **移除管理员检查**: 注入现在使用可用权限工作，如果需要管理员权限则优雅失败
6. **更新常量**: 添加了字体大小常量和动画速度常量
7. **改进窗口**: 更大的窗口（700x700）以更好地显示内容
8. **修复 Unicode 箭头**: 更改为 ASCII 箭头字符以获得更好的兼容性
9. **清洁代码**: 简化 UI 代码，移除复杂样式以提高可维护性

## 重要说明

### 安全性考虑

- **杀毒软件检测**: DLL 注入是杀毒软件监控的常见技术
- **权限模型**: 注入使用当前权限工作 - 某些进程可能需要管理员访问权限
- **进程保护**: 系统保护的进程无法注入

### 最佳实践

- **备份 DLL**: 修改原始 DLL 时请保留备份
- **安全环境测试**: 首先在非关键应用程序上测试注入
- **监控日志**: 始终检查日志输出中的错误或警告
- **关闭句柄**: 所有 Windows 句柄都正确关闭以防止泄漏

### 限制

- **进程保护**: 系统保护的进程（如 `csrss.exe`、`lsass.exe`）无法注入
- **杀毒软件干扰**: 实时保护可能会阻止注入尝试
- **权限依赖**: 某些进程可能需要管理员访问权限才能成功注入

## 许可证

本项目**按原样**提供，**仅供教育和开发目的使用**。

### 使用指南

- 仅将 DLL 注入到您拥有或具有明确修改权限的进程中
- 本工具不应用于恶意目的
- 用户有责任遵守适用的法律和法规
- 作者不对本软件的任何滥用负责

## 致谢

- **启发来源**: [FateInjector](https://github.com/fligger/FateInjector) - 原始 C++ 实现
- **依赖项**: [egui](https://github.com/emilk/egui), [windows-rs](https://github.com/microsoft/windows-rs), [rfd](https://github.com/PolyMeow/rfd), [serde](https://github.com/serde-rs/serde), [dirs](https://github.com/dirs-dev/dirs-rs), [winres](https://github.com/mxre/winres), [chrono](https://github.com/chronotope/chrono)

## 贡献

欢迎贡献！请随时：
- 通过 issue 报告错误
- 建议新功能
- 提交拉取请求
- 改进文档

贡献时，请遵循 [AGENTS.md](AGENTS.md) 中的指南。

## 链接

- [GitHub 仓库](https://github.com/Ian-bug/ruin-injector)
- [问题跟踪器](https://github.com/Ian-bug/ruin-injector/issues)
- [发布说明](https://github.com/Ian-bug/ruin-injector/releases)

## 代码质量

本项目已进行全面代码审查和质量改进：

- 所有代码通过 `cargo clippy` 检查
- 全面的测试覆盖率（13 个测试）
- 正确的错误处理和资源清理
- 用命名常量替换魔术数字
- 流畅的动画和现代 UI
- 自动注入功能已实现
- 资源管理的 RAII 模式
- 带有 AGENTS.md 指南的详细文档代码库

---

**用 Rust 制作**
