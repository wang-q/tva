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

## 参考项目分析: rust-csv

`rust-csv` (BurntSushi/rust-csv) 是 Rust 生态中最权威的 CSV 解析库，也是 `tva` 的核心依赖之一。对其源码的分析有助于指导 `tva` 的底层优化和功能扩展。

### 核心架构

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

### 性能优化借鉴

*   **缓冲策略**: `rust-csv` 内部使用了精细调整的缓冲区。`tva` 在处理 I/O 时应确保始终使用 `BufReader` 和 `BufWriter` 包装器（`src/libs/io.rs` 中已实现）。
*   **SIMD**: 虽然 `csv-core` 本身是标量实现，但现代 CSV 解析器（如 `simd-csv`）利用 SIMD 指令集可获得数倍性能提升。
    *   **思考**: 详见下文对 `simd-csv` 的分析。

## 参考项目分析: simd-csv

`simd-csv` (medialab/simd-csv) 是一个专门利用 SIMD 指令集加速 CSV 解析的 Rust crate。它并非 C++ `simdjson` 的直接移植，而是采用了混合传统状态机与 SIMD 字符串搜索的新颖方法。

### 核心特性与架构

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

### 对 `tva` 的启示与潜在集成

*   **特定场景加速**:
    *   **行计数 (`tva nl --count`)**: 使用 `Splitter` 可能获得 4-6 倍的性能提升。
    *   **简单切片/过滤**: 如果确定数据无复杂转义，`ZeroCopyReader` 可大幅加速 `slice` 或简单 `filter`。
*   **作为可选后端**:
    *   考虑到其 API 不如 `csv` crate 友好且灵活性较低（例如对编码支持、非标准方言的处理），不建议完全替换 `csv`。
    *   **建议**: 可以在 `tva` 内部抽象一个 `Reader` trait，默认使用 `csv`，但在用户显式开启 `--fast` 标志或检测到简单数据时，切换到 `simd-csv` 后端。
*   **性能权衡**:
    *   README 指出在 worst-case（如全数字短字段）下性能提升微乎其微。因此集成时需谨慎评估引入依赖的成本与收益。

## 性能瓶颈分析: tva select vs tsv-select

用户反馈 `tva select` 在某些场景下比 `tsv-select` (D语言版本) 慢 4 倍。经过源码对比分析，主要原因在于内存分配策略和处理逻辑的差异。

### 1. 激进的内存分配 (Aggressive Allocation)

**tva (`src/cmd_tva/select.rs`)**:
```rust
// 每一行都会分配一个新的 Vec<&str>
let fields: Vec<&str> = line.split(delimiter).collect();
```
*   **问题**: 对每一行都进行全量切分并收集到 `Vec` 中，即使文件有数百万行。这导致了巨大的内存分配和释放开销。
*   **后果**: 随着行长和列数的增加，性能急剧下降。

**tsv-select (`tsv-select.d`)**:
```d
// 使用惰性迭代器 (Lazy Iterator)，无堆内存分配
foreach (fieldIndex, fieldValue; line.splitter(cmdopt.delim).enumerate)
```
*   **优势**: D 语言版本使用了 `std.algorithm.splitter`，它是惰性的。它不会一次性为所有字段分配内存，而是按需处理。

### 2. 缺乏提前退出机制 (No Early Exit)

**tva**:
由于使用了 `.collect()`，`tva` 必须解析整行的所有字段，即使你只想要第 1 列。
*   例如: `tva select -f 1 big_file.tsv`
*   **实际行为**: 解析整行 -> 分配 Vec -> 取第 1 个元素 -> 丢弃 Vec。

**tsv-select**:
```d
// 一旦收集齐所需字段，立即停止解析该行
if (fieldReordering.allFieldsFilled) break;
```
*   **优势**: 如果只请求前几列，`tsv-select` 会在解析完这些列后立即停止处理该行的剩余部分。对于宽表（由于列多，行很长）且只选择前几列的场景，这种差异会导致巨大的性能鸿沟。

### 3. 优化建议

1.  **移除 `collect()`**: 改用迭代器处理字段，避免为每行分配 `Vec`。
2.  **实现提前退出**: 在迭代过程中，一旦获取了所有目标字段（且没有 `--rest` 或排除逻辑），立即停止解析当前行。
3.  **复用缓冲区**: 考虑复用行缓冲区或字段缓冲区，减少内存抖动。

## 自研 TSV 解析器 (Rationale for Custom TSV Parser)

通过分析 `rust-csv` 和 `simd-csv`，我们发现通用的 CSV 解析器为了兼容 RFC 4180 (处理引号、转义、多行记录等) 引入了复杂的不可避免的开销。
鉴于 TSV (Tab-Separated Values) 的格式极其简单，我们可以实现一个专用、高性能的 TSV 解析器。

### 1. 格式差异分析

| 特性 | CSV (RFC 4180) | TSV (Simple) | 影响 |
| :--- | :--- | :--- | :--- |
| **分隔符** | `,` (可变) | `\t` (固定) | TSV 可硬编码分隔符，利于 SIMD 优化。 |
| **引号** | 支持 `"` 包裹字段 | **不支持** | TSV 无需维护 "in_quote" 状态，彻底消除状态机分支预测失败。 |
| **转义** | `""` 转义引号 | 无 (或 C 风格 `\t`) | TSV 无需处理 `""` -> `"` 的内存拷贝/重写，支持真正的零拷贝切片。 |
| **换行** | 字段内可含换行 | **不允许** | TSV 保证 `\n` 永远代表记录结束。可并行分块查找 `\n`。 |

### 2. 现有实现开销

*   **rust-csv**: 使用 DFA (确定性有限自动机) 逐字节扫描。虽然优化到了极致，但仍需为每个字节检查 `Transition Table`，处理引号状态和转义逻辑。对于不含引号的 TSV，这些检查是多余的。
*   **simd-csv**: 混合了 SIMD 和状态机。虽然它使用 SIMD 快速跳过非特殊字符，但一旦遇到特殊字符，仍需进入状态机判断是否在引号内。

### 3. 自研 TSV 解析器设计思路

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

## 性能基准测试结果 (parse_benchmark)

我们在 `benches/parse_benchmark.rs` 中对比了不同解析策略的性能。
数据样本: `1\tJohn\tDoe\t30\tNew York\n...` (3行数据重复 1000 次)

| 策略 | 平均耗时 | 吞吐量 | 说明 |
| :--- | :--- | :--- | :--- |
| **simd-csv** | **61.65 µs** | **1.15 GiB/s** | 混合 SIMD 状态机，最快 |
| **Memchr2 SIMD Loop** | **84.84 µs** | **854.35 MiB/s** | **理论验证**: 纯 SIMD 扫描 `\t` 和 `\n`，超越 csv crate 25% |
| **Memchr Reused Buffer** | 89.85 µs | 806.66 MiB/s | 逐行 memchr，受限于函数调用开销 |
| **csv crate** | 105.43 µs | 687.47 MiB/s | 经典的 DFA 状态机 |
| **TsvRecord Struct** | 105.87 µs | 684.63 MiB/s | 封装好的零拷贝结构体 |
| **Manual Byte Loop** | 139.56 µs | 519.33 MiB/s | 标量循环 |
| **Std Split Iterator** | 172.37 µs | 420.48 MiB/s | Rust 标准库 `split` |
| **Memchr Inline Loop** | 200.99 µs | 360.62 MiB/s | 受限于 `reader.lines()` 分配 |
| **Naive Split** | 451.10 µs | 160.67 MiB/s | 原始实现，最慢 |

**结论**:
1.  **SIMD 的胜利**: `Memchr2 SIMD Loop` (854 MiB/s) 证明了使用 `memchr2` 同时查找分隔符和换行符比逐行查找更快。它比 `csv` crate 快了 25%，逼近了 `simd-csv` 的水平。
2.  **I/O 模型决定上限**: 所有高性能策略 (`Memchr2`, `Reused Buffer`, `simd-csv`) 都依赖于直接操作 Buffer。`reader.lines()` 的分配开销是性能杀手 (限制在 ~400 MiB/s)。
3.  **最终方案**: 我们应该在 `tva` 中实现一个基于 `memchr2` 的分块读取器 (Chunked Reader)。这不仅能超越 `csv` crate，而且代码复杂度远低于 `simd-csv`（因为无需处理引号）。

**下一步行动**:
实现 `TsvReader`，内部维护一个 buffer，使用 `memchr2` 迭代器直接在 buffer 上切分字段，实现真正的零拷贝解析。

## 性能优化路线图 (Performance Optimization Roadmap)

基于上述分析，我们制定以下三阶段优化计划，旨在将 `tva` 打造成最快的 TSV 处理工具。

### 第一阶段：摘取低垂的果实 (Low Hanging Fruit) - 立即执行

**目标**: 不改变整体架构，通过消除明显的低效代码，获得 2-3 倍的性能提升。

1.  **移除 `Vec` 分配**:
    *   在 `select` 中，将 `line.split('\t').collect::<Vec<&str>>()` 替换为惰性迭代器 `line.split('\t')`。
2.  **实现 "Early Exit"**:
    *   在迭代字段时，一旦获取了所需的字段（例如只需要第 1、3 列），立即停止对该行剩余部分的解析。
3.  **减少 String 分配**:
    *   使用 `std::io::BufRead::read_until` 配合复用的 `Vec<u8>` buffer，代替 `lines()` 迭代器（后者每次都会分配新的 `String`）。

### 第二阶段：自研专用 TSV 解析器 (Specialized TSV Parser) - 短期

**目标**: 引入 `tva-core` 模块，实现零拷贝、基于字节的 TSV 解析，性能超越 `csv` crate，逼近 `simd-csv`。

1.  **引入 `memchr`**: 添加 `memchr` crate 依赖，利用其 SIMD 加速的字节查找功能。
2.  **实现 `TsvRecord`**:
    *   输入: `&[u8]` (当前行的字节切片)。
    *   逻辑: 使用 `memchr_iter(b'\t', line)` 返回字段的切片迭代器。
    *   特性: 纯指针算术，无任何堆内存分配。
3.  **集成到 `select`**:
    *   重写 `select` 命令的核心循环，使用新的解析器。

### 第三阶段：极致 SIMD 与单线程优化 (Extreme SIMD & Single-threaded Optimization) - 中长期

**目标**: 在单线程模式下，通过极致的指令集优化和内存管理，超越通用解析器的极限。

1.  **AVX2/NEON 优化**:
    *   探索使用 `std::simd` (Portable SIMD) 或手写 intrinsic，一次性处理 32/64 字节。
    *   **Bitmask 技术**: 生成 `\t` 和 `\n` 的位置掩码，利用 `tzcnt` (Trailing Zero Count) 快速跳跃，避免逐字节比较。
2.  **Buffer 管理**:
    *   实现环形缓冲区 (Ring Buffer) 或双缓冲 (Double Buffering)，实现 I/O 与解析的重叠 (虽然在单线程下受限，但通过 `io_uring` 或异步 I/O 可能有收益)。
3.  **Profile-Guided Optimization (PGO)**:
    *   使用真实数据收集性能剖析信息，指导编译器进行分支预测优化和函数内联。

## 深度分析: 为何 csv crate 如此之快? (Deep Dive into csv crate)

尽管 `csv` crate 基于 DFA 状态机，理论上比纯 SIMD 慢，但其 700 MiB/s 的性能仍令人印象深刻。分析其源码 (`rust-csv-master/csv-core/src/reader.rs`) 揭示了其性能秘密：

### 1. 极其紧凑的 DFA 状态表 (Compact DFA Table)

*   **问题**: 朴素的 DFA 表大小为 `(状态数) * 256`。对于 CSV 这种只有几个特殊字符的格式，这是巨大的浪费且缓存不友好。
*   **优化**: `csv` crate 将所有字节映射为 **7 个等价类 (Equivalence Classes)**:
    1.  Delimiter (分隔符)
    2.  Terminator (行结束符)
    3.  CR, LF (如果 Terminator 是 CRLF)
    4.  Quote (引号)
    5.  Escape (转义符)
    6.  Comment (注释符)
    7.  **Everything Else (其他所有字节)**
*   **结果**: 状态表大小从 `N * 256` 缩减为 `N * 7`。这使得整个状态表可以完全放入 L1 Cache，极大减少了内存访问延迟。

### 2. 批量处理与分支消除 (Branch Elimination)

在 `read_field_dfa` 循环中：
```rust
while nin < input.len() && nout < output.len() {
    let b = input[nin];
    // 查表，几乎无分支
    let (s, has_out) = self.dfa.get_output(state, b);
    state = s;
    if has_out {
        output[nout] = b;
        nout += 1;
    }
    nin += 1;
}
```
*   它不进行逐个 `if b == b','` 的判断，而是查表。CPU 的分支预测器在这里几乎不会失败，因为大部分字节都属于 "Everything Else" 类，状态转换非常规律。

### 3. SIMD 还是标量?

`csv-core` 主要是标量实现。它之所以快，是因为它**不分配内存**且**缓存极其友好**。
这给我们一个警示：`TsvSplitter` (memchr) 虽用了 SIMD，但如果迭代器本身的函数调用开销 (Function Call Overhead) 或状态维护 (State Management) 过于复杂，反而可能不如一个紧凑的标量循环。

**对 tva 的修正策略**:
我们目前的 `TsvSplitter` 还是一个迭代器，每次 `next()` 都有开销。
为了超越 `csv` crate，我们需要像它一样实现一个**紧凑的内部循环**，或者一次性处理多个字段，减少函数调用边界。

### 4. buffer?

这是一个非常反直觉的现象。`csv` crate 使用了复杂的状态机，而我们的 `Memchr Reused Buffer` 只是简单地扫描字节。理论上我们应该更快。

原因在于 **I/O 锁与 buffer 拷贝**：
1.  **std::io::BufReader::read_until**:
    *   这个函数内部会进行内存拷贝。它将 `BufReader` 内部 buffer 的数据 `memcpy` 到用户提供的 `Vec<u8>` 中。
    *   对于每一行数据，我们都付出了一次额外的内存拷贝代价。
2.  **csv crate 的零拷贝黑魔法**:
    *   `csv` crate 的 `ByteRecord` 设计非常精妙。它直接操作内部 buffer，或者在必要时才进行拷贝。
    *   更重要的是，它的状态机是直接在 buffer 上运行的，最大限度地减少了数据移动。

**解决方案**:
要超越 `csv` crate，我们不能使用标准的 `BufReader` 接口（因为它强制拷贝）。我们需要：
*   **直接管理 Buffer**: 手动填充 buffer，直接在 buffer 上解析，只在 buffer 耗尽时移动剩余数据并再次填充。
*   或者使用 `memmap` 将文件映射到内存，彻底消除用户态拷贝。

## 深度分析: simd-csv 为何最快? (Deep Dive into simd-csv)

`simd-csv` 能够达到 1.12 GiB/s 的惊人速度，比 `csv` crate 快 60% 以上。通过分析其源码，我们发现了以下关键技术：

### 1. 混合架构 (Hybrid Architecture)

`simd-csv` 并不是纯 SIMD 解析器，而是一个混合体：
*   **CoreReader (core.rs)**: 维护状态机 (Unquoted, Quoted, Quote)。
*   **Searcher (searcher.rs)**: 使用 SIMD (`memchr` 或 SSE2/AVX2 intrinsic) 快速跳过普通字符。

### 2. Searcher 的核心逻辑

在 `CoreReader::split_record` 中：
```rust
Unquoted => {
    // 快速路径：跳过非特殊字符
    // memchr2(delimiter, newline, ...)
    if let Some(offset) = memchr2(b'\n', self.quote, &input[pos..]) {
        pos += offset;
        // ... 处理特殊字符
    }
}
```
它利用 SIMD 指令一次性扫描多个特殊字符（分隔符、换行符、引号）。这比逐字节查表（`csv` crate 的做法）更快，因为在大多数 CSV 数据中，特殊字符是稀疏的。

### 3. SSE2/AVX2 手写 Intrinsic

`searcher.rs` 中包含了手写的 SSE2 实现：
*   加载 16 字节到向量寄存器 (`_mm_loadu_si128`)。
*   并行比较分隔符、换行符、引号 (`_mm_cmpeq_epi8`)。
*   使用 `_mm_movemask_epi8` 将比较结果提取为位掩码。
*   使用 `trailing_zeros` 快速找到第一个匹配位置。

### 4. 我们可以超越它吗?

*   **TSV 的优势**: TSV **没有引号**。这意味着我们不需要像 `simd-csv` 那样在 `memchr` 命中后还要检查是否是引号，也不需要维护 `Quoted` 状态。
*   **更简单的 SIMD**: 我们只需要查找 `\t` 和 `\n`。这比 CSV 的 3-4 个特殊字符更少，寄存器压力更小。
*   **理论极限**: 如果 `simd-csv` 要处理引号逻辑还能跑 1.12 GiB/s，那么纯粹查找 `\t` 和 `\n` 的 TSV 解析器理论上应该能达到内存带宽的极限（或至少 2-3 GiB/s）。

**行动项**:
我们不需要复杂的混合状态机。我们只需要一个极致优化的 `memchr2(b'\t', b'\n')` 循环，配合 Buffer 管理。我们的 `Memchr Reused Buffer` (814 MiB/s) 已经验证了这一点，差距仅在于 `simd-csv` 可能使用了更底层的 SIMD 优化或更高效的 I/O 缓冲。

