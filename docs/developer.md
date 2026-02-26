# 开发者指南

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

