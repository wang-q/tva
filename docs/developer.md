# 开发者指南

本文档旨在为 `tva` 的开发者提供技术背景、架构设计思路以及未来演进路线。

## changelog

```bash
git log v0.2.4..HEAD > gitlog.txt
git diff v0.2.4 HEAD -- "*.rs" "*.md" > gitdiff.txt

```

## code coverage

```bash
rustup component add llvm-tools
cargo install cargo-llvm-cov

# 生成覆盖率报告
cargo llvm-cov
```

使用 `cargo llvm-cov` 生成覆盖率报告，找出需要提升测试覆盖率的代码路径，供我分析。

XXX 的测试覆盖度不高，使用 `cargo llvm-cov` 生成覆盖率报告，找出需要提升的地方.

## 深度技术分析

### 参考项目: GNU Datamash

`datamash` 是命令行统计分析的标杆工具。`tva` 可以借鉴其在数据验证和交叉制表方面的设计。

#### 2. 交叉制表 (Crosstab / Pivot Table)
*   **功能**: `crosstab` 命令。
*   **特性**:
    *   计算两个分类变量之间的关系矩阵。
    *   支持 `count` (默认), `sum`, `unique` 等聚合操作。
*   **借鉴**: `tva` 目前通过 `wider` 实现类似功能，但 `crosstab` 作为一种专门的统计视图，其简洁性（自动处理行列标签）值得参考。

#### 3. 数值提取 (Numeric Extraction)
*   **功能**: `getnum` 操作。
*   **特性**: 从混合文本中提取数字（如 "zoom-123.45xyz" -> 123.45）。
*   **借鉴**: 这在处理脏数据时非常有用，可以作为 `filter` 或 `mutate` (待开发) 的一种转换函数。

#### 4. 丰富的统计指标
*   **特性**: 除了基础的 sum/mean，还支持：
    *   **稳健统计**: `mad` (中位数绝对偏差), `trimmean` (截尾均值)。
    *   **正态性检验**: `jarque` (Jarque-Bera 检验), `dpo` (D'Agostino-Pearson Omnibus 检验)。
    *   **偏度与峰度**: `sskew`/`pskew` (偏度), `skurt`/`pkurt` (峰度)。
    *   **协方差与相关**: `scov`/`pcov` (协方差), `spearson`/`ppearson` (Pearson 相关系数)。
    *   **众数相关**: `antimode` (反众数 - 最少出现的值)。
*   **借鉴**: `tva stats` 可以逐步补充这些高级统计指标，满足更专业的分析需求。

#### 5. 逐行转换操作 (Per-Line Operations)
*   **特性**: datamash 提供大量逐行转换操作，无需分组即可使用：
    *   **数值修约**: `trunc`, `frac`。
    *   **文件路径处理**: `dirname`, `basename`, `extname`, `barename`。
    *   **数值提取**: `getnum` 从混合文本中提取数字（如 "zoom-123.45xyz" -> 123.45）。
    *   **分箱**: `strbin` (字符串哈希分箱)。

#### 6. 示例文件组织
*   **特性**: datamash 的 `examples/` 目录包含：
    *   `scores.txt` / `scores_h.txt`: 成对的无表头/有表头示例。
    *   `genes.txt` / `genes_h.txt`: 真实生物信息学数据（UCSC Genome Browser）。
    *   `readme.md`: 详细解释每个示例的用法和场景。
*   **借鉴**: 
    *   为 `tva` 的 `docs/data/` 提供成对的示例文件（有/无表头）。
    *   添加真实领域数据：金融数据、日志数据、科学数据。
    *   编写 `docs/data/README.md` 详细说明示例用途。

### 参考项目: xan

`xan` (前身为 `xsv` 的 fork) 是一个功能极强的 CSV/TSV 工具集。通过分析其源码，我们可以为 `tva` 汲取以下几个关键的架构和功能灵感。

#### 1. 并行处理架构 (Parallel Processing)

*   **实现**: `cmd/parallel.rs`
*   **机制**: 类似于 Map-Reduce。它不试图让每个命令内部并行化，而是提供一个通用的 `parallel` 子命令。
    *   **Chunking**: 自动将文件分块，或按文件分发任务。
    *   **Shuffle**: 保证输出顺序与输入一致（如果需要）。
*   **对 `tva` 的启示**:
    *   `tva` 目前是单线程流式处理。
    *   **建议**: 实现 `tva parallel` 命令，负责将大文件切分 (利用 `split` 逻辑) 并启动多个子进程/线程处理，最后聚合结果。

#### 3. 近似算法 (Approximation)

*   **现状**: `tva` 目前所有计算（`nunique`, `median`）都是精确的，这意味着内存消耗随数据量线性增长 `O(N)`。
*   **借鉴**: `xan` 提供了近似算法支持：
    *   **基数估计**: `ApproxCardinality` 使用 **HyperLogLog (HLL)**，内存占用恒定。
    *   **分位数**: `ApproxQuantiles` (预计使用 T-Digest 或 KLL)，无需存储全量数据。
*   **行动项**: 针对大数据场景，引入 `--approx-unique` 和 `--approx-quantile` 选项。

#### 5. Join 架构对比: xan vs tva

通过分析 `xan/src/cmd/join.rs`，我们发现其设计哲学与 `tva` 截然不同。

| 特性 | xan join (SQL Style) | tva join (Stream Static) |
| :--- | :--- | :--- |
| **内存模型** | **全量加载 (Indexed Side)** | **部分加载 (Key + Append)** |
| **数据结构** | `Vec<IndexNode>` (Arena) + `HashMap` (Index) | `HashMap<KeyBuffer, Vec<u8>>` |
| **Join 类型** | Inner, Left, Right, Full, Cross (N-to-N) | Hash Semi-Join (N-to-1) |
| **多重匹配** | 支持 (通过链表 `next` 指针) | **不支持** (Last-Win 或 Error) |
| **Key 构建** | `ByteRecord` (Vector of Fields) | `KeyBuffer` (SmallVec<[u8; 32]>) |

*   **xan 的核心结构 (Arena + Linked List)**:
    ```rust
    struct IndexNode {
        record: ByteRecord, // 存储完整记录！内存占用大
        written: bool,      // 用于 Outer Join 标记
        next: Option<NonZeroUsize>, // 链表指针，解决 Hash 冲突和多重匹配
    }
    struct Index {
        map: HashMap<ByteRecord, (usize, usize)>, // Key -> (Head, Tail) in Arena
        nodes: Vec<IndexNode>, // Arena
    }
    ```
    *   **优势**: 支持完备的 SQL Join 语义 (包括 N-to-N 笛卡尔积)。
    *   **劣势**: 内存消耗巨大。Left Join 时需将整个 Right File 加载进内存；Full Join 时需将 Left File 加载进内存。Key 提取涉及 `ByteRecord` 的创建，有较多小内存分配。

*   **tva 的核心结构 (HashMap)**:
    ```rust
    // 仅存储 Key 和需要 Append 的字段
    // KeyBuffer 是 SmallVec<[u8; 32]>，优化小 Key 的内存分配
    let mut filter_map: HashMap<KeyBuffer, Vec<u8>> = HashMap::with_hasher(RandomState::new());
    ```
    *   **优势**: 极致的内存效率和速度。只存储必要数据。
    *   **劣势**: 仅支持 "查找并追加" 模式，无法处理 N-to-N 关系（Filter 文件中的 Key 必须唯一，否则需去重）。

#### 4. 随机访问与索引 (Random Access & Indexing)

*   **实现**: `src/config.rs` & `bgzip`
*   **机制**: 利用 `.gzi` 索引文件（BGZF 格式），支持不解压整个文件的情况下 Seek 到 Gzip 中间。
*   **对 `tva` 的启示**:
    *   对于大文件（GB/TB 级）的并行处理至关重要。
    *   **建议**: 处理超大压缩 TSV 时，支持 BGZF 索引是实现并行切片 (`slice`) 和随机采样 (`sample`) 的基础。

## 计划中的功能

### 数据重塑 (Data Reshaping) - Tidyr 对等功能

*   **多度量透视 (Multi-measure Pivoting)**:
    *   `longer`: 支持在 `--names-to` 中使用 `.value` 哨兵，同时透视到多个值列。
    *   `wider`: 允许 `--values-from` 接受多个列。
*   **列拆分/合并 (Column Splitting/Merging)**:
    *   `separate` (unpack): 使用分隔符或正则将单个字符串列拆分为多个列。
    *   `unite` (pack): 使用模板或分隔符将多个列合并为单个字符串列。
*   **行拆分 (Row Splitting)**:
    *   `separate-rows` (explode): 将包含分隔符的单元格拆分为多行 (e.g. "a,b" -> 2 rows)。
*   **致密化 (Densification)**:
    *   `complete`: 暴露数据因子的缺失组合，并支持填充默认值。
    *   `expand`: 仅生成唯一值的笛卡尔积（Cartesian Product），用于构建参考网格。
*   **行复制 (Row Replication)**:
    *   `uncount`: 根据计数列的值复制行（逆向 `count`）。
*   **缺失值处理 (Missing Values)**:
    *   `replace_na`: 将显式 `NA` (空字符串) 替换为指定值。
    *   `drop_na`: 丢弃包含缺失值的行。

### 数据操作 (Data Manipulation) - dplyr 核心模式

*   **安全连接 (Safe Joins)**:
    *   行动: 添加 `--relationship` 标志（例如 `one-to-one`, `many-to-one`）在连接时验证键。
*   **Tidy Selection DSL**:
    *   行动: 增强 `src/libs/fields.rs` 以支持正则 (`matches('^date_')`)、谓词 (`where(is_numeric)`) 和集合操作 (`-colA`)。
*   **窗口函数 (Window Functions)**:
    *   行动: 为 `filter` 和 `stats` 实现滑动窗口逻辑（例如，组内 `filter --expr "val > mean(val)"`）。
*   **高强度测试 (Torture Testing)**:
    *   行动: 创建 `tests/torture/` 用于模糊测试输入，确保零 panic。

### 借鉴 xan 的未来演进路线 (Future Roadmap: Lessons from xan)

通过对 `xan` 源码的深入分析，我们发现了几个极具价值的功能模块，值得 `tva` 在未来版本中借鉴或引入。

#### 1. Transform (列变换)
*   **功能**: `xan transform` 允许使用表达式（基于 `moonblade` 解释器）对列进行就地修改。例如 `xan transform surname 'upper(_)'`。
*   **价值**: `tva` 目前缺乏灵活的列处理能力。虽然 `awk` 可以胜任，但内置的 `transform` 可以提供更好的性能和更简便的语法（无需处理分隔符）。
*   **建议**: 引入轻量级表达式引擎（如 `rhai` 或简单的自定义解析器），实现类似 `tva apply` 或 `tva transform` 的命令，支持常见的字符串处理（upper, lower, trim, regex_replace）和数值计算。

#### 2. Search (高级搜索)
*   **功能**: `xan search` 远超简单的 `grep`。它支持：
    *   **多模式匹配**: 同时搜索数千个关键词（基于 Aho-Corasick 算法）。
    *   **模糊匹配**: `xan fuzzy-join` 和搜索支持基于 Levenshtein 距离的匹配。
    *   **替换**: 支持正则替换并输出到新列。
*   **价值**: 在数据清洗（ETL）场景中，批量关键词匹配和替换是刚需。
*   **建议**: 增强 `tva filter` 或新增 `tva search`，集成 `aho-corasick` crate 以支持高性能的多模式匹配。

## tva 与 xan 命令对比分析

通过对比 tva 和 xan 的命令集，以下是 tva 尚未实现但可能有价值的功能：

### 高优先级（建议尽快实现）

| 命令 | xan 对应 | 功能描述 | 价值 |
| :--- | :--- | :--- | :--- |
| **frequency** | `freq` | 频次统计表 | 快速了解数据分布，配合 `hist` 可视化 |
| **rename** | `rename` | 重命名列 | 数据清洗基础功能 |

### 低优先级（特定场景）

| 命令 | xan 对应 | 功能描述 | 价值 |
| :--- | :--- | :--- | :--- |
| **progress** | `progress` | 显示处理进度 | 大文件用户体验 |
| **search** | `search` | 多模式搜索（Aho-Corasick） | 高性能关键词搜索 |
| **separate** | `separate` | 使用正则拆分列 | 比 `unpack` 更灵活 |
| **window** | `window` | 滑动窗口计算 | 时间序列分析 |

### 暂不推荐实现

| 命令 | xan 对应 | 原因 |
| :--- | :--- | :--- |
| `input` | `input` | 功能简单，可用其他方式替代 |
| `parallel` | `parallel` | 架构复杂，可后期考虑 |


### Hist (直方图) 补充分析

除了 `plot`，`xan` 还提供了一个专门的 `hist` 命令 (`xan/src/cmd/hist.rs`)，用于绘制水平条形图（Horizontal Bar Charts）。

*   **定位差异**:
    *   `plot`: 通用绘图工具，支持散点图、折线图、垂直条形图，基于 `ratatui`，交互性强，适合复杂数据探索。
    *   `hist`: 专注于**频次分布可视化**，通常配合 `freq` 或 `bins` 命令使用。它不使用 `ratatui`，而是直接通过 Unicode 字符（如 `█`, `▌`）在标准输出打印，更轻量，适合管道操作。
*   **核心逻辑**:
    *   **数据模型**: 期望输入包含 `field` (可选), `value` (标签), `count` (数值) 三列。
    *   **渲染**: 手动计算每个条形的宽度，使用 `Scale` 进行线性或对数缩放，并处理颜色（Rainbow/Category/Stripes）。
    *   **特色功能**: 支持日期补全 (`--dates`)，自动填充缺失的日期并设为 0；支持间隙压缩 (`--compress-gaps`)，隐藏连续的 0 值。
