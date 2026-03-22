<div align="center">

[**English**](README.md) | [**中文**](README_CN.md)

# 🎯 Ruin DLL 注入器

**基于 Rust 和 egui 构建的现代化轻量级 DLL 注入器**

[![ko-fi](https://ko-fi.com/img/githubbutton_sm.svg)](https://ko-fi.com/Y8Y01WG0DL)

![截图](screenshot.png)

</div>

---

## ✨ 概述

Ruin Injector 是一款专为 Windows 应用程序设计、强大且易于使用的 DLL 注入器。凭借 Rust 的内存安全保证和 egui 的现代 GUI 框架，它提供了清晰的界面和先进功能，如自动注入、进程过滤和实时日志。

**灵感来自 [FateInjector](https://github.com/fligger/FateInjector)**

---

## 🚀 快速开始

### 从源码构建

```bash
git clone https://github.com/Ian-bug/ruin-injector.git
cd ruin-injector
cargo build --release
```

编译后的可执行文件位于 `target/release/ruin-injector.exe`。

### 下载发布版本

从 [Releases](https://github.com/Ian-bug/ruin-injector/releases) 页面获取最新的预构建二进制文件。

---

## 📋 系统要求

- **操作系统**: Windows 10/11 (64 位)
- **权限**: 管理员（建议，但并非总是需要）
- **目标**: 要注入的 DLL 文件

---

## 🎮 使用指南

### 基本注入

1. **启动应用程序**
   ```bash
   .\ruin-injector.exe
   ```
   *右键 → "以管理员身份运行" 以获得最大兼容性*

2. **选择 DLL 文件**
   - 点击 **Browse** 按钮
   - 导航到您的 DLL 文件
   - 选择并确认

3. **选择目标进程**
   - 点击 **List** 按钮
   - 搜索或浏览正在运行的进程
   - 点击以选择目标

4. **注入**
   - 点击 **Inject DLL**
   - 监控日志状态

### 自动注入模式

启用目标进程启动时的自动注入：

- ☑️ 勾选 **Auto Inject** 复选框
- 选择您的 DLL 文件
- 选择目标进程
- Ruin Injector 将在进程出现时自动注入
- 设置在会话之间持久保存

### 高级功能

<details>
<summary>📊 进程浏览器</summary>

- **实时进程列表**: 实时枚举所有正在运行的进程
- **搜索过滤**: 按进程名称即时过滤
- **进程详情**: 显示 PID（进程 ID）和名称
- **UWP 检测**: 自动标记通用 Windows 平台应用
- **动画 UI**: 平滑的窗口过渡效果
</details>

<details>
<summary>🎨 视觉增强</summary>

- **类型安全动画系统**: 自定义 Fade、Scale、Slide 和 Pulse 动画
- **平滑过渡**: 淡入标题、滑动内容、缩放对话框
- **日志动画**: 新条目平滑淡入
- **状态指示器**: 管理员/自动注入状态的动画脉冲
- **模态框模糊**: 对话框的模糊背景叠加
</details>

<details>
<summary>⚡ 注入选项</summary>

- **手动注入**: 点击按钮立即注入
- **自动注入**: 自动检测和注入
- **架构验证**: 确保 32 位/64 位兼容性
- **UWP 保护**: 防止注入受保护的应用程序
</details>

<details>
<summary>🔧 错误处理</summary>

- **详细消息**: 带有可操作建议的上下文描述
- **架构不匹配**: 清晰解释位数问题
- **UWP 警告**: 不受支持应用的信息提示
- **权限指导**: "尝试以管理员身份运行"提示
- **DLL 诊断**: 详细失败原因（依赖、反作弊等）
</details>

---

## 🏗️ 架构

### 项目结构

```
ruin-injector/
├── src/
│   ├── main.rs       # 入口点、egui UI、动画系统
│   ├── injector.rs  # 核心注入逻辑、Windows API
│   └── config.rs    # 配置持久化
├── Cargo.toml        # 依赖项和元数据
├── build.rs          # 资源编译（图标嵌入）
├── icon.ico          # 应用程序图标
├── cliff.toml        # Changelog 配置
├── AGENTS.md         # AI 助手指南
├── ICON.md           # 图标使用说明
├── README.md         # 英文文档
└── README_CN.md      # 中文文档（本文件）
```

### 技术栈

| 组件 | 技术 |
|-----------|-----------|
| **语言** | Rust 2021 |
| **GUI 框架** | egui 0.27 |
| **Windows API** | windows-rs 0.54 |
| **文件对话框** | rfd 0.14 |
| **序列化** | serde + serde_json |
| **配置路径** | dirs 5.0 |
| **时间戳** | chrono 0.4 |

---

## 🔬 技术实现

### DLL 注入工作流程

```mermaid
graph LR
    A[进程发现] --> B[UWP 检测]
    B --> C[架构检查]
    C --> D[打开进程]
    D --> E[分配内存]
    E --> F[写入 DLL 路径]
    F --> G[创建远程线程]
    G --> H[等待完成]
    H --> I[清理]
```

#### 详细步骤

1. **进程发现**
   - `CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS)` 枚举
   - 使用 `Process32First`/`Process32Next` 迭代

2. **UWP 检测**
   - 检查 WindowsApps/AppPackages 目录
   - `QueryFullProcessImageNameW` 获取准确路径
   - 返回 `UwpProcessNotSupported` 错误

3. **架构验证**
   - 注入器位数通过 `cfg(target_pointer_width)`
   - 目标架构通过 `IsWow64Process`
   - 防止不匹配注入

4. **进程访问**
   - `OpenProcess(PROCESS_ALL_ACCESS, ...)`
   - 处理 Windows 权限模型
   - 权限不足时优雅失败

5. **内存分配**
   - `VirtualAllocEx(...)` 分配目标内存
   - `PAGE_READWRITE` 保护

6. **DLL 路径注入**
   - `WriteProcessMemory(...)` 写入 UTF-16 路径
   - 验证路径长度（MAX_PATH_LENGTH = 260）

7. **远程线程创建**
   - `CreateRemoteThread(...)` 使用 `LoadLibraryW` 入口点
   - 10 秒超时等待完成

8. **清理**
   - `WaitForSingleObject(...)` 等待线程完成
   - 检查退出代码（NULL = LoadLibraryW 失败）
   - `VirtualFreeEx(...)` 和 `CloseHandle(...)` 释放资源

### 核心使用的 Windows API

| API | 用途 |
|-----|---------|
| `CreateToolhelp32Snapshot` | 创建进程快照 |
| `Process32First/Next` | 枚举进程 |
| `OpenProcess` | 访问目标进程 |
| `IsWow64Process` | 检测进程架构 |
| `QueryFullProcessImageNameW` | 获取完整进程路径 |
| `VirtualAllocEx` | 在目标中分配内存 |
| `WriteProcessMemory` | 将 DLL 路径写入目标 |
| `GetProcAddress` | 获取函数地址 |
| `CreateRemoteThread` | 创建远程执行线程 |
| `LoadLibraryW` | 在目标进程中加载 DLL |
| `CloseHandle` | 释放资源 |
| `GetLastError` | 获取错误代码 |
| `WaitForSingleObject` | 等待线程完成 |
| `GetExitCodeThread` | 检查线程结果 |

### 动画系统

类型安全、模块化的动画架构：

```rust
trait Animatable {
    fn update(&mut self, dt: f32);
    fn is_complete(&self) -> bool;
}

struct Fade { current: f32, target: f32, speed: f32 }
struct Scale { current: f32, target: f32, speed: f32 }
struct Slide { current: f32, target: f32, speed: f32 }
struct Pulse { phase: f32, speed: f32, amplitude: f32, base: f32 }
```

**动画类型**：
- **Fade**: Alpha 值插值用于透明度
- **Scale**: 窗口大小插值用于对话框
- **Slide**: Y 偏移插值用于面板移动
- **Pulse**: 连续相位旋转用于状态指示器

**关键常量**：
```rust
const ANIMATION_DEFAULT_SPEED: f32 = 0.12;
const ANIMATION_FAST_SPEED: f32 = 0.2;
const PULSE_SPEED_DEFAULT: f32 = 0.03;
const ALPHA_THRESHOLD: f32 = 0.01;
```

### 配置管理

**原子写入模式**：
```rust
// 首先写入临时文件
let temp_path = config_path.with_extension("tmp");
fs::write(&temp_path, config_str)?;

// 然后重命名（在大多数文件系统上是原子的）
fs::rename(&temp_path, &config_path)?;
```

**优点**：
- 防止崩溃/断电时的配置损坏
- 线程安全的配置更新
- 首次运行时的优雅回退

### 错误处理

```rust
pub enum InjectionError {
    ProcessNotFound(String),
    OpenProcessFailed(String),
    MemoryAllocationFailed(String),
    WriteMemoryFailed(String),
    CreateRemoteThreadFailed(String),
    InvalidPath(String),
    InvalidProcessName(String),
    PathTooLong(String),
    DllLoadFailed(String),
    ThreadWaitFailed(String),
    UwpProcessNotSupported(String),
}
```

---

## 🛠️ 开发

### 构建命令

```bash
# Release 构建（优化）
cargo build --release

# Development 构建（更快）
cargo build

# 清理 + 重新构建
cargo clean && cargo build --release

# 快速编译检查
cargo check
```

### 代码质量

```bash
# 运行 linter
cargo clippy

# 格式化代码
cargo fmt

# 检查格式
cargo fmt --check

# 运行测试
cargo test

# 带输出测试
cargo test -- --nocapture
```

### 测试覆盖率（v1.3.1）

- 21 个全面的单元和集成测试
- 进程枚举
- 输入验证
- 错误处理
- 配置管理
- UI 日志功能
- 动画系统测试
- 模态动画测试
- 按钮悬停动画

### 代码质量标准

✅ 零编译器警告
✅ 零 Clippy 警告
✅ 完全的 rustfmt 合规性
✅ 所有 21 个测试通过
✅ 正确的错误处理和资源清理
✅ 命名常量（无魔术数字）
✅ 类型安全的动画系统和 trait 抽象
✅ 数据完整性的原子配置写入
✅ 资源管理的 RAII 模式

### 对于 AI 助手

请参阅 [AGENTS.md](AGENTS.md) 获取关于以下方面的详细指南：
- 代码风格约定
- 构建/测试命令
- 项目结构
- 常见陷阱
- 测试策略
- 动画系统架构

---

## 📈 最新更新（v1.3.1）

### 近期改进

1. **动画系统重设计**
   - 类型安全架构，带 `Animatable` trait
   - Fade、Scale、Slide、Pulse 动画类型
   - 配置的构建器模式
   - 零魔术数字

2. **UWP 进程检测**
   - 检查 WindowsApps/AppPackages 目录
   - 带警告的 `UwpProcessNotSupported` 错误
   - 准确的路径解析

3. **原子配置写入**
   - 临时文件 + 重命名模式
   - 线程安全更新
   - 优雅的回退处理

4. **增强的错误消息**
   - 架构不匹配检测
   - DLL 加载失败诊断
   - 可操作建议

5. **生产就绪代码库**
   - 所有工具零警告
   - 全面的测试覆盖
   - 清晰、可维护的架构

6. **模态动画**
   - 一致的缩放 + 淡入行为
   - 窗口背景淡入淡出
   - 统一的动画模式

---

## ⚠️ 重要说明

### 安全性考虑

- **杀毒软件监控**: DLL 注入是杀毒软件监控的常见技术
- **权限模型**: 使用当前权限；管理员可能需要
- **受保护进程**: 系统进程无法注入
- **UWP 应用**: 受设计限制的注入能力
- **架构**: 必须匹配（32 位 ↔ 32 位，64 位 ↔ 64 位）

### 最佳实践

- 🔒 首先在安全环境测试
- 💾 备份原始 DLL
- 📊 监控日志输出
- 🎯 验证架构匹配
- 🚫 避免 UWP 应用程序
- 👤 仅注入您拥有的进程

### 限制

- 系统受保护进程（`csrss.exe`、`lsass.exe`）无法注入
- 实时杀毒软件保护可能会阻止尝试
- 某些进程需要管理员访问
- UWP 应用被故意阻止
- 架构不匹配阻止注入

---

## 📄 许可证

本项目**按原样**提供，**仅供教育和开发目的使用**。

### 使用指南

- 仅将 DLL 注入到您拥有或获得修改权限的进程中
- 不得用于恶意目的
- 遵守适用的法律和法规
- 作者不对任何滥用负责

---

## 🙏 致谢

- **灵感来自**: [FateInjector](https://github.com/fligger/FateInjector) - 原始 C++ 实现
- **依赖项**:
  - [egui](https://github.com/emilk/egui) - GUI 框架
  - [windows-rs](https://github.com/microsoft/windows-rs) - Windows API 绑定
  - [rfd](https://github.com/PolyMeow/rfd) - 文件对话框
  - [serde](https://github.com/serde-rs/serde) - 序列化
  - [dirs](https://github.com/dirs-dev/dirs-rs) - 配置路径
  - [winres](https://github.com/mxre/winres) - Windows 资源
  - [chrono](https://github.com/chronotope/chrono) - 时间戳
  - [git-cliff](https://github.com/orhun/git-cliff) - Changelog 生成

---

## 🤝 贡献

欢迎贡献！请随时：

- 🐛 通过 issue 报告错误
- 💡 建议新功能
- 📝 提交拉取请求
- 📚 改进文档

贡献时，请遵循 [AGENTS.md](AGENTS.md) 中的指南。

---

## 🔗 链接

- [GitHub 仓库](https://github.com/Ian-bug/ruin-injector)
- [问题跟踪器](https://github.com/Ian-bug/ruin-injector/issues)
- [发布说明](https://github.com/Ian-bug/ruin-injector/releases)
- [English Documentation](README.md)

---

<div align="center">

**使用 Rust 构建** ❤️  🦀

</div>
