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
- 使用 `assert_cmd` 或类似的库来测试 CLI 调用和输出验证。

示例测试结构：

```rust
// tests/cli_example.rs

use std::process::Command;
use assert_cmd::prelude::*;

#[test]
fn test_example_basic() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("tva")?;
    cmd.arg("example").arg("input.tsv");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("expected output"));

    Ok(())
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
    *   **缓冲**: 所有文件 I/O 必须包裹在 `BufReader` / `BufWriter` 中（建议缓冲区大小 >= 64KB）。

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


## 帮助文本规范 (Help Text Style Guide)

* **`about`**: Third-person singular, describing the TSV operation
  (e.g., "Converts TSV to markdown table", "Deduplicates TSV rows").
* **`after_help`**: Use raw string `r###"..."###`.
    * **Description**: Short paragraph of what the subcommand does and its trade-offs.
    * **Notes**: Bullet points starting with `*`.
        * TSV input: `* Supports plain text and gzipped (.gz) TSV files`
        * Stdin behavior:
            * Single-input tools (e.g. `md`): `* Reads from stdin if input file is 'stdin' or no input file is given`
            * Multi-input tools (e.g. `uniq`): `* Reads from stdin if no input file is given or if input file is 'stdin'`
        * Memory-heavy tools (e.g. `uniq`): `* Keeps a hash for each unique row; does not count occurrences`
    * **Examples**: Numbered list (`1.`, `2.`) with code blocks indented by 3 spaces.
* **Arguments**:
    * **Input**: `infile` (single) or `infiles` (multiple).
        * Help (single): `Input TSV file to process (default: stdin).`
        * Help (multiple): `Input TSV file(s) to process`.
    * **Output**: `outfile` (`-o`, `--outfile`).
        * Help: `Output filename. [stdout] for screen`.
* **Terminology**:
    * Prefer "TSV" when referring to files.
    *   Use "row" / "column" in help text where it makes sense.

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
