# 开发者指南

本文档旨在为 `tva` 的开发者提供技术背景、架构设计思路以及未来演进路线。

## changelog

```bash
git log v0.2.3..HEAD > gitlog.txt
git diff v0.2.3 HEAD -- "*.rs" "*.md" > gitdiff.txt

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

### 参考项目: rust-csv

`rust-csv` (BurntSushi/rust-csv) 是 Rust 生态中最权威的 CSV 解析库，也是 `tva` 的核心依赖之一。对其源码的分析有助于指导 `tva` 的底层优化和功能扩展。

**`csv-index` (索引机制)**:
    *   **定位**: 提供 CSV 文件的随机访问能力。
    *   **实现**: 创建辅助的索引文件（通常是 `.idx`），记录每行（或每块）的字节偏移量。
    *   **价值**: 这是 `qsv` 能够瞬间完成切片 (`slice`) 和统计计数 (`count`) 的秘密武器。
    *   **对 `tva` 的建议**:
        *   目前 `tva` 是纯流式的，这对于单次扫描非常高效。
        *   但对于需要多次扫描或随机访问大文件的场景（如 `sample --random-access` 或大文件 `slice`），引入 `csv-index` 是实现性能飞跃的关键。
        *   **行动项**: 研究将 `csv-index` 集成到 `tva` 的 `input` 层，允许用户为大文件生成索引，从而加速后续操作。

### 参考项目: simd-csv

`simd-csv` (medialab/simd-csv) 是一个专门利用 SIMD 指令集加速 CSV 解析的 Rust crate。它并非 C++ `simdjson` 的直接移植，而是采用了混合传统状态机与 SIMD 字符串搜索的新颖方法。

#### 核心特性与架构

1.  **混合架构 (Hybrid Approach)**:
    *   结合了传统状态机逻辑与 `memchr` 风格的 SIMD 字符串搜索。
    *   **优势**: 相比纯状态机，在大段文本或长字段处理上极其高效。
    *   **劣势**: 性能提升高度依赖数据形态。对于短字段、密集数字的数据，性能可能回退到与标量代码持平。

2.  **多层级 Reader API**:
    *   **`Splitter`**: 仅查找记录分隔符。极其适合 `wc -l` (行计数) 或并行切分场景。
    *   **`ZeroCopyReader`**: 仅查找单元格分隔符，不进行转义。适合不需要处理引号内容的快速过滤/选择。
    *   **`Reader`**: 完整的流式解析器，支持转义。API 尽量模仿 `csv` crate，但更底层。

3.  **流式支持**:
    *   与某些 SIMD 解析器要求全量加载不同，此 crate 明确支持流式处理 (`streaming`)，这使其成为 `tva` 的潜在高性能后端候选。

### 参考项目: GNU Datamash

`datamash` 是命令行统计分析的标杆工具。`tva` 可以借鉴其在数据验证和交叉制表方面的设计。

#### 1. 结构验证 (Check / Validation)
*   **功能**: `check` 命令。
*   **特性**:
    *   **Fail-fast**: 在管道中尽早发现格式错误（如字段数不一致）。
    *   **上下文报错**: 报错时提供具体的行号和内容。
*   **借鉴**: `tva` 目前已通过 `check` 命令实现了核心的结构一致性检查，具备 Fail-fast 和上下文报错特性，满足大多数数据验证需求。Datamash 的显式行/列数断言可作为未来可选的增强功能。

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
    *   **分布检验**: `jarque` (正态性检验), `dpo` (Omnibus 检验)。
    *   **偏度与峰度**: `sskew`/`pskew` (偏度), `skurt`/`pkurt` (峰度)。
    *   **协方差与相关**: `scov`/`pcov` (协方差), `spearson`/`ppearson` (Pearson 相关系数)。
*   **借鉴**: `tva stats` 可以逐步补充这些高级统计指标，满足更专业的分析需求。

#### 5. 逐行转换操作 (Per-Line Operations)
*   **特性**: datamash 提供大量逐行转换操作，无需分组即可使用：
    *   **数值修约**: `round`, `floor`, `ceil`, `trunc`, `frac`。
    *   **哈希与编码**: `base64`, `debase64`, `md5`, `sha1`, `sha256` 等。
    *   **文件路径处理**: `dirname`, `basename`, `extname`, `barename`。
    *   **数值提取**: `getnum` 从混合文本中提取数字（如 "zoom-123.45xyz" -> 123.45）。
    *   **分箱**: `bin` (数值分箱), `strbin` (字符串哈希分箱)。
*   **借鉴**: 这些操作可以作为 `tva` 的新命令或 `transform` 命令的功能。

#### 6. 示例文件组织
*   **特性**: datamash 的 `examples/` 目录包含：
    *   `scores.txt` / `scores_h.txt`: 成对的无表头/有表头示例。
    *   `genes.txt` / `genes_h.txt`: 真实生物信息学数据（UCSC Genome Browser）。
    *   `readme.md`: 详细解释每个示例的用法和场景。
*   **借鉴**: 
    *   为 `tva` 的 `docs/data/` 提供成对的示例文件（有/无表头）。
    *   添加真实领域数据：金融数据、日志数据、科学数据。
    *   编写 `docs/data/README.md` 详细说明示例用途。

#### 7. 命令语法设计
*   **特性**: datamash 使用简洁的位置参数语法：
    ```bash
    datamash [options] op1 column1 [op2 column2 ...]
    # 例如: datamash -g 2 mean 3 pstdev 3
    ```
*   **借鉴**: 
    *   考虑为 `tva stats` 添加位置参数语法支持作为替代方案。
    *   提供操作别名（如 `uniq` 作为 `unique` 的别名）。

#### 8. 文档与帮助系统
*   **特性**: 
    *   Texinfo 格式文档，可生成 info/HTML/PDF 多种格式。
    *   详细的 man page，包含数学公式和渐进式示例。
    *   每个统计操作都有与 R 函数的对应关系说明。
*   **借鉴**: 
    *   为 `tva` 的统计操作添加数学公式说明。
    *   添加与其他工具（R、Python pandas）的对比。
    *   在帮助文档中提供从简单到复杂的渐进式示例。

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

#### 3. 表达式引擎 (Expression Engine)

*   **实现**: `src/moonblade`
*   **机制**: 内置基于 `pest` 的解释器，支持类似 Excel 的表达式。
*   **对 `tva` 的启示**:
    *   `tva` 目前的 `filter` 和 `select` 逻辑是硬编码的。
    *   **建议**: 未来可引入轻量级表达式引擎（如 `rhai` 或手写递归下降解析器）以支持复杂计算（如 `if(a>0, b, c)`）。

#### 4. 随机访问与索引 (Random Access & Indexing)

*   **实现**: `src/config.rs` & `bgzip`
*   **机制**: 利用 `.gzi` 索引文件（BGZF 格式），支持不解压整个文件的情况下 Seek 到 Gzip 中间。
*   **对 `tva` 的启示**:
    *   对于大文件（GB/TB 级）的并行处理至关重要。
    *   **建议**: 处理超大压缩 TSV 时，支持 BGZF 索引是实现并行切片 (`slice`) 和随机采样 (`sample`) 的基础。

## 计划中的功能

### 核心功能增强 (Core Enhancements)

*   **索引机制 (Indexing Mechanism)**:
    *   现状: `tva` 目前主要是基于流的。
    *   提案: 考虑为 `tva` 引入可选的索引机制（参考 `qsv` 的 `.idx`），特别是对于需要多次传递的大文件，以支持瞬间切片和随机访问。
*   **Apply 命令 (复杂转换)**:
    *   参考: `qsv apply` 支持基于字符串、日期、数学甚至 NLP 的列转换。
    *   提案: 增强 `select` 的表达式能力，或添加 `apply` 命令处理 `datefmt` 和 `regex_replace`。

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
    *   `fill`: 支持向上/向下填充 (LOCF/NOCB) - **已实现向下填充**。
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

### 扩展统计 (Extended Statistics)

基于 GNU Datamash 的分析，`tva stats` 可以逐步添加以下统计指标：

#### 高优先级
*   **稳健统计**: `trimmean` (截尾均值), `madraw` (未缩放的中位数绝对偏差)。

#### 中优先级
*   **分布形态**: `sskew`/`pskew` (样本/总体偏度), `skurt`/`pkurt` (样本/总体峰度)。
*   **正态性检验**: `jarque` (Jarque-Bera 检验), `dpo` (D'Agostino-Pearson Omnibus 检验)。
*   **相关与协方差**: `scov`/`pcov` (样本/总体协方差), `spearson`/`ppearson` (Pearson 相关系数), `dotprod` (点积)。

#### 低优先级
*   **百分位数**: `perc` (自定义百分位数, 默认 95%)。
*   **众数相关**: `antimode` (反众数 - 最少出现的值)。

### 逐行转换命令 (Transform / Apply)

参考 GNU Datamash 的 Per-Line Operations，添加 `transform` 或 `apply` 命令支持：

#### 高优先级
*   **数值修约**: `round`, `floor`, `ceil`, `trunc`, `frac` (取小数部分)。
*   **数值提取**: `getnum` 从混合文本中提取数字（如 "price-$123.45" -> 123.45）。

#### 中优先级
*   **哈希与编码**: `base64`/`debase64`, `md5`, `sha1`, `sha256`, `sha512`。
*   **文件路径处理**: `dirname`, `basename`, `extname`, `barename` (无扩展名的文件名)。

#### 低优先级
*   **字符串分箱**: `strbin` 将字符串哈希到固定数量的桶中（用于数据分片）。

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

#### 3. Cluster (聚类/重复检测)
*   **功能**: `xan cluster` 基于键值冲突（Key Collision）或指纹算法（Fingerprinting）来发现相似的行。
*   **价值**: 这是数据质量分析的高级功能，用于发现拼写错误（如 "Apple Inc." vs "Apple Inc"）。
*   **建议**: 作为一个高级功能，可以考虑在未来版本中引入，用于数据去重和清洗。

#### 4. Vocabulary (词汇表管理)
*   **功能**: `xan vocab` 用于管理文本数据的词汇表，支持文档-词项矩阵（Document-Term Matrix）的生成。
*   **价值**: 对于NLP预处理非常有用。
*   **建议**: 虽然 `tva` 定位于通用数据处理，但如果目标用户包含大量文本分析需求，这是一个值得考虑的方向。

---

## tva plot 子命令未来扩展

以下 plot 子命令已实现：
*   **散点图**: `tva plot point` - 支持散点、折线、路径、回归线、按颜色分组
*   **箱线图**: `tva plot box` - 支持分组、多列、异常值显示
*   **二维分箱热力图**: `tva plot bin2d` - 自动分箱，字符密度表示计数

**其他 plot 子命令计划**:
*   **直方图**: 增加 `tva plot hist` 命令（水平条形图）

### Hist (直方图) 补充分析

除了 `plot`，`xan` 还提供了一个专门的 `hist` 命令 (`xan/src/cmd/hist.rs`)，用于绘制水平条形图（Horizontal Bar Charts）。

*   **定位差异**:
    *   `plot`: 通用绘图工具，支持散点图、折线图、垂直条形图，基于 `ratatui`，交互性强，适合复杂数据探索。
    *   `hist`: 专注于**频次分布可视化**，通常配合 `freq` 或 `bins` 命令使用。它不使用 `ratatui`，而是直接通过 Unicode 字符（如 `█`, `▌`）在标准输出打印，更轻量，适合管道操作。
*   **核心逻辑**:
    *   **数据模型**: 期望输入包含 `field` (可选), `value` (标签), `count` (数值) 三列。
    *   **渲染**: 手动计算每个条形的宽度，使用 `Scale` 进行线性或对数缩放，并处理颜色（Rainbow/Category/Stripes）。
    *   **特色功能**: 支持日期补全 (`--dates`)，自动填充缺失的日期并设为 0；支持间隙压缩 (`--compress-gaps`)，隐藏连续的 0 值。


## 深度模块分析：Tidyr 核心功能移植

本节详细规划如何将 R 语言 `tidyr` 包的核心数据清洗功能移植到 `tva`，重点关注如何在保持流式处理优势的同时实现这些功能。

### 1. `unpack` & `pack` (拆分与合并)

*   **`unpack` (Split Column)**:
    *   **对应**: `tidyr::separate`
    *   **逻辑**: 将一列按分隔符或正则拆分为多列。
    *   **流式实现**: 读取一行 -> 找到目标列 -> split -> 插入新列 -> 输出。完全流式。
    *   **难点**: 拆分后的列数不一致怎么办？`tidyr` 提供了 `extra = "warn" | "drop" | "merge"`。`tva` 应默认 `error` 或 `warn`，支持 `fill` (填充右侧) 或 `drop` (截断)。

*   **`pack` (Merge Columns)**:
    *   **对应**: `tidyr::unite`
    *   **逻辑**: 将多列合并为一列。
    *   **流式实现**: 读取一行 -> 提取目标列 -> join -> 替换/追加 -> 输出。完全流式。

*   **API 设计**:
    ```bash
    tva unpack -f 2 --sep "," --into "A,B,C"   # Split col 2 into A, B, C
    tva pack -f 1-3 --sep "-" --into "ID"       # Merge col 1-3 into ID
    ```

### 2. `complete` & `expand` (补全组合)

这是一个典型的 "非流式" 操作，因为它需要知道所有唯一值的集合。

*   **实现策略**:
    *   必须先扫描全表（或使用外部 `sort | uniq`）获取因子级别的唯一值。
    *   构建笛卡尔积。
    *   Left Join 原数据。
*   **建议**: 作为一个高级命令，初期可以不实现，或者仅支持基于预定义字典的 `expand`。
