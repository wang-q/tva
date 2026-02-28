# CLAUDE.md

此文件为 AI 助手在处理本仓库代码时提供指南与上下文。

## 项目概览

**当前状态**: 活跃开发中 | **主要语言**: Rust

**语言约定**: 为了便于指导，本文件 (`CLAUDE.md`) 使用中文编写，且**与用户交流时请使用中文**。但项目代码中的**所有文档注释 (doc comments)**、**行内注释**以及**提交信息**必须使用**英文**。

`tva` 是一个高性能的命令行 TSV (Tab-Separated Values) 数据处理工具集。它旨在提供类 Unix 的文本处理体验，专注于流式处理、高性能和易用性。
核心设计哲学是“流式优先”，尽量以 O(1) 内存处理大数据。

## 构建命令

### 构建

```bash
# 开发构建
cargo build

# 发布构建 (高性能)
cargo build --release
```

### 文档构建

项目使用 `mdBook` 构建文档网站。文档源位于 `docs/` 目录。
根目录的 `README.md` 是主页的源文件，构建时会自动同步到 `docs/README.md`。

```bash
# 同步 README.md 并构建文档
./build-docs.ps1

# 本地预览 (需手动同步 README.md 或先运行 build-docs.ps1)
mdbook serve
```

Do not use the `cargo build --release` option during development as it takes a long time.

### 测试

```bash
# 运行所有测试
cargo test

```

## 架构

### 源代码组织

- **`src/tva.rs`** - 主程序入口，负责命令行解析和分发。
    - Uses `clap` for argument parsing.
    - Enforces consistent flag naming and help text styles.
- **`src/lib.rs`** - 库入口，导出模块。
- **`src/cmd_tva/`** - 命令实现模块。每个子命令对应一个 `.rs` 文件（例如 `stats.rs`, `select.rs`）。
- **`src/libs/`** - 共享工具库和核心逻辑。
  - **`tsv/`** - 核心 TSV 解析与处理模块。
    - **`fields.rs`** - 强大的字段选择逻辑。
        - 支持统一的字段语法: 数字索引 (`1,3-5`), 倒序范围 (`5-3`), 名称匹配 (`user_id`), 通配符 (`*_time`, `col*`), 名称范围 (`start_col-end_col`)。
        - 处理转义字符，支持包含空格或特殊字符的列名。
    - **`reader.rs`** - 高性能零拷贝 TSV 读取器。
        - `TsvReader`: 管理内部缓冲区，支持行级迭代，避免字符串分配。
        - 自动处理 `CRLF` 和行尾异常。
    - **`record.rs`** - 记录抽象。
        - `TsvRecord`: 拥有数据的记录结构，记录字段偏移量。
        - `TsvRow`: 借用数据的轻量级视图，实现零拷贝访问。
    - **`split.rs`** - 基于 SIMD 的字段切分工具。
        - `TsvSplitter`: 使用 `memchr` 快速迭代字段切片。
  - **`io.rs`** - I/O 辅助函数。
    - 统一处理 stdin/stdout 和文件。
    - 透明处理 `.gz` 压缩/解压。
    - `InputSource`: 提供多文件统一视图。
  - **`select.rs`** - 列选择与重排引擎。
    - `SelectPlan`: 预计算字段映射计划。
    - `write_selected_from_bytes`: 基于计划的高性能零拷贝输出。
  - **`filter.rs`** - 过滤逻辑引擎。
    - 支持多种比较操作符 (`eq`, `le`, `str-in-fld` 等)。
    - 针对数值和字符串优化的求值逻辑。
  - **`sampling.rs`** - 高级采样算法。
    - 实现 Reservoir Sampling (蓄水池采样)。
    - 实现 Weighted Reservoir Sampling (A-Res 算法) - O(K) 内存。
    - 实现 Bernoulli Sampling (Skip Sampling) - 几何分布跳过。
  - **`stats.rs`** - 统计计算。
    - 流式计算 sum, min, max, mean, stdev。
    - 支持中位数和四分位数 (需内存缓冲)。
  - **`number.rs`** - 数字格式化工具。

### 命令结构 (Command Structure)

每个命令在 `src/cmd_tva/` 中作为一个独立的模块实现，必须包含两个公开函数：

1.  **`make_subcommand`**: 定义命令行接口。
    -   返回 `clap::Command`。
    -   使用 `.about(...)` 设置简短描述。
    -   推荐使用 `.after_help(include_str!("../../docs/help/<cmd>.md"))` 引入详细文档。
2.  **`execute`**: 命令执行逻辑。
    -   接收 `&clap::ArgMatches`。
    -   返回 `anyhow::Result<()>`。

示例模式：

```rust
// src/cmd_tva/example.rs
use clap::{Arg, ArgMatches, Command};

pub fn make_subcommand() -> Command {
    Command::new("example")
        .about("Example command description")
        .after_help(include_str!("../../docs/help/example.md"))
        .arg(Arg::new("input")...)
}

pub fn execute(matches: &ArgMatches) -> anyhow::Result<()> {
    // Parse arguments
    let input = matches.get_one::<String>("input");

    // Input/Output handling (using shared libs)
    // let mut writer = crate::libs::io::writer(outfile);
    // for input in crate::libs::io::input_sources(&infiles) { ... }

    // Execution logic
    // ...

    Ok(())
}
```

### 关键架构模式

**流式 vs 内存密集型**:
- 大多数命令（如 `select`, `slice`, `filter`）是流式的，内存占用恒定。
- 部分命令（如 `sort`, `reverse`, `stats` 的某些模式）可能需要加载更多数据到内存。
- 文档中应明确标识内存特性。

**错误处理**:
- 使用 `anyhow::Result` 进行统一的错误传播。

## 开发工作流

### 添加新命令

1.  在 `src/cmd_tva/` 下创建新文件 `yourcommand.rs`。
2.  在 `src/cmd_tva/mod.rs` 中声明该模块。
3.  在 `src/tva.rs` (或主入口) 中注册该子命令。
4.  实现 `make_subcommand` 和执行逻辑。
5.  添加测试文件 `tests/cli_yourcommand.rs`。

### 测试约定

- 集成测试位于 `tests/` 目录下，文件命名为 `cli_<command>.rs`。
- 测试数据通常放在 `tests/data/<command>/` 目录下。
- **推荐使用 `TvaCmd` 辅助结构体**（定义在 `tests/common/mod.rs`）来编写集成测试，以简化子进程调用和断言。
- 必须使用 `assert_cmd::cargo::cargo_bin_cmd!` 宏来定位二进制文件，以兼容自定义构建目录。
- **稳定性原则 (Zero Panic)**: 任何用户输入（包括畸形数据、二进制文件）都不应导致程序 Panic。必须捕获所有错误并返回友好的错误信息。
- **基准测试**: 性能敏感的变更必须伴随 `benches/` 下的基准测试结果（使用 `criterion`）。

示例测试结构：

```rust
// tests/cli_example.rs

#[macro_use]
#[path = "common/mod.rs"]
mod common;

use common::TvaCmd;

#[test]
fn test_example_basic() {
    let input = "id\tval\n1\ta\n";
    let expected = "id\tval\n1\ta\n";
    
    let (result, _) = TvaCmd::new()
        .stdin(input)
        .args(&["example", "--flag"])
        .run();

    assert_eq!(result, expected);
}
```

## 代码规范

- 使用 `cargo fmt` 格式化代码。
- 使用 `cargo clippy` 检查潜在问题。
- 优先使用标准库和项目中已引入的 crate (`csv`, `clap`, `anyhow`, `regex` 等)。
- 保持代码简洁，注重性能。

### 关键算法与性能规范

1.  **采样算法 (Sampling)**:
    *   **加权采样 (Weighted)**: 必须使用 **A-Res 算法** (Efraimidis-Spirakis) 配合 Min-Heap。
        *   *禁止*: 全量排序 (Naive Sort)，因为其内存复杂度为 O(N)。
    *   **伯努利采样 (Bernoulli)**: 必须使用 **几何分布跳过 (Geometric Skip)** 算法。
        *   *禁止*: 对每一行调用 RNG，这在低概率采样时效率极低。

2.  **I/O 与 解析 (I/O & Parsing)**:
    *   **零拷贝 (Zero-Copy)**: 优先使用 `&[u8]` 切片或 `Cow<'a, [u8]>`。
        *   *禁止*: 在热点路径中无谓地将 `&[u8]` 转换为 `String`。
    *   **SIMD**: 使用 `memchr` crate 查找分隔符 (`\t`, `\n`)。
    *   **缓冲**: 所有文件 I/O 必须包裹在 `BufReader` / `BufWriter` 中（推荐缓冲区大小 **128KB** 以减少 syscall）。

### 并发与并行策略 (Concurrency Strategy)

1.  **默认单线程 (Single-Threaded Default)**:
    *   `tva` 的核心命令默认采用单线程流式处理，以保证最低的启动开销和确定的内存使用。
    *   通过管道 (`|`) 组合命令可以让操作系统负责进程级并行。

2.  **未来并行方向 (Future Parallelism)**:
    *   对于 CPU 密集型任务 (如复杂 `filter` 或 `stats`)，架构应支持基于 "Chunking" 的数据并行。
    *   利用 TSV `\n` 的唯一性，将大文件切分为多个 `Chunk`，分发给线程池处理，最后聚合结果。

### 依赖选择策略 (Dependency Policy)

1.  **CSV/TSV 解析**:
    *   **标准路径**: 使用 `csv` crate 处理复杂的 CSV 兼容性场景（如需要处理引号时）。
    *   **高性能路径**: 对于纯 TSV 处理，优先使用 `memchr` + `split` 手写逻辑，避免 `csv` crate 的状态机开销。
    *   **谨慎引入**: 类似 `simd-csv` 的库虽然快，但可能引入复杂的构建依赖或平台限制，仅在基准测试证明有显著收益（>20%）时才考虑引入。

### Hash 算法选择规范

为保证极致性能与安全性的平衡，`tva` 对 Hash 算法的选择有严格规定：

1.  **Hash Map (如 `join` 命令)**:
    *   **必须使用**: `ahash::RandomState`。
    *   **原因**: `ahash` 专为 Rust `HashMap` 设计，支持高效的增量哈希 (Incremental Hashing)，且无需缓冲数据，比一次性哈希更适合 Map 查找场景。同时提供基本的 HashDoS 防护。
    *   **禁止**: 使用默认的 `SipHash` (太慢) 或 `rapidhash` (无原生增量接口，需缓冲，导致 Map 性能下降)。

2.  **一次性哈希 (One-shot Hashing, 如 `uniq` 命令)**:
    *   **推荐使用**: `rapidhash::rapidhash()`。
    *   **原因**: 在可以一次性获取完整 Key 的场景下（如读取整行），`rapidhash` 是目前最快的非加密哈希算法之一。
    *   **适用场景**: 去重、布隆过滤器、数据指纹计算。

3.  **加密/安全哈希**:
    *   **必须使用**: `SipHash` (Rust 默认) 或 `SHA-256`。
    *   **适用场景**: 即使牺牲性能也必须绝对防止 HashDoS 攻击的公共接口，或需要加密签名的场景。

### 架构决策与设计约束 (Architectural Constraints)

1.  **TSV 专用假设 (TSV Assumptions)**:
    *   **无引号优化**: 利用 TSV 不支持引号和转义的特性，核心解析逻辑应完全依赖 SIMD (`memchr`) 扫描，避免复杂的 CSV 状态机。
    *   **行级并行**: 假设 `\n` 具有唯一记录分隔符语义，架构应支持按行切分数据块进行并行处理。

2.  **Join 策略 (Stream-Static Model)**:
    *   **机制**: 采用 "Hash Semi-Join" (Filter + Append) 模式。
    *   **约束**: 仅将 Filter 文件加载到内存 (HashMap)，流式处理 Data 文件。避免像 `xan` 那样全量加载数据以支持 SQL 全连接，保持对大数据集的 O(1) 内存友好性。

3.  **内存热点优化 (Hot Path Memory)**:
    *   **Buffer 复用**: 在处理循环中，必须复用 `Vec`/`String` 缓冲区，禁止在循环体内频繁分配/释放堆内存。
    *   **小对象优化**: 对于短 Key (如 ID, Code)，优先使用 `SmallVec<[u8; N]>` (推荐 N=32) 以利用栈内存，避免 `malloc` 开销。


## 帮助文本规范 (Help Text Style Guide)

### Rust 实现规范 (Implementation)

* **`about`**: 使用第三人称单数动词，简要描述 TSV 操作。
  (e.g., "Converts TSV to markdown table", "Deduplicates TSV rows").
* **`after_help`**: 使用 `include_str!("../../docs/help/<cmd>.md")` 引入外部文档。
* **Arguments**:
    * **Input**: 命名为 `infile` (单文件) 或 `infiles` (多文件)。
        * Help (single): `Input TSV file to process (default: stdin).`
        * Help (multiple): `Input TSV file(s) to process`.
    * **Output**: 命名为 `outfile` (`-o`, `--outfile`)。
        * Help: `Output filename. [stdout] for screen`.
* **Terminology**:
    * 优先使用 "TSV" 而非 "file"。
    * **Column vs Field**:
        * **Column**: 用于文档和概念描述，指代数据的垂直维度 (e.g., "Select the first column").
        * **Field**: 仅用于指代 CLI 参数 (`--field` / `-f`) 或底层分隔符逻辑 (e.g., "tab-separated fields").
    * **Key**:
        * 特指用于排序、连接、去重等操作的那些 Column (e.g., "Sort by the first column as key").
    * **Row vs Line**:
        * **Row**: 指代数据记录 (Data Record)，通常不包含表头。
        * **Line**: 指代物理文本行 (e.g., "The first line is a header").

### 文档内容规范 (Markdown Content)

所有子命令的帮助文档 (`docs/help/<cmd>.md`) 必须遵循以下统一风格：

1.  **标题**:
    *   以 `# <command>` 开头。
2.  **简述 (Description)**:
    *   标题后紧跟一行简洁的功能描述。
    *   可以包含一段关于该命令做什么及其权衡的简短段落。
3.  **分节结构**:
    *   使用 `Section Name:` (加粗，后跟冒号) 作为节标题，而不是 Markdown 的 `##`。
    *   **不要**包含 `Usage:` 或 `Options:` 小节 (这些由 `clap` 自动生成)。
4.  **内容格式**:
    *   **列表**: 使用 `*` 引导的无序列表来描述行为细节 (Input, Header behavior, Logic 等)。
        * TSV input: `* Supports plain text and gzipped (.gz) TSV files`
        * Stdin behavior: `* Reads from stdin if input file is 'stdin' or no input file is given`
    *   **参数引用**: 使用反引号包裹参数，如 `` `--header` / `-H` ``。
    *   **语法说明**: 通常包含 `Input:`, `Field syntax:` 等固定小节。
5.  **示例 (Examples)**:
    *   使用 `Examples:` 作为标题。
    *   采用编号列表 (`1.`, `2.`)。
    *   描述后紧跟一个缩进的代码块或反引号包裹的命令。

### 完整示例 (Example)

```markdown
# command-name

Short description of what the command does.

Input:
*   Reads from files or standard input; multiple files are processed as one stream.
*   Files ending in `.gz` are transparently decompressed.

Specific Behavior:
*   `--flag`: Description of specific flag behavior.

Output:
*   Writes processed records to standard output...

Examples:
1. Example description:
   `tva command -f input.tsv`
```

## 用户文档工作流

我们使用 [mdBook](https://rust-lang.github.io/mdBook/) 生成文档网站。

*   源目录: `docs/`。
*   配置: `book.toml`。
*   本地构建:
    ```bash
    ./build-docs.ps1  # 同步 README.md 并运行 mdbook build
    mdbook serve      # 用于本地预览
    ```
*   部署:
    *   GitHub Actions (`.github/workflows/docs.yml`) 会在推送到 `master` 时自动构建并部署到 `gh-pages` 分支。

## 开发者文档规范

`docs/developer.md` 是供项目开发者参考的内部指南，不要包含在最终生成的用户文档（mdBook 站点）中。

*   **语言**: 使用**中文**编写。
*   **格式**: 避免过多的加粗 (Bold) 或强调格式，以保持在纯文本编辑器中的可读性。
*   **内容**: 包含测试策略、架构设计、功能计划和开发工作流。
