# 开发者指南

本文档旨在为 `tva` 的开发者提供技术背景、架构设计思路以及未来演进路线。

## 目录

1. [架构与模块](#架构与模块)
2. [性能基准与分析](#性能基准与分析)
3. [深度技术分析](#深度技术分析)
   - [参考项目: rust-csv](#参考项目-rust-csv)
   - [参考项目: simd-csv](#参考项目-simd-csv)
   - [参考项目: xan](#参考项目-xan)
4. [自研 TSV 解析器设计](#自研-tsv-解析器设计)
5. [性能优化路线图](#性能优化路线图)
6. [计划中的功能](#计划中的功能)

---

## 架构与模块

`tva` 采用模块化设计，核心逻辑位于 `src/libs`，命令行接口位于 `src/cmd_tva`。

- **`src/libs/tsv`**: 核心解析层，包含零拷贝 Reader、Record 抽象和字段处理逻辑。
- **`src/libs/filter`**: 过滤引擎，支持多种比较操作符。
- **`src/libs/select`**: 列选择与重排逻辑。
- **`src/libs/stats`**: 统计计算逻辑。

---

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

## 深度技术分析

### 参考项目: rust-csv

`rust-csv` (BurntSushi/rust-csv) 是 Rust 生态中最权威的 CSV 解析库，也是 `tva` 的核心依赖之一。对其源码的分析有助于指导 `tva` 的底层优化和功能扩展。

#### 核心架构

该项目采用多 crate 架构，实现了分层抽象：

1.  **`csv-core` (核心状态机)**:
    *   **定位**: 无 `std` 依赖的裸机 CSV 解析器。
    *   **实现**: 基于确定性有限自动机 (DFA) 的状态机。
    *   **特点**: 极致的性能，专注于字节流的处理，不涉及 I/O 或内存分配。
    *   **启示**: `tva` 的高性能流式处理很大程度上归功于此。如果需要自定义复杂的解析逻辑（如特殊的转义规则），可能需要深入此层。

2.  **`csv` (高层 API)**:
    *   **定位**: 提供易用的 `Reader` 和 `Writer` 接口，集成 `serde`。
    *   **实现**: 封装 `csv-core`，处理缓冲 (Buffering)、I/O、UTF-8 验证和记录解析。
    *   **关键特性**:
        *   **ByteRecord**: 零拷贝解析的核心。它存储原始字节，避免了 UTF-8 校验和内存分配的开销，直到用户真正需要字符串时才转换。这对于 `tva filter` 和 `select` 等操作至关重要。
        *   **Serde 集成**: 提供了极其方便的 `deserialize` 接口，但对于极致性能场景（如 `tva` 的大部分命令），通常优先使用 `ByteRecord` API。

3.  **`csv-index` (索引机制)**:
    *   **定位**: 提供 CSV 文件的随机访问能力。
    *   **实现**: 创建辅助的索引文件（通常是 `.idx`），记录每行（或每块）的字节偏移量。
    *   **价值**: 这是 `qsv` 能够瞬间完成切片 (`slice`) 和统计计数 (`count`) 的秘密武器。
    *   **对 `tva` 的建议**:
        *   目前 `tva` 是纯流式的，这对于单次扫描非常高效。
        *   但对于需要多次扫描或随机访问大文件的场景（如 `sample --random-access` 或大文件 `slice`），引入 `csv-index` 是实现性能飞跃的关键。
        *   **行动项**: 研究将 `csv-index` 集成到 `tva` 的 `input` 层，允许用户为大文件生成索引，从而加速后续操作。

#### 性能优化借鉴

*   **缓冲策略**: `rust-csv` 内部使用了精细调整的缓冲区。`tva` 在处理 I/O 时应确保始终使用 `BufReader` 和 `BufWriter` 包装器（`src/libs/io.rs` 中已实现）。
*   **SIMD**: 虽然 `csv-core` 本身是标量实现，但现代 CSV 解析器（如 `simd-csv`）利用 SIMD 指令集可获得数倍性能提升。

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

#### 对 `tva` 的启示与潜在集成

*   **特定场景加速**:
    *   **行计数 (`tva nl --count`)**: 使用 `Splitter` 可能获得 4-6 倍的性能提升。
    *   **简单切片/过滤**: 如果确定数据无复杂转义，`ZeroCopyReader` 可大幅加速 `slice` 或简单 `filter`。
*   **作为可选后端**:
    *   考虑到其 API 不如 `csv` crate 友好且灵活性较低（例如对编码支持、非标准方言的处理），不建议完全替换 `csv`。
    *   **建议**: 可以在 `tva` 内部抽象一个 `Reader` trait，默认使用 `csv`，但在用户显式开启 `--fast` 标志或检测到简单数据时，切换到 `simd-csv` 后端。
*   **性能权衡**:
    *   README 指出在 worst-case（如全数字短字段）下性能提升微乎其微。因此集成时需谨慎评估引入依赖的成本与收益。

#### 深度分析: simd-csv 为何最快?

`simd-csv` 能够达到 1.12 GiB/s 的惊人速度，比 `csv` crate 快 60% 以上。通过分析其源码，我们发现了以下关键技术：

1.  **混合架构 (Hybrid Architecture)**:
    `simd-csv` 并不是纯 SIMD 解析器，而是一个混合体：
    *   **CoreReader (core.rs)**: 维护状态机 (Unquoted, Quoted, Quote)。
    *   **Searcher (searcher.rs)**: 使用 SIMD (`memchr` 或 SSE2/AVX2 intrinsic) 快速跳过普通字符。

2.  **Searcher 的核心逻辑**:
    在 `CoreReader::split_record` 中，它利用 SIMD 指令一次性扫描多个特殊字符（分隔符、换行符、引号）。这比逐字节查表（`csv` crate 的做法）更快，因为在大多数 CSV 数据中，特殊字符是稀疏的。

3.  **SSE2/AVX2 手写 Intrinsic**:
    `searcher.rs` 中包含了手写的 SSE2 实现：
    *   加载 16 字节到向量寄存器 (`_mm_loadu_si128`)。
    *   并行比较分隔符、换行符、引号 (`_mm_cmpeq_epi8`)。
    *   使用 `_mm_movemask_epi8` 将比较结果提取为位掩码。
    *   使用 `trailing_zeros` 快速找到第一个匹配位置。

### 参考项目: xan

`xan` (前身为 `xsv` 的 fork) 是一个功能极强的 CSV/TSV 工具集。通过分析其源码，我们可以为 `tva` 汲取以下几个关键的架构和功能灵感。

#### 1. 并行处理架构 (Parallel Processing)

*   **实现**: `cmd/parallel.rs`
*   **机制**: 类似于 Map-Reduce。它不试图让每个命令内部并行化，而是提供一个通用的 `parallel` 子命令。
    *   **Chunking**: 自动将文件分块，或按文件分发任务。
*   **对 `tva` 的启示**:
    *   `tva` 目前是单线程流式处理。
    *   **建议**: 实现 `tva parallel` 命令，负责将大文件切分 (利用 `split` 逻辑) 并启动多个子进程/线程处理，最后聚合结果。

#### 2. Join 架构对比: xan vs tva

通过分析 `xan/src/cmd/join.rs`，我们发现其设计哲学与 `tva` 截然不同。

| 特性 | xan join (SQL Style) | tva join (Stream Static) |
| :--- | :--- | :--- |
| **内存模型** | **全量加载 (Indexed Side)** | **部分加载 (Key + Append)** |
| **数据结构** | `Vec<IndexNode>` (Arena) + `HashMap` (Index) | `HashMap<Key, AppendValues>` |
| **Join 类型** | Inner, Left, Right, Full, Cross (N-to-N) | Hash Semi-Join (N-to-1) |
| **多重匹配** | 支持 (通过链表 `next` 指针) | **不支持** (Last-Win 或 Error) |
| **Key 构建** | `ByteRecord` (Vector of Fields) | `Cow<'a, [u8]>` (Slice) |

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
    let mut filter_map: HashMap<Vec<u8>, Vec<u8>> = HashMap::new();
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

## 性能优化路线图

**目标**: 在单线程模式下，通过极致的指令集优化和内存管理，超越通用解析器的极限。

1.  **AVX2/NEON 优化**:
    *   探索使用 `std::simd` (Portable SIMD) 或手写 intrinsic，一次性处理 32/64 字节。
    *   **Bitmask 技术**: 生成 `\t` 和 `\n` 的位置掩码，利用 `tzcnt` (Trailing Zero Count) 快速跳跃，避免逐字节比较。
2.  **Buffer 管理**:
    *   实现环形缓冲区 (Ring Buffer) 或双缓冲 (Double Buffering)，实现 I/O 与解析的重叠 (虽然在单线程下受限，但通过 `io_uring` 或异步 I/O 可能有收益)。
3.  **Profile-Guided Optimization (PGO)**:
    *   使用真实数据收集性能剖析信息，指导编译器进行分支预测优化和函数内联。

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
*   **列拆分/合并**:
    *   `unpack`: 使用分隔符或正则将单个字符串列拆分为多个列。
    *   `pack`: 使用模板或分隔符将多个列合并为单个字符串列。
*   **致密化 (Densification)**:
    *   `complete`: 暴露数据因子的缺失组合。

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

*   向 `stats` 添加 `skewness`, `kurtosis`。

### 缺失值填充 (Fill Missing Values)

*   `fill`: 实现前向/后向填充以及常数填充。


## 性能优化路线图

1.  **算法升级**: 将 `sample` 等命令的 O(N) 算法重构为 O(K) 或流式算法。
2.  **指令集优化**: 在关键路径 (如 `split`, `newline` 查找) 引入 AVX2/NEON 优化。
3.  **I/O 吞吐**: 增大默认缓冲区 (64KB -> 128KB+)，探索 `io_uring` (Linux)。

---

## 对标分析: tsv-sample (D语言)

为了彻底超越 `tsv-utils`，我们对其源码 (`tsv-sample.d`) 进行了深度逆向分析。

### 1. 核心架构对比

| 特性 | tsv-sample (D) | tva (Rust) | 差异分析 |
| :--- | :--- | :--- | :--- |
| **I/O 缓冲** | `bufferedByLine` (1MB Buffer) | `TsvReader` (64KB Buffer) | D 的缓冲区更大，减少了 syscall 次数。 |
| **RNG** | `std.random.Mt19937` | `rapidhash` (Wyhash) | **Rust 胜出**。Wyhash 更快且质量足够。 |
| **加权采样** | **A-Res (Heap)** - O(K) 内存 | **Naive Sort** - O(N) 内存 | **D 胜出**。D 使用最小堆维护 Top-K，Rust 目前全量排序导致大量内存分配。 |
| **Bernoulli 采样** | **Skip Sampling** | Naive Check | **D 胜出**。D 计算跳过行数，Rust 逐行检查。 |

### 2. 关键瓶颈定位 (v0.1.0)

用户反馈 `tva sample -w` 比 `tsv-sample` 慢 3-4 倍，核心原因是 **算法复杂度差异**：

*   **tsv-sample (D)**: 使用 **Efraimidis-Spirakis A-Res 算法**。
    *   维护一个大小为 $K$ 的最小堆。
    *   仅当新元素的 Key ($u^{1/w}$) 大于堆顶时才替换并调整堆。
    *   内存: $O(K)$, CPU: $O(N \log K)$。

*   **tva (Rust)**: 使用 **全量排序算法**。
    *   `Vec::push` 保存所有记录。
    *   最后 `sort_by` 全量排序。
    *   内存: $O(N)$, CPU: $O(N \log N)$。
    *   **后果**: 对于大文件 (如 1000万行)，Rust 版本会分配巨大的内存并进行昂贵的数据移动，导致性能雪崩。

### 3. 优化行动项

1.  **重构加权采样 (Weighted Sampling)**:
    *   实现 `BinaryHeap` (Min-Heap) 版本的 A-Res 算法。
    *   消除所有不必要的内存分配 (`Vec::push`)。

2.  **实现 Skip Sampling**:
    *   对于 `sample -p` (伯努利采样)，引入几何分布跳过算法，避免对每一行都调用 RNG。

3.  **调整 I/O 参数**:
    *   将输入缓冲区从 64KB 提升至 128KB 或 1MB。

---

## 对标分析: tsv-join (D语言)

为了解决 `tva join` 比 `tsv-join` 慢 1.5 倍的问题（通过优化 Key 构建逻辑，目前差距已缩小至 1.1 倍），我们对其源码 (`tsv-join.d`) 进行了深度逆向分析。

### 1. 核心组件与流程对比

| 组件 | tsv-join (D) | tva (Rust) | 差异与优势 |
| :--- | :--- | :--- | :--- |
| **I/O 缓冲** | `bufferedByLine` (1MB Buffer) | `TsvReader` (128KB Buffer) | **D 微弱优势**。更大的缓冲区能减少 syscall，但 128KB 已接近边际效益递减点。 |
| **字段解析** | `splitter` (Range Based) | `SelectPlan` + `memchr` | **Rust 胜出**。`tva` 使用预计算的 Plan 和 SIMD 查找，避免了 D 语言 Range 抽象的开销。 |
| **Key 构建** | `string.join` (Allocation!) | `Cow<'a, [u8]>` (Zero-Copy*) | **Rust 胜出**。D 在构建 Key 时（即使是单列）通常涉及字符串分配或切片复制；`tva` 对单列 Key 实现了完全零拷贝 (`Cow::Borrowed`)，仅多列 Key 需要分配。 |
| **Hash Map** | `string[string]` (Built-in AA) | `HashMap` (SipHash) | **D 胜出**。这是剩余差距的主要来源。D 的关联数组 (AA) 经过高度优化，且极可能使用了非加密 Hash (如 Murmur)。Rust 默认 `SipHash` 虽然抗 DoS 攻击但速度较慢。 |
| **内存管理** | GC (Garbage Collection) | RAII (Drop) | **D 胜出 (在小对象分配上)**。在多列 Join 需要分配 Key 时，D 的 GC 指针碰撞分配比 Rust 的 `malloc/free` 更快。 |

### 2. 深度流程分析

#### tsv-join.d (D)
1.  **Read**: `bufferedByLine` 读取一行。
2.  **Parse**: `splitter` 切分字段，生成 Range。
3.  **Key Construction**:
    *   使用 `InputFieldReordering` 提取字段。
    *   **Crucial**: 代码中 `key = ... .join(cmdopt.delim).to!string` 表明，对于每一行数据，D 都会分配一个新的 `string` 作为 Key。
4.  **Lookup**: `values = (key in filterHash)`。
    *   使用 D 语言内置的 `Associative Array`。
5.  **Write**: 写入匹配行。

#### tva join.rs (Rust)
1.  **Read**: `TsvReader` 读取一行 (`&[u8]`)。
2.  **Parse**: `SelectPlan` 使用 `memchr` 填充 `Vec<Range>` (Zero Allocation, Reused Buffer)。
3.  **Key Construction**:
    *   **单列 (常见情况)**: 直接返回 `Cow::Borrowed(&line[range])`。**完全无内存分配**。
    *   **多列**: `Vec::with_capacity` -> `extend_from_slice` -> `Cow::Owned`。
4.  **Lookup**: `map.get(key)`。
    *   Rust 的 `HashMap` 默认使用 `SipHash 1-3`，这是一种加密安全但相对较慢的哈希算法。
5.  **Write**: 写入匹配行。

### 3. 为什么还有 1.1x 差距?

既然 `tva` 在 I/O 和 Parsing 上实现了零拷贝，甚至在 Key 构建上也优于 D（单列场景），为什么总时间依然慢 10%？

**结论**: **哈希函数的开销掩盖了零拷贝的优势**。
在 `join` 操作中，哈希表的查找是绝对的热点路径。对于数百万行的 Join，就需要计算数百万次 Hash。
*   **Rust**: `SipHash` (安全，慢)。
*   **D**: `MurmurHash` 或类似 (非加密，快)。

### 4. 进一步优化建议

1.  **Prefetching**:
    *   在计算 Hash 的同时预取内存，但这在纯内存操作中收益可能有限。

### 5. 已实施的优化 (v0.1.0+)

1.  **更换 Hasher**:
    *   已将 `HashMap` 的 Hasher 替换为 `ahash` (RandomState)。
    *   预期: 消除 Hash 计算上的性能劣势。
2.  **Inline Key (Small String Optimization)**:
    *   使用 `SmallVec<[u8; 32]>` 作为 Key 类型。
    *   对于短 Key (< 32 字节)，完全消除了堆分配（无论是单列还是多列）。

