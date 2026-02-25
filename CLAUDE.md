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
  - **`fields.rs`** - 字段选择逻辑 (支持索引、范围、名称)。
    - Implements the unified field syntax: Numeric intervals (`1,3-5`), header-aware selection (`user_id`, wildcards `*_time`, ranges `start_col-end_col`).
    - Used by `select`, `join`, `uniq`, `stats`, `sample`, etc.
  - **`io.rs`** - I/O 辅助函数 (stdin/stdout, gzip 等)。
    - `reader`: Handles `stdin`, files, and `.gz` decompression transparently.
    - `InputSource`: Provides a unified view for iterating over multiple input files.
  - **`number.rs`** - 数字格式化 (千位分隔符等)。
    - Ensures consistent printing of floating-point numbers across tools.
    - Implements R-compatible quantile calculations (`stats`).
  - **`stats.rs`** - 统计计算核心逻辑。
  - **`filter.rs`** - 过滤逻辑。
  - **`sampling.rs`** - 随机采样相关逻辑。

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
