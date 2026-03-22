# Ruin DLL 注入器

基于 Rust 和 egui 构建的现代化轻量级 DLL 注入器，专为 Windows 应用程序设计。受 [FateInjector](https://github.com/fligger/FateInjector) 启发。

[![ko-fi](https://ko-fi.com/img/githubbutton_sm.svg)](https://ko-fi.com/Y8Y01WG0DL)

![截图](screenshot.png)

## 功能特性

- **现代 GUI**: 使用 egui 构建的简洁响应式界面，带有类型安全的动画
- **高级动画系统**: 自定义 Fade、Scale、Slide 和 Pulse 动画类型，使用 Animatable trait
- **进程浏览器**: 从运行中的进程中进行可视化选择，支持搜索功能
- **自动配置**: 原子配置写入防止损坏（临时文件 + 重命名模式）
- **自动注入**: 检测到目标进程时自动注入
- **注入历史**: 记录最近 10 次注入及其时间戳
- **实时日志**: 监控注入状态，带有动画日志条目
- **UWP 保护**: 检测并阻止注入到 UWP 应用（WindowsApps/AppPackages）
- **无控制台**: 纯 GUI 应用程序 - 无黑色终端窗口
- **轻量级**: 约 4.5 MB 可执行文件，依赖最少
- **强大的错误处理**: 全面的错误类型和可操作的消息
- **架构检测**: 防止 32 位/64 位不匹配错误
- **管理员状态**: 带有脉冲动画的实时管理员权限指示器

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
    - 使用搜索框过滤进程
    - 点击所需的进程

4. **注入**
    - 点击 "Inject DLL" 按钮
    - 监控日志区域查看成功/错误消息

### 功能详情

#### 进程选择
- **实时进程列表**: 显示所有当前运行的进程
- **搜索/过滤**: 输入以快速查找特定进程
- **进程信息**: 显示进程名和 PID（进程 ID）
- **UWP 检测**: 自动识别并警告 UWP 应用程序
- **动画窗口**: 进程列表以平滑缩放动画打开

#### 注入选项
- **自动注入**: 检测到目标进程时自动注入
  - 通过 UI 中的复选框启用
  - 设置在会话之间持久化（原子写入）
  - 检测进程启动并自动注入
  - 启用时显示 "Active" 指示器（带脉冲动画）
- **手动注入**: 点击注入按钮立即注入
- **架构验证**: 确保注入器和目标进程位数匹配

#### 视觉增强
- **类型安全动画**: Fade、Scale、Slide、Pulse 结构体，带 Animatable trait
- **淡入动画**: 标题在启动时平滑淡入
- **滑动动画**: 内容从下方滑入
- **日志动画**: 新日志条目在添加时淡入
- **窗口缩放**: 进程选择器窗口平滑缩进/缩出
- **模态框模糊**: 对话框使用模糊背景叠加
- **状态指示器**: 管理员/自动注入状态的动画脉冲

#### 错误处理
- **详细消息**: 带有建议的上下文错误描述
- **架构不匹配**: 明确解释位数问题
- **UWP 进程**: 关于不支持应用的警告信息
- **可操作建议**: 适用时显示"尝试以管理员身份运行"
- **DLL 加载失败**: 详细原因（缺少依赖、反作弊等）

## 架构

```
rust-injector/
├── src/
│   ├── main.rs          # 入口、egui UI、动画系统、状态管理
│   ├── injector.rs      # 核心注入、Windows API、UWP 检测、架构检查
│   └── config.rs       # 配置持久化，带原子写入
├── Cargo.toml           # 项目依赖和元数据
├── build.rs             # Windows 资源编译（图标嵌入）
├── icon.ico             # 应用图标（可选，自动嵌入）
├── cliff.toml           # Changelog 生成配置
├── README.md            # 英文文档
├── README_CN.md        # 中文文档（本文件）
├── ICON.md             # 图标使用说明
└── AGENTS.md           # AI 编程助手指南
```

## 技术实现

### DLL 注入流程

1. **进程发现**
   - 使用 `CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS)` 枚举进程
   - 使用 `Process32First` 和 `Process32Next` 迭代

2. **UWP 检测**（新增）
   - 检查进程路径是否包含 WindowsApps/AppPackages 目录
   - 使用 `QueryFullProcessImageNameW` 获取准确路径（Windows 8.1+）
   - 对 UWP 应用返回 `UwpProcessNotSupported` 错误
   - 提供可操作的警告消息

3. **架构兼容性**
   - 通过 `cfg(target_pointer_width)` 检查注入器位数（32/64）
   - 使用 `IsWow64Process` 检测目标进程架构
   - 通过清晰的错误消息防止不匹配的注入

4. **进程访问**
   - 通过 `OpenProcess(PROCESS_ALL_ACCESS, ...)` 打开目标进程
   - 处理 Windows 权限模型
   - 如果权限不足则优雅失败

5. **内存分配**
   - 通过 `VirtualAllocEx(...)` 在目标进程中分配内存
   - 将内存保护设置为 `PAGE_READWRITE`

6. **DLL 路径注入**
   - 使用 `WriteProcessMemory(...)` 将 DLL 路径字符串写入分配的内存
   - 将路径转换为 UTF-16 宽字符串格式
   - 验证路径长度（MAX_PATH_LENGTH = 260）

7. **远程线程创建**
   - 通过 `CreateRemoteThread(...)` 在目标进程中创建线程
   - 线程入口点是 `LoadLibraryW`（通过 `GetProcAddress` 获取）
   - 等待完成，超时为 10 秒

8. **清理**
   - 使用 `WaitForSingleObject(...)` 等待线程完成
   - 检查线程退出代码（NULL 表示 LoadLibraryW 失败）
   - 使用 `VirtualFreeEx(...)` 释放分配的内存
   - 正确关闭所有句柄

### 使用的核心 Windows API

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

应用程序使用类型安全的模块化动画系统：

**动画类型**:
```rust
trait Animatable {
    fn update(&mut self, dt: f32);
    fn is_complete(&self) -> bool;
}

struct Fade { current: f32, target: f32, speed: f32 }
struct Scale { current: f32, target: f32, speed: f32 }
struct Slide { current: f32, target: f32, speed: f32 }
struct Pulse { phase: f32, speed: f32, amplitude: f32, base: f32 }
struct ModalAnimation { fade: Fade, scale: Scale }
```

**用途**:
- **淡入动画**: UI 元素的 Alpha 值插值
- **滑动动画**: 面板移动的 Y 偏移插值
- **缩放动画**: 对话框的窗口缩放插值
- **脉冲动画**: 状态指示器的连续相位旋转
- **日志 Alpha**: 每条日志的透明度，用于平滑淡入

**动画常量**:
```rust
const ANIMATION_DEFAULT_SPEED: f32 = 0.12;
const ANIMATION_FAST_SPEED: f32 = 0.2;
const PULSE_SPEED_DEFAULT: f32 = 0.03;
const ALPHA_THRESHOLD: f32 = 0.01;
const SCALE_THRESHOLD: f32 = 0.01;
```

### 配置管理（已更新）

**原子写入模式**:
```rust
// 首先写入临时文件
let temp_path = config_path.with_extension("tmp");
fs::write(&temp_path, config_str)?;

// 然后重命名（在大多数文件系统上是原子的）
fs::rename(&temp_path, &config_path)?;
```

**优点**:
- 防止崩溃/断电时的配置损坏
- 线程安全的配置更新
- 首次运行时的优雅回退

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
    PathTooLong(String),
    DllLoadFailed(String),
    ThreadWaitFailed(String),
    UwpProcessNotSupported(String),  // 新增
}
```

**错误显示**:
- 错误在 UI 日志中显示，带有描述性上下文
- 颜色编码（错误为红色，成功为绿色）
- 可操作的建议（例如，"尝试以管理员身份运行"）
- DLL 加载失败的详细原因

## 开发

### 构建命令

```bash
# Release 构建（优化）
cargo build --release

# Development 构建（更快）
cargo build

# 清理并重新构建
cargo clean && cargo build --release

# 快速编译检查
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

**测试覆盖率**（v1.3.0）:
- 21 个全面的单元和集成测试
- 进程枚举
- 输入验证
- 错误处理
- 配置管理
- UI 日志功能
- 动画系统测试
- 模态动画测试
- 按钮悬停动画

### 对于 AI 助手

请参阅 [AGENTS.md](AGENTS.md) 获取关于以下方面的详细指南：
- 代码风格约定
- 构建/测试命令
- 项目结构
- 常见陷阱
- 测试策略
- 动画系统架构

### 最近改进（v1.3.0）

代码库的主要更新：

1. **动画系统重设计**: 完全重写，使用类型安全架构
   - 统一接口的 Animatable trait
   - Fade、Scale、Slide、Pulse 动画类型
   - 从 AnimationState 中移除原始 f32 字段
   - 配置的构建器模式（with_speed、with_easing）
   - 消除所有魔术数字

2. **UWP 进程检测**: 防止注入到通用 Windows 平台应用
   - 检查 WindowsApps/AppPackages 目录
   - 带有警告消息的 UwpProcessNotSupported 错误
   - 用于准确路径的 QueryFullProcessImageNameW

3. **原子配置写入**: 防止配置文件损坏
   - 临时文件 + 重命名模式
   - 线程安全更新
   - 优雅的回退处理

4. **增强的错误消息**: 详细的、可操作的错误描述
   - 架构不匹配检测和解释
   - DLL 加载失败原因（依赖、反作弊等）
   - "尝试以管理员身份运行"建议

5. **代码质量**: 生产就绪的代码库
   - 零编译器警告
   - 零 Clippy 警告
   - 完全的 rustfmt 合规性
   - 所有 21 个测试通过

6. **模态动画修复**: 一致的缩放 + 淡入行为
   - 窗口背景现在正确淡入淡出
   - 两个模态框使用相同的动画模式

## 重要说明

### 安全性考虑

- **杀毒软件检测**: DLL 注入是杀毒软件监控的常见技术
- **权限模型**: 注入使用当前权限工作 - 某些进程可能需要管理员访问权限
- **进程保护**: 系统保护的进程无法注入
- **UWP 应用**: 通用 Windows 平台应用程序具有受限的注入能力
- **架构匹配**: 注入器和目标进程必须匹配（32 位或 64 位）

### 最佳实践

- **备份 DLL**: 修改原始 DLL 时请保留备份
- **安全环境测试**: 首先在非关键应用程序上测试注入
- **监控日志**: 始终检查日志输出中的错误或警告
- **关闭句柄**: 所有 Windows 句柄都正确关闭以防止泄漏
- **检查架构**: 确保您的 DLL 匹配目标进程位数
- **避免 UWP**: 不要尝试注入到 UWP 应用程序

### 限制

- **进程保护**: 系统保护的进程（如 `csrss.exe`、`lsass.exe`）无法注入
- **杀毒软件干扰**: 实时保护可能会阻止注入尝试
- **权限依赖**: 某些进程可能需要管理员访问权限才能成功注入
- **UWP 应用程序**: UWP 应用无法注入（故意阻止）
- **架构不匹配**: 32 位注入器无法注入 64 位进程（反之亦然）

## 许可证

本项目**按原样**提供，**仅供教育和开发目的使用**。

### 使用指南

- 仅将 DLL 注入到您拥有或具有明确修改权限的进程中
- 本工具不应用于恶意目的
- 用户有责任遵守适用的法律和法规
- 作者不对本软件的任何滥用负责

## 致谢

- **启发来源**: [FateInjector](https://github.com/fligger/FateInjector) - 原始 C++ 实现
- **依赖项**: [egui](https://github.com/emilk/egui)、[windows-rs](https://github.com/microsoft/windows-rs)、[rfd](https://github.com/PolyMeow/rfd)、[serde](https://github.com/serde-rs/serde)、[serde_json](https://github.com/serde-rs/json)、[dirs](https://github.com/dirs-dev/dirs-rs)、[winres](https://github.com/mxre/winres)、[chrono](https://github.com/chronotope/chrono)、[git-cliff](https://github.com/orhun/git-cliff)

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

- 所有代码通过 `cargo clippy` 代码检查（零警告）
- 所有代码通过 `cargo fmt` 格式化（零警告）
- 全面的测试覆盖率（21 个测试，全部通过）
- 正确的错误处理和资源清理
- 用命名常量替换魔术数字
- 类型安全的动画系统和 trait 抽象
- 自动注入功能已实现
- 用于数据完整性的原子配置写入
- 用于安全的 UWP 进程检测
- 防止不匹配注入的架构验证
- 资源管理的 RAII 模式
- 带有 AGENTS.md 指南的详细文档代码库

---

**用 Rust 制作** 🦀
