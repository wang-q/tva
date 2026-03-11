# 开发者指南

本文档旨在为 `tva` 的开发者提供技术背景、架构设计思路以及未来演进路线。

## changelog

```bash
git log v0.2.3..HEAD > gitlog.txt
git diff v0.2.3 HEAD -- "*.rs" "*.md" > gitdiff.txt

```

## 性能基准与分析

我们在 `benches/parse_benchmark.rs` 中对比了不同解析策略的性能。
数据样本: `1\tJohn\tDoe\t30\tNew York\n...` (3行数据重复 1000 次)

| 策略 | 平均耗时 | 吞吐量 | 说明 |
| :--- | :--- | :--- | :--- |
| **simd-csv** | **67 µs** | **1.05 GiB/s** | 混合 SIMD 状态机，性能天花板 |
| **Tva TsvReader** | **72 µs** | **1.00 GiB/s** | **当前实现**: 零拷贝 Reader + SIMD (memchr) |
| **Memchr Reused Buffer** | 82 µs | 878 MiB/s | 逐行 memchr，受限于函数调用开销 |
| **csv crate** | 111 µs | 652 MiB/s | 经典的 DFA 状态机，正确性基准 |
| **Naive Split** | 443 µs | 163 MiB/s | 原始实现，最慢 |

**结论**:
1.  **性能飞跃**: 我们的新实现 `Tva TsvReader` (1.00 GiB/s) 比旧版快了 **2.6 倍**，已达到 `simd-csv` 的 **95%**。
2.  **瓶颈转移**: 瓶颈已从 "内存分配/IO" 转移到了 "字段遍历"。

---

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

---

## 自研 TSV 解析器设计

鉴于 TSV (Tab-Separated Values) 的格式极其简单，我们可以实现一个专用、高性能的 TSV 解析器。

### 1. 格式差异分析

| 特性 | CSV (RFC 4180) | TSV (Simple) | 影响 |
| :--- | :--- | :--- | :--- |
| **分隔符** | `,` (可变) | `\t` (固定) | TSV 可硬编码分隔符，利于 SIMD 优化。 |
| **引号** | 支持 `"` 包裹字段 | **不支持** | TSV 无需维护 "in_quote" 状态，彻底消除状态机分支预测失败。 |
| **转义** | `""` 转义引号 | 无 (或 C 风格 `\t`) | TSV 无需处理 `""` -> `"` 的内存拷贝/重写，支持真正的零拷贝切片。 |
| **换行** | 字段内可含换行 | **不允许** | TSV 保证 `\n` 永远代表记录结束。可并行分块查找 `\n`。 |

### 2. 自研 TSV 解析器设计思路

目标：实现比 `rust-csv` 快，且比 `simd-csv` 更轻量的专用 TSV 解析。

1.  **纯 SIMD 扫描 (Pure SIMD Scanning)**:
    *   因为不需要处理引号，我们可以盲目地使用 SIMD (如 `memchr` crate 或 `std::simd`) 查找 `\t` 和 `\n`。
    *   无需回溯，无需状态维护。
2.  **绝对零拷贝 (Absolute Zero-Copy)**:
    *   CSV 解析器在遇到转义引号时必须进行内存拷贝 (`Cow<str>`)。
    *   TSV 解析器永远可以返回原始 buffer 的切片 (`&str` 或 `&[u8]`)，完全避免内存分配。
3.  **行级并行 (Line-Level Parallelism)**:
    *   由于 `\n` 在 TSV 中具有唯一语义 (记录分隔符)，我们可以安全地将大文件切分为多个 chunks，并行寻找 `\n` 进行对齐，然后并行解析。这在 CSV 中很难做到 (因为 `\n` 可能在引号内)。
4.  **API 设计**:
    ```rust
    // 概念验证 API
    struct TsvReader<'a> {
        buffer: &'a [u8],
    }

    impl<'a> TsvReader<'a> {
        // 返回字段迭代器，无需分配
        fn fields(&self) -> impl Iterator<Item = &'a [u8]> {
            // 使用 memchr::memchr_iter 快速查找 \t
            memchr::memchr_iter(b'\t', self.buffer).map(...)
        }
    }
    ```

### 3. 我们可以超越它吗?

*   **TSV 的优势**: TSV **没有引号**。这意味着我们不需要像 `simd-csv` 那样在 `memchr` 命中后还要检查是否是引号，也不需要维护 `Quoted` 状态。
*   **更简单的 SIMD**: 我们只需要查找 `\t` 和 `\n`。这比 CSV 的 3-4 个特殊字符更少，寄存器压力更小。
*   **理论极限**: 如果 `simd-csv` 要处理引号逻辑还能跑 1.12 GiB/s，那么纯粹查找 `\t` 和 `\n` 的 TSV 解析器理论上应该能达到内存带宽的极限（或至少 2-3 GiB/s）。

**行动项**:
我们不需要复杂的混合状态机。我们只需要一个极致优化的 `memchr2(b'\t', b'\n')` 循环，配合 Buffer 管理。我们的 `Memchr Reused Buffer` (814 MiB/s) 已经验证了这一点，差距仅在于 `simd-csv` 可能使用了更底层的 SIMD 优化或更高效的 I/O 缓冲。

---

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

## 深度模块分析：Plot (可视化)

`xan` 的 `plot` 命令 (`xan/src/cmd/plot.rs`) 是一个功能强大的终端绘图工具，基于 `ratatui` 库实现。通过分析其源码，我们可以了解如何在终端中实现高质量的数据可视化。

### 1. 核心流程

`plot` 命令的处理流程可以概括为以下几个阶段：

1.  **参数解析与配置**:
    *   使用 `clap` 解析大量绘图选项（如 `--line`, `--bars`, `--time`, `--category` 等）。
    *   处理颜色模式 (`--color`) 和输入配置 (`Config`)。
    *   推断终端尺寸 (`cols`, `rows`) 和刻度数量 (`ticks`)。

2.  **数据采集 (Data Collection)**:
    *   **列选择**: 确定 x 轴、y 轴（可能多个）和分类列 (`category`)。
    *   **Series 构建**: 使用 `SeriesBuilder` 动态构建数据系列。支持三种模式：
        *   `Single`: 单一数据系列。
        *   `Multiple`: 多列 y 值（如 `plot x y1 y2`）。
        *   `Categorical`: 基于分类列分组（如 `plot x y -c category`）。
    *   **数据解析**: 流式读取 CSV，利用 `fast_float` 解析数值，利用 `jiff` 解析时间戳。支持 `--ignore` 忽略错误。

3.  **数据处理 (Data Processing)**:
    *   **时间聚合**: 如果启用了 `--time` (`-T`)，会对 x 轴进行时间粒度推断 (`infer_temporal_granularity`) 和聚合计算 (`GroupAggregationProgram`)。
    *   **域 (Domain) 计算**: 动态计算 x 和 y 的最大最小值 (`extent`)。
    *   **排序**: 如果是折线图 (`--line`)，强制按 x 轴排序。

4.  **布局与渲染 (Layout & Rendering)**:
    *   **坐标轴推断**: `AxisInfo` 结构体负责根据所有 Series 的范围统一计算坐标轴的刻度和类型（Int, Float, Timestamp）。
    *   **Ratatui 绘图**:
        *   使用 `print_ratatui_frame_to_stdout` 将 TUI 帧直接输出到标准输出。
        *   支持 **Small Multiples (小多图)**: 如果指定了 `-S`，会使用 `Layout` 将屏幕分割成网格，并在每个格子里独立绘图。
        *   **Canvas Patching**: `patch_buffer` 函数直接修改 `ratatui` 的底层 Buffer，用于绘制更精细的网格线 (`┼`, `─`, `│`) 和刻度，这是标准 `Chart` widget 难以做到的。

### 2. 关键技术点

*   **混合坐标轴类型**: `AxisType` 枚举处理 `Int`, `Float`, `Timestamp` 的混合逻辑，确保不同系列能共享坐标轴。
*   **Liang-Barsky 裁剪**: 实现了 `clip` 算法，用于计算回归线在图表边界内的端点。
*   **鲁棒的日期解析**: 组合了 `jiff` 的多种解析能力 (`parse_zoned`, `parse_partial_date`)，支持多种时间格式。
*   **终端自适应**: 能够根据终端大小自动调整图表尺寸和刻度密度。

### 3. tva plot 子命令未来扩展

`plot point` 已实现。以下是计划中的其他 plot 子命令：

#### 3.1 plot point 未来扩展

参考 ggplot2 `geom_point()` 的美学映射（Aesthetic Mappings）设计，以下是 `plot point` 可以改进的方向：

**待实现的美学映射（按优先级）**:

*中优先级*:
*   **统计变换 `stat`**: 添加 `--stat <smooth>` 支持回归线（`geom_smooth` 等价物）

**非美学功能改进**:

*中优先级*:
*   **回归线**: 添加`--regression` 参数叠加回归线
*   **置信区间**: 回归线时显示置信区间阴影

**其他 plot 子命令计划**:
*   **直方图**: 增加 `tva plot hist` 命令（水平条形图）
*   **箱线图**: 增加 `tva plot box` 命令

### 4. Hist (直方图) 补充分析

除了 `plot`，`xan` 还提供了一个专门的 `hist` 命令 (`xan/src/cmd/hist.rs`)，用于绘制水平条形图（Horizontal Bar Charts）。

*   **定位差异**:
    *   `plot`: 通用绘图工具，支持散点图、折线图、垂直条形图，基于 `ratatui`，交互性强，适合复杂数据探索。
    *   `hist`: 专注于**频次分布可视化**，通常配合 `freq` 或 `bins` 命令使用。它不使用 `ratatui`，而是直接通过 Unicode 字符（如 `█`, `▌`）在标准输出打印，更轻量，适合管道操作。
*   **核心逻辑**:
    *   **数据模型**: 期望输入包含 `field` (可选), `value` (标签), `count` (数值) 三列。
    *   **渲染**: 手动计算每个条形的宽度，使用 `Scale` 进行线性或对数缩放，并处理颜色（Rainbow/Category/Stripes）。
    *   **特色功能**: 支持日期补全 (`--dates`)，自动填充缺失的日期并设为 0；支持间隙压缩 (`--compress-gaps`)，隐藏连续的 0 值。

### 4. 频率与分箱 (Freq & Bins)

`xan` 的可视化能力通常依赖于预处理命令 `frequency` (或 `freq`) 和 `bins`，它们生成 `hist` 所需的格式化数据。

*   **Frequency (`freq`)**:
    *   **功能**: 计算离散值的频次表（Top-K）。
    *   **实现**: 使用 `Counter` (基于 Hashmap) 进行计数。支持并行计算 (`--parallel`)，利用 `rayon` 加速。
    *   **关键点**: 支持近似算法 (`--approx`)，在内存受限时使用 Space-Saving 算法计算 Top-K。
*   **Bins (`bins`)**:
    *   **功能**: 将连续数值离散化为桶（分箱）。
    *   **实现**: 支持多种分箱启发式算法 (`freedman-diaconis`, `sturges`, `sqrt`) 自动确定最佳桶数。
    *   **关键点**: `LinearScale::nice` 用于生成人类可读的桶边界（如 0-10, 10-20 而非 0-9.87, 9.87-19.74）。

### 5. Bin vs Bins: 两种分箱哲学

`tva` 的 `bin` 命令目前借鉴了 GNU `datamash` 的设计，而 `xan` 的 `bins` 则展现了完全不同的思路。

| 特性 | tva bin (Datamash Style) | xan bins (Auto/Nice) |
| :--- | :--- | :--- |
| **核心逻辑** | `floor((val - min) / width) * width + min` | 自动计算桶数 + "Nice" 刻度 |
| **输入** | **流式 (Streaming)** | **全量加载 (In-Memory)** |
| **参数** | 必须指定 `--width` (桶宽) | 指定 `--bins` (桶数) 或启发式算法 |
| **输出** | 替换/追加原列数值 (Row-wise) | 生成新的统计表 (Summary Table) |
| **用途** | 数据清洗/ETL (如将年龄 23 变为 20) | 数据探索/直方图预处理 |

*   **tva 的优势**: 极快，恒定内存，适合作为中间步骤（例如 `bin` 后再 `groupby`）。
*   **xan 的优势**: 智能，无需用户猜测 `width`，直接生成可视化友好的统计结果（包含 `lower_bound`, `upper_bound`, `count`）。
*   **借鉴意义**:
    *   **启发式算法**: `xan` 实现了 `Freedman-Diaconis`, `Sturges`, `Sqrt` 等算法来自动推断最佳桶数，这对数据探索非常有价值。
    *   **Nice Scaling**: `LinearScale::nice` 算法能生成人类可读的边界（10, 20, 30），而不是数学上精确但丑陋的边界（10.123, 20.246）。
    *   **未来方向**: `tva` 可以保留现有的流式 `bin` 用于 ETL，但可以考虑增加一个聚合模式（或单独的 `histogram` 命令）来吸纳 `xan` 的自动分箱能力。


## 聚合架构深度对比 (Aggregation Architecture)

本节深入剖析 `tva` 与 `xan` (Rust) 以及 `tsv-utils` (D Language) 在聚合模块设计上的核心差异与权衡。

### 1. 架构模式对比

| 特性 | tva | xan | tsv-utils | datamash | dplyr | qsv |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| **模式** | **SoA + Dyn Trait** | **Enum Dispatch** | **Template Strategy** | **Stateful Struct** | **Masking** | **Hybrid** |
| **核心** | `Vec<f64>` + `Box<dyn>` | `enum Agg` | `Summarizer` | `struct op` | `eval(mask)` | `Stats+Idx` |
| **内存** | 极致紧凑 (SoA) | 碎片化 (Enum) | AoS (Struct) | O(1) (Streaming) | 向量化 | 混合 |
| **GroupBy** | **Stable Hash (IndexMap)** | Hash Map | **Stable Hash (DList)** | **Sort-based** | Vectorized | Parallel |
| **亮点** | 模块化+低编译时间 | 强类型 | 极致特化性能 | 高精度 | 避免小对象 | 缓存+并行 |

### 2. 详细分析

#### A. tva: 运行时多态与内存布局的平衡 (Runtime Polymorphism)
*   **设计**: **Hybrid SoA**。Schema (`StatsProcessor`) 在运行时构建计算图，State (`Aggregator`) 采用紧凑的列式存储 (`Vec<f64>`, `Vec<String>`)。计算逻辑通过 `Box<dyn Calculator>` trait 动态分发。
*   **优势**:
    *   **内存友好**: 即使有数百万个分组，每个分组的 `Aggregator` 开销也极低（仅分配几个 `Vec` 头部）。
    *   **模块化**: 新增算子只需实现 `Calculator` trait，完全解耦。
    *   **编译速度**: 相比泛型/模板膨胀，`dyn Trait` 显著减少了编译时间和二进制大小。
    *   **确定性**: 使用 `IndexMap` 保证了 GroupBy 输出顺序与输入首次出现顺序一致。
*   **权衡**: 虚函数调用 (`vtable`) 在极高频循环中（如每行调用 10 次）相比内联代码有微小开销，但在 I/O 主导的 CLI 工具中通常可忽略。

#### B. xan: 枚举分发 (Enum Dispatch)
*   **设计**: 每个统计指标是一个独立的结构体，通过巨大的 `enum Agg { Sum(SumState), Mean(MeanState), ... }` 进行分发。
*   **优势**: 避免了堆分配 (`Box`) 和虚函数调用；利用 Rust 的 `match` 优化。
*   **短板**: `enum` 的大小取决于最大的成员；添加新算子需要修改核心枚举定义（侵入式）；GroupBy 时每个状态是独立对象，内存碎片较多。

#### C. tsv-utils (D Language): 编译时模板特化 (Compile-time Polymorphism)
*   **模式**: **Template-based Strategy Pattern**
*   **核心**: `SummarizerBase` -> `NoKeySummarizer` / `OneKeySummarizer` / `KeySummarizerBase`。
*   **亮点**:
    *   **极致性能**: 利用 D 语言强大的模板元编程，在编译时生成针对特定列类型的代码路径（特化），完全消除运行时检查。
    *   **稳定排序**: 使用 `DList` (双向链表) 维护插入顺序。
*   **短板**: 编译时间长；代码复杂度极高（大量模板样板代码）。

#### D. datamash (C Language): Sort-based Streaming
*   **模式**: **Stateful Fat Struct**
*   **核心**: `struct fieldop` (包含 Schema 和 State)。
*   **策略**: **Sort-based Grouping**。依赖输入已排序，每次只处理一个组，处理完即重置。
*   **优势**: 内存占用极低 (O(1) Groups)；使用 `long double` 提供更高精度的累加。
*   **劣势**: 必须预先排序 (O(N log N))；无法处理无序数据的 Hash GroupBy。

#### E. dplyr (R/C++): Hybrid Mask Evaluation
*   **模式**: **Vectorized Evaluation (Masking)**
*   **核心**: `dplyr_mask_eval_all_summarise` (src/summarise.cpp)。
*   **策略**: 不为每个组创建单独的 Aggregator 对象。相反，它利用 R 的向量化特性，通过 **Mask (位掩码)** 迭代每个组，对整个向量切片进行求值。
*   **优势**: 极致利用 R 的向量化生态；避免了成千上万个小对象的创建开销。
*   **劣势**: 极其依赖列式存储（Columnar Store），不适合流式处理（Row-based Streaming）。

#### F. qsv (Rust): Caching & Parallelism
*   **模式**: **Hybrid (Streaming + Indexed)**
*   **策略**:
    *   **缓存优先**: 将 `stats` 结果缓存为 `<file>.stats.csv`，后续命令 (如 `frequency`) 直接读取，极大提升工作流效率。
    *   **索引加速**: 如果检测到 `.idx` 文件，自动启用多线程并行计算 (Parallel Chunking)。
    *   **类型推断**: 结合全量扫描进行严格的类型推断 (Integer/Float/Date/String)，而非简单的采样。
*   **优势**: 在重复分析场景下无敌；支持并行加速。
*   **劣势**: 代码逻辑复杂，需处理缓存失效和索引同步。

##### G. 综合建议 (Action Items)
1.  **分级特化**: 借鉴 `tsv-utils`，针对单列 GroupBy (特别是整数列) 使用特化路径，避免 `KeyBuffer` 构建。
2.  **数值稳定性**: 借鉴 `xan`，引入 Welford 算法作为默认或可选 (`--stable`) 实现。
3.  **近似算法**: 借鉴 `xan`，针对大数据场景引入 `--approx-unique` (HyperLogLog) 和 `--approx-quantile` (T-Digest)。
4.  **向量化思考**: 借鉴 `dplyr`，在实现 `tva` 的 `window` 或 `mutate` 功能时，应优先考虑基于 Chunk 的向量化计算，而非逐行迭代。
5.  **类型化键优化**: 借鉴 `tsv-utils` 的模板特化思路，为整数 Key 提供专门的 `u64` 哈希路径，避免字符串转换开销。

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
