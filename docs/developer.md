# 开发者指南

## 测试策略

我们复用上游 `tsv-utils` 的大量测试套件，以确保行为兼容性。

1.  Golden Tests (黄金测试):
    *   将 `tsv-utils/tests/` 中的输入文件和预期输出文件复制到 `tva/tests/data/<tool>/`。
2.  CLI 测试:
    *   在 `tests/cli_<tool>.rs` 中使用 `assert_cmd` 运行 `tva` 命令。
    *   将标准输出与上游的黄金输出进行比较。
3.  行为对齐:
    *   对于支持的标志，`tva` 旨在产生与 `tsv-utils` 完全相同的输出。
    *   有意的偏差（例如错误消息格式）会被单独记录和测试。

## 架构与模块

### 计划中的功能 (灵感来自 Datamash, R, 和 qsv)

*   扩展统计 (Extended Statistics):
    *   向 `stats` 添加 `q1` (25%), `q3` (75%), `iqr`, `skewness`, `kurtosis`。
*   缺失值填充 (Fill Missing Values):
    *   `fill`: 实现前向/后向填充以及常数填充。
*   索引机制 (Indexing Mechanism):
    *   现状: `tva` 目前主要是基于流的。
    *   参考: `qsv` 的核心优势是为 CSV 创建倒排索引 (`.idx` 文件)。这使得 GB 级文件可以瞬间完成 `slice`, `count` 和随机访问。
    *   提案: 考虑为 `tva` 引入可选的索引机制，特别是对于需要多次传递的大文件。
*   Apply 命令 (复杂转换):
    *   参考: `qsv apply` 支持基于字符串、日期、数学甚至 NLP（模糊匹配、情感分析）的列转换。
    *   提案: `tva` 的 `select` 目前倾向于选择。考虑增强其表达式能力，或添加 `apply` 命令来处理 `datefmt` (日期格式化) 和 `regex_replace`。
*   Tidyr 对等功能 (高级重塑):
    *   多度量透视 (Multi-measure Pivoting):
        *   `longer`: 支持在 `--names-to` 中使用 `.value` 哨兵，同时透视到多个值列（例如 `cols = c("x_1", "x_2", "y_1", "y_2")` -> `id, num, x, y`）。
        *   `wider`: 允许 `--values-from` 接受多个列，创建如 `val1_A`, `val1_B`, `val2_A`, `val2_B` 的输出列。
    *   列拆分/合并:
        *   `unpack`: 使用分隔符或正则将单个字符串列拆分为多个列（例如，将 "2023-10-27" 拆分为 "year", "month", "day"）。
        *   `pack`: 使用模板或分隔符将多个列合并为单个字符串列（例如，将 "Lat", "Lon" 合并为 "Coordinates"）。
    *   致密化 (Densification):
        *   `complete`: 暴露数据因子的缺失组合（显式缺失行）。
*   dplyr 核心模式:
    *   安全连接 (Safe Joins):
        *   概念: 防止 `join` 中意外的笛卡尔积爆炸。
        *   行动: 添加 `--relationship` 标志（例如 `one-to-one`, `many-to-one`）在连接时验证键。遇到意外的多对多匹配时默认为警告或错误。
    *   Tidy Selection DSL:
        *   概念: 解耦、表达力强的列选择逻辑。
        *   行动: 增强 `src/libs/fields.rs` 以支持正则 (`matches('^date_')`)、谓词 (`where(is_numeric)`) 和集合操作 (`-colA`)，可在 `select`, `wider`, `longer` 中通用。
    *   窗口函数 (Window Functions):
        *   概念: 上下文感知的行操作 (rank, lead, lag)。
        *   行动: 为 `filter` 和 `stats` 实现滑动窗口逻辑（例如，组内 `filter --expr "val > mean(val)"`）。
    *   高强度测试 (Torture Testing):
        *   概念: 针对畸形/边缘情况数据的鲁棒性。
        *   行动: 创建 `tests/torture/` 用于模糊测试输入（空文件、参差不齐的行、巨大的列），确保零 panic。

### 文档工作流

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

### 性能基准测试 (计划中)

我们旨在重现 [tsv-utils](https://github.com/eBay/tsv-utils/blob/master/docs/comparative-benchmarks-2017.md) 使用的严格基准测试策略。

#### 1. 基准工具
*   tsv-utils (D): 主要性能对标目标。
*   qsv (Rust): xsv 的活跃分支，功能超级强大。
*   GNU datamash (C): 统计操作的标准。
*   GNU awk / mawk (C): 行过滤和基本处理的基准。
*   csvtk (Go): 另一个现代跨平台工具包。

#### 2. 关键指标与数据集
我们将使用大数据集（GB 级）来压力测试流处理能力和内存使用。

*   **数据集来源**:
    *   **HEPMASS**: [UCI Machine Learning Repository](https://archive.ics.uci.edu/ml/datasets/HEPMASS)。约 4.8GB，700万行，29列数值数据。
    *   **FIA Tree Data**: [USDA Forest Service](https://apps.fs.usda.gov/fia/datamart/CSV/datamart_csv.html)。使用 `TREE_GRM_ESTN.csv` 的前 1400 万行 (约 2.7GB)，包含混合文本和数值。

*   **数值行过滤**: `tva filter --gt` vs `awk`。
    *   数据集: HEPMASS (4.8GB)。
*   **正则行过滤**: `tva filter --regex` vs `grep`/`awk`。
    *   数据集: FIA Tree Data (2.7GB)。
*   列选择: `tva select` vs `cut`。
    *   目标: 测量字段解析开销。
*   连接: `tva join` vs `join` (Unix) vs `qsv join`。
    *   场景: 大文件与子集文件基于公共键连接。
*   统计: `tva stats` vs `datamash`。
    *   操作: GroupBy + Sum/Mean/Min/Max。
*   CSV 转 TSV: `tva from csv` vs `qsv fmt`。
    *   目标: 测量解析速度和正确性（引号处理）。

#### 3. 执行环境
*   **工具**: 使用 [hyperfine](https://github.com/sharkdp/hyperfine) 进行自动化基准测试。它能自动处理预热、统计分析和异常值检测。
*   **配置**:
    *   预热运行: `--warmup 3`
    *   最小运行次数: `--min-runs 10`
    *   导出格式: `--export-markdown` 以便直接生成报告。
*   清晰记录硬件（CPU, RAM, 磁盘 I/O）和操作系统。
*   优化目标:
    *   流式命令的 O(1) 内存使用。
    *   尽可能零拷贝解析。
    *   高效的 I/O 缓冲（已使用 `BufReader`/`BufWriter`）。
