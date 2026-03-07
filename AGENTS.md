# CLAUDE.md

此文件为 AI 助手在处理本仓库代码时提供指南与上下文。

## 项目概览

**当前状态**: 活跃开发中 | **主要语言**: Rust

**语言约定**: 为了便于指导，本文件 (`AGENTS.md`) 使用中文编写，且**与用户交流时请使用中文**。但项目代码中的**所有文档注释 (doc comments)**、**行内注释**以及**提交信息**必须使用**英文**。

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
- **`src/cmd_tva/`** - 命令实现模块。
    - **Selection & Sampling**: `sample.rs`, `select.rs`, `slice.rs`
    - **Filtering**: `filter.rs`
    - **Ordering**: `reverse.rs`, `sort.rs`, `transpose.rs`
    - **Statistics & Summary**: `bin.rs`, `stats.rs`, `uniq.rs`
    - **Reshaping**: `blank.rs`, `fill.rs`, `longer.rs`, `wider.rs`
    - **Combining & Splitting**: `append.rs`, `join.rs`, `split.rs`
    - **Formatting & Utilities**: `check.rs`, `keep_header.rs`, `nl.rs`
    - **Import & Export**:
        - `from/`: `csv.rs`, `html.rs`, `xlsx.rs`, `mod.rs`
        - `to/`: `csv.rs`, `md.rs`, `xlsx.rs`, `mod.rs`
- **`src/libs/`** - 共享工具库和核心逻辑。
  - **`aggregation/`** - 高性能 SoA 聚合引擎 (用于 `stats`)。
    - **`aggregator.rs`** - 扁平化的状态存储 (`Vec<f64>`)，实现 Struct-of-Arrays 布局。
    - **`processor.rs`** - 聚合计划与执行器，管理 `Calculator` 集合。
    - **`ops/`** - 具体算子实现 (Sum, Mean, Unique 等)，通过 `Calculator` trait 解耦。
    - **`math.rs`** - 核心数学函数库 (Mean, Variance, Quantile, MAD)。
  - **`cell.rs`** - AoS 聚合单元 (用于 `wider`)。
    - `Cell`: 动态类型的聚合状态容器，支持多种 OpKind。
  - **`filter/`** - 模块化过滤引擎。
    - **`config.rs`** - 过滤配置结构定义。
    - **`builder.rs`** - 解析配置并构建测试链。
    - **`engine.rs`** - `TestKind` 枚举与核心求值逻辑。
    - **`runner.rs`** - 过滤命令的执行主循环。
  - **`number.rs`** - 通用数字处理与格式化工具。
    - `format_number`: 支持千位分隔符与小数位控制。
  - **`io.rs`** - I/O 辅助函数。
    - 统一处理 stdin/stdout 和文件。
    - 透明处理 `.gz` 压缩/解压。
    - `InputSource`: 提供多文件统一视图。
  - **`sampling/`** - 高级采样算法。
    - **`reservoir.rs`**: 实现 Reservoir Sampling (蓄水池采样)。
    - **`bernoulli.rs`**: 实现 Bernoulli Sampling (Skip Sampling) - 几何分布跳过。
    - **`other.rs`**: 其他采样辅助。
  - **`tsv/`** - 核心 TSV 解析与处理模块。
    - **`reader.rs`** - 高性能零拷贝 TSV 读取器。
        - `TsvReader`: 管理内部缓冲区，支持行级迭代，避免字符串分配。
    - **`record.rs`** - 记录抽象。
        - `TsvRecord` / `TsvRow`: 实现 `Row` trait 的零拷贝访问。
    - **`fields.rs`** - 强大的字段选择逻辑。
        - 支持统一的字段语法: 数字索引, 名称匹配, 通配符等。
    - **`key.rs`** - Key 提取与处理。
        - `KeyExtractor`: 基于字段选择提取 Key。
        - `ParsedKey`: 优化的小 Key 存储 (`SmallVec`)。
    - **`select.rs`** - 列选择与重排引擎。
        - `SelectPlan`: 预计算字段映射计划。
        - `write_selected_from_bytes`: 基于计划的高性能零拷贝输出。
    - **`split.rs`** - 基于 SIMD 的字段切分工具。
        - `TsvSplitter`: 使用 `memchr` 快速迭代字段切片。

### 命令结构 (Command Structure)

每个命令在 `src/cmd_tva/` 中作为一个独立的模块实现，必须包含两个公开函数：

1.  **`make_subcommand`**: 定义命令行接口。
    -   返回 `clap::Command`。
    -   使用 `.about(...)` 设置简短描述。
    -   推荐使用 `.after_help(include_str!("../../docs/help/<cmd>.md"))` 引入详细文档。
2.  **`execute`**: 命令执行逻辑。
    -   接收 `&clap::ArgMatches`。
    -   返回 `anyhow::Result<()>`。

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

## 代码规范

- 使用 `cargo fmt` 格式化代码。
- 使用 `cargo clippy` 检查潜在问题。
- 优先使用标准库和项目中已引入的 crate (`csv`, `clap`, `anyhow`, `regex` 等)。
- 保持代码简洁，注重性能。

### 并发与并行策略 (Concurrency Strategy)

1.  **默认单线程 (Single-Threaded Default)**:
    *   `tva` 的核心命令默认采用单线程流式处理，以保证最低的启动开销和确定的内存使用。
    *   通过管道 (`|`) 组合命令可以让操作系统负责进程级并行。

2.  **未来并行方向 (Future Parallelism)**:
    *   对于 CPU 密集型任务 (如复杂 `filter` 或 `stats`)，架构应支持基于 "Chunking" 的数据并行。
    *   利用 TSV `\n` 的唯一性，将大文件切分为多个 `Chunk`，分发给线程池处理，最后聚合结果。

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
