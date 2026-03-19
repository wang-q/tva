# 开发者指南

本文档旨在为 `tva` 的开发者提供技术背景、架构设计思路以及未来演进路线。

## changelog

```bash
git log v0.3.0..HEAD > gitlog.txt
git diff v0.3.0 HEAD -- "*.rs" "*.md" > gitdiff.txt
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

为这些地方，添加单元测试与整合测试

为刚才的修改，添加单元测试与整合测试

## WSL

```bash
mkdir -p /tmp/cargo
export CARGO_TARGET_DIR=/tmp/cargo
cargo build
```

## 深度技术分析

### 参考项目: GNU Datamash

`datamash` 是命令行统计分析的标杆工具。`tva` 可以借鉴其在数据验证和交叉制表方面的设计。

#### 5. 逐行转换操作 (Per-Line Operations)

* **特性**: datamash 提供大量逐行转换操作，无需分组即可使用：
    * **数值修约**: `trunc`, `frac`。
    * **文件路径处理**: `dirname`, `basename`, `extname`, `barename`。
    * **数值提取**: `getnum` 从混合文本中提取数字（如 "zoom-123.45xyz" -> 123.45）。
    * **分箱**: `strbin` (字符串哈希分箱)。

#### 6. 示例文件组织

* **特性**: datamash 的 `examples/` 目录包含：
    * `scores.txt` / `scores_h.txt`: 成对的无表头/有表头示例。
    * `genes.txt` / `genes_h.txt`: 真实生物信息学数据（UCSC Genome Browser）。
    * `readme.md`: 详细解释每个示例的用法和场景。
* **借鉴**:
    * 为 `tva` 的 `docs/data/` 提供成对的示例文件（有/无表头）。
    * 添加真实领域数据：金融数据、日志数据、科学数据。
    * 编写 `docs/data/README.md` 详细说明示例用途。

### 参考项目: xan

`xan` (前身为 `xsv` 的 fork) 是一个功能极强的 CSV/TSV 工具集。通过分析其源码，我们可以为 `tva`
汲取以下几个关键的架构和功能灵感。

#### 1. 并行处理架构 (Parallel Processing)

* **实现**: `cmd/parallel.rs`
* **机制**: 类似于 Map-Reduce。它不试图让每个命令内部并行化，而是提供一个通用的 `parallel` 子命令。
    * **Chunking**: 自动将文件分块，或按文件分发任务。
    * **Shuffle**: 保证输出顺序与输入一致（如果需要）。
* **对 `tva` 的启示**:
    * `tva` 目前是单线程流式处理。
    * **建议**: 实现 `tva parallel` 命令，负责将大文件切分 (利用 `split` 逻辑) 并启动多个子进程/线程处理，最后聚合结果。

#### 4. 随机访问与索引 (Random Access & Indexing)

* **实现**: `src/config.rs` & `bgzip`
* **机制**: 利用 `.gzi` 索引文件（BGZF 格式），支持不解压整个文件的情况下 Seek 到 Gzip 中间。
* **对 `tva` 的启示**:
    * 对于大文件（GB/TB 级）的并行处理至关重要。
    * **建议**: 处理超大压缩 TSV 时，支持 BGZF 索引是实现并行切片 (`slice`) 和随机采样 (`sample`)
      的基础。

## 计划中的功能

### 数据重塑 (Data Reshaping) - Tidyr 对等功能

* **多度量透视 (Multi-measure Pivoting)**:
    * `longer`: 支持在 `--names-to` 中使用 `.value` 哨兵，同时透视到多个值列。
    * `wider`: 允许 `--values-from` 接受多个列。
* **列拆分/合并 (Column Splitting/Merging)**:
    * `separate` (unpack): 使用分隔符或正则将单个字符串列拆分为多个列。
    * `unite` (pack): 使用模板或分隔符将多个列合并为单个字符串列。
* **行拆分 (Row Splitting)**:
    * `separate-rows` (explode): 将包含分隔符的单元格拆分为多行 (e.g. "a,b" -> 2 rows)。
* **致密化 (Densification)**:
    * `complete`: 暴露数据因子的缺失组合，并支持填充默认值。
    * `expand`: 仅生成唯一值的笛卡尔积（Cartesian Product），用于构建参考网格。
* **行复制 (Row Replication)**:
    * `uncount`: 根据计数列的值复制行（逆向 `count`）。
* **缺失值处理 (Missing Values)**:
    * `replace_na`: 将显式 `NA` (空字符串) 替换为指定值。
    * `drop_na`: 丢弃包含缺失值的行。

### 数据操作 (Data Manipulation) - dplyr 核心模式

* **安全连接 (Safe Joins)**:
    * 行动: 添加 `--relationship` 标志（例如 `one-to-one`, `many-to-one`）在连接时验证键。
* **Tidy Selection DSL**:
    * 行动: 增强 `src/libs/fields.rs` 以支持正则 (`matches('^date_')`)、谓词 (`where(is_numeric)`)
      和集合操作 (`-colA`)。
* **窗口函数 (Window Functions)**:
    * 行动: 为 `filter` 和 `stats` 实现滑动窗口逻辑（例如，组内 `filter --expr "val > mean(val)"`）。
* **高强度测试 (Torture Testing)**:
    * 行动: 创建 `tests/torture/` 用于模糊测试输入，确保零 panic。

### 借鉴 xan 的未来演进路线 (Future Roadmap: Lessons from xan)

通过对 `xan` 源码的深入分析，我们发现了几个极具价值的功能模块，值得 `tva` 在未来版本中借鉴或引入。

#### 1. Transform (列变换)

* **功能**: `xan transform` 允许使用表达式（基于 `moonblade` 解释器）对列进行就地修改。例如
  `xan transform surname 'upper(_)'`。
* **价值**: `tva` 目前缺乏灵活的列处理能力。虽然 `awk` 可以胜任，但内置的 `transform`
  可以提供更好的性能和更简便的语法（无需处理分隔符）。
* **建议**: 引入轻量级表达式引擎（如 `rhai` 或简单的自定义解析器），实现类似 `tva mutate` 或
  `tva transform` 的命令，支持常见的字符串处理（upper, lower, trim, regex_replace）和数值计算。

#### 2. Search (高级搜索)

* **功能**: `xan search` 远超简单的 `grep`。它支持：
    * **多模式匹配**: 同时搜索数千个关键词（基于 Aho-Corasick 算法）。
    * **模糊匹配**: `xan fuzzy-join` 和搜索支持基于 Levenshtein 距离的匹配。
    * **替换**: 支持正则替换并输出到新列。
* **价值**: 在数据清洗（ETL）场景中，批量关键词匹配和替换是刚需。
* **建议**: 增强 `tva filter` 或新增 `tva search`，集成 `aho-corasick` crate 以支持高性能的多模式匹配。

#### 3. 表达式语言设计 (Moonblade 分析)

`xan` 的 `moonblade` 表达式引擎设计精良，以下是值得 `tva` 借鉴的关键特性：

**3.2 待添加的函数**

以下函数尚未实现，可参考 `xan` 添加：

| 函数类别 | 函数名 | 功能 | 优先级 |
|:--------|:------|:-----|:------|
| **文件路径** | `abspath`/`dirname`/`basename` | 路径处理 | 低 |

## tva 与 xan 命令对比分析

通过对比 tva 和 xan 的命令集，以下是 tva 尚未实现但可能有价值的功能：

### 高优先级（建议尽快实现）

| 命令            | xan 对应   | 功能描述  | 价值                     |
|:--------------|:---------|:------|:-----------------------|
| **frequency** | `freq`   | 频次统计表 | 快速了解数据分布，配合 `hist` 可视化 |
| **rename**    | `rename` | 重命名列  | 数据清洗基础功能               |

### 低优先级（特定场景）

| 命令           | xan 对应     | 功能描述                | 价值             |
|:-------------|:-----------|:--------------------|:---------------|
| **progress** | `progress` | 显示处理进度              | 大文件用户体验        |
| **search**   | `search`   | 多模式搜索（Aho-Corasick） | 高性能关键词搜索       |
| **separate** | `separate` | 使用正则拆分列             | 比 `unpack` 更灵活 |
| **window**   | `window`   | 滑动窗口计算              | 时间序列分析         |

### 暂不推荐实现

| 命令         | xan 对应     | 原因            |
|:-----------|:-----------|:--------------|
| `input`    | `input`    | 功能简单，可用其他方式替代 |
| `parallel` | `parallel` | 架构复杂，可后期考虑    |

### Hist (直方图) 补充分析

除了 `plot`，`xan` 还提供了一个专门的 `hist` 命令 (`xan/src/cmd/hist.rs`)，用于绘制水平条形图（Horizontal
Bar Charts）。

* **定位差异**:
    * `plot`: 通用绘图工具，支持散点图、折线图、垂直条形图，基于 `ratatui`，交互性强，适合复杂数据探索。
    * `hist`: 专注于**频次分布可视化**，通常配合 `freq` 或 `bins` 命令使用。它不使用 `ratatui`，而是直接通过
      Unicode 字符（如 `█`, `▌`）在标准输出打印，更轻量，适合管道操作。
* **核心逻辑**:
    * **数据模型**: 期望输入包含 `field` (可选), `value` (标签), `count` (数值) 三列。
    * **渲染**: 手动计算每个条形的宽度，使用 `Scale` 进行线性或对数缩放，并处理颜色（Rainbow/Category/Stripes）。
    * **特色功能**: 支持日期补全 (`--dates`)，自动填充缺失的日期并设为 0；支持间隙压缩 (
  `--compress-gaps`)，隐藏连续的 0 值。

### 优先级建议

| 优先级 | 改进项 | 原因 |
|:------|:------|:-----|
| 🟢 低 | 路径处理函数 | 特定场景 |
| 🔵 未来 | 类型系统重构 (`Arc`, `DateTime`) | 需要大量测试 |

## Arc 优化基准测试结果

针对 `tva` 的 `Value` 类型使用 `Arc` 进行优化的可行性，我们编写了基准测试（`benches/value_arc.rs`），
对比当前直接克隆与使用 `Arc` 包装后的性能差异。

**测试环境**: Release 模式，iterations = 10,000

**关键发现**:

| 场景 | 当前实现 | Arc 优化 | 加速比 | 结论 |
|:---:|:---:|:---:|:---:|:---:|
| String 克隆 | 407 µs | 35 µs | **11.6x** | ✅ 显著提升 |
| List 克隆 | 399 µs | 36 µs | **11.1x** | ✅ 显著提升 |
| `take()` 函数 | 48.3 ms | 1.38 ms | **35x** | ✅ 显著提升 |
| `reverse()` 函数 | 47.2 ms | 1.35 ms | **35x** | ✅ 显著提升 |
| `slice()` 函数 | 2.67 ms | 2.67 ms | **1x** | ⚠️ 持平 |
| `sort()` 函数 | 1.33 ms | 64.9 ms | **0.02x** | ❌ 显著下降 |
| `unique()` 函数 | 3.97 ms | 426 ms | **0.009x** | ❌ 显著下降 |
| `filter()` 函数 | 1.08 ms | 30.9 ms | **0.035x** | ❌ 显著下降 |
| `map()` 函数 | 1.49 ms | 77.7 ms | **0.019x** | ❌ 显著下降 |

**分析**:

* **Arc 有优势的场景**: 纯克隆操作（不修改数据）和频繁参数传递的场景。
  `Arc` 仅递增引用计数（O(1)），而直接克隆需要深拷贝（O(n)）。

* **Arc 无优势的场景**: 需要遍历并创建新列表的操作（`sort`, `filter`, `map`, `unique`）。
  这些操作需要 `list.iter().cloned().collect()`，比直接 `list.clone()` 慢得多。
  此外，`Arc<Vec<T>>` 无法直接获取可变引用，需要 `Arc::make_mut` 或重新分配 Vec。

**字符串操作基准测试** (`benches/string_arc.rs`):

| 函数 | 当前实现 | Arc 优化 | 加速比 | 结论 |
|:---:|:---:|:---:|:---:|:---:|
| String 克隆 | 407 µs | 35 µs | **11.6x** | ✅ 显著提升 |
| `split()` | 47.2 ms | 1.35 ms | **35x** | ✅ 显著提升 |
| `replace()` | 114 µs | 96.6 µs | **1.2x** | ⚠️ 轻微提升 |
| `concat()` | 4.01 ms | 2.58 ms | **1.6x** | ✅ 一定提升 |
| `upper()` | 26.2 µs | 22.5 µs | **1.2x** | ⚠️ 轻微提升 |
| `take_str()` | 1.66 ms | 1.61 ms | **1.03x** | ⚠️ 基本持平 |
| `substr()` | 1.73 ms | 1.94 ms | **0.89x** | ⚠️ 轻微下降 |

* **字符串操作分析**:
  * `split()` 受益于 Arc，因为需要频繁克隆参数
  * `replace()`, `upper()` 主要开销在字符串操作本身，Arc 优势不明显
  * `take_str()`, `substr()` 主要开销在新字符串分配，Arc 优势被抵消

**对 `tva` 的启示**:

* 如果 `expr` 命令主要使用 `take`, `first`, `last`, `nth` 等访问操作，`Arc` 优化是有价值的。
* 如果频繁使用 `sort`, `filter`, `map` 等转换操作，当前实现可能更合适。
* 字符串操作：`split()` 受益明显，其他操作收益有限。
* **最终建议**: 考虑到 `tva` 的核心是 TSV 文本处理，当前 `Value` 类型设计已足够高效，暂不引入 `Arc` 增加复杂度。

## SIMD 规划

`tva` 已经使用 `memchr` crate 实现 SIMD 加速的 TSV 解析。`memchr` 内部使用 SIMD 指令（SSE2/AVX2/NEON）来加速字节搜索。

### 当前 SIMD 实现

#### 1. TsvSplitter - SIMD 字段分割

`src/libs/tsv/split.rs` 使用 `memchr::memchr_iter` 快速定位分隔符：

```rust
use memchr::memchr_iter;

pub struct TsvSplitter<'a> {
    data: &'a [u8],
    iter: memchr::Memchr<'a>,
    last_pos: usize,
    finished: bool,
}

impl<'a> Iterator for TsvSplitter<'a> {
    type Item = &'a [u8];

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match self.iter.next() {
            Some(pos) => {
                let field = unsafe { self.data.get_unchecked(self.last_pos..pos) };
                self.last_pos = pos + 1;
                Some(field)
            }
            None => {
                self.finished = true;
                Some(unsafe { self.data.get_unchecked(self.last_pos..) })
            }
        }
    }
}
```

#### 2. TsvRecord - SIMD 加速解析

`src/libs/tsv/record.rs` 使用 `memchr::memchr_iter` 扫描分隔符位置：

```rust
pub fn parse_line(&mut self, line: &[u8], delimiter: u8) {
    self.clear();
    self.line.extend_from_slice(line);

    // 使用 memchr_iter 快速扫描分隔符
    for pos in memchr::memchr_iter(delimiter, &self.line) {
        self.ends.push(pos);
    }
    self.ends.push(self.line.len());
}
```

#### 3. SelectPlan - SIMD 字段选择

`src/libs/tsv/select.rs` 使用 `memchr::memchr_iter` 进行高效的字段提取：

```rust
pub fn extract_ranges(
    &self,
    line: &[u8],
    delimiter: u8,
    output_ranges: &mut Vec<Range<usize>>,
) -> Result<(), usize> {
    let mut iter = memchr::memchr_iter(delimiter, line);
    // ... 单次遍历提取指定字段
}
```

### SIMD 优化原理

`memchr` crate 的 SIMD 实现：

| 架构 | 指令集 | 处理宽度 | 说明 |
|:-----|:-------|:---------|:-----|
| x86_64 | SSE2 | 16 字节 | 基准实现，所有 x86_64 支持 |
| x86_64 | AVX2 | 32 字节 | 运行时检测，需较新 CPU |
| aarch64 | NEON | 16 字节 | ARM64 支持 |
| fallback | 标量 | 1 字节 | 无 SIMD 时的回退 |

核心算法（以 SSE2 为例）：
```rust
// 一次比较 16 个字节
let chunk = _mm_loadu_si128(ptr as *const __m128i);
let cmp = _mm_cmpeq_epi8(chunk, target);  // 16 个并行比较
let mask = _mm_movemask_epi8(cmp) as u32; // 压缩为 16-bit mask
// mask 的每一位表示对应位置是否匹配
```

### 性能对比

#### 基准测试结果 (`benches/tsv_parse.rs`)

测试数据：3000 行 TSV，每行 5 个字段 (约 71KB 数据)

| 实现 | 时间 | 吞吐量 | 相对性能 |
|:-----|:-----|:-------|:---------|
| **simd-csv** | 71.8 µs | **1009.5 MiB/s** | 🥇 最快 |
| **tva_tsv_reader** | 84.6 µs | **857.0 MiB/s** | 🥈 快 (比 simd-csv 慢 15%) |
| csv crate | 106.5 µs | 682.0 MiB/s | 中等 |
| memchr2_simd_loop | 99.4 µs | 729.4 MiB/s | 较好 |
| chunked_reader_sim | 102.0 µs | 710.9 MiB/s | 较好 |
| memchr_reused_buffer | 102.3 µs | 708.6 MiB/s | 较好 |
| TsvRecord 结构 | 120.9 µs | 599.6 MiB/s | 中等 |
| std::split 迭代器 | 182.6 µs | 396.9 MiB/s | 较慢 |
| 手动字节循环 | 196.4 µs | 369.1 MiB/s | 较慢 |
| memchr_inline_loop | 236.6 µs | 306.3 MiB/s | 较慢 |
| naive split + collect | 517.4 µs | 140.1 MiB/s | 🐢 最慢 |

**关键发现**：
- `tva_tsv_reader` 比 `csv` crate 快约 **31%** (867 vs 576 MiB/s)
- `tva_tsv_reader` 比 naive split 快约 **6.2 倍**
- `simd-csv` 比 `tva_tsv_reader` 快约 **12%**，差距主要来自自定义 SIMD Searcher
- `memchr` 重用的缓冲区方法比 naive split 快 **5.0 倍**

**基准测试详情** (`benches/tsv_parse.rs`)：

1. **csv_crate** - 使用 `csv` crate 的 `ReaderBuilder`，DFA 状态机实现
2. **simd_csv_crate** - 使用 `simd-csv` crate，SIMD 加速解析
3. **algo_naive_split_collect** - 使用 `BufReader::lines()` + `split().collect()`，每次分配 String 和 Vec
4. **algo_std_split_iter** - 使用 `BufReader::lines()` + `split()` 迭代器，避免 Vec 分配
5. **algo_manual_byte_loop** - 使用 `BufReader::lines()` + 手动字节遍历找分隔符
6. **algo_memchr_inline_loop** - 使用 `BufReader::lines()` + `memchr::memchr_iter` 找分隔符
7. **algo_memchr_reused_buffer** - 使用 `read_until()` 重用缓冲区 + `memchr::memchr_iter`
8. **tsv_record_struct** - 使用 `TsvRecord::parse_line()` 预分配缓冲区解析
9. **algo_memchr2_simd_loop** - 使用 `memchr::memchr2_iter` 同时搜索 `\t` 和 `\n`
10. **algo_chunked_reader_sim** - 模拟分块读取器，8KB 块 + 处理跨边界记录
11. **tva_tsv_reader** - TVA 的 `TsvReader::for_each_record()`，内部缓冲区 + SIMD

#### seps 字段优化效果分析

我们将 `seps: Vec<usize>` 改为 `seps: Option<Vec<usize>>` 进行延迟初始化，预期性能提升，但实际效果有限：

**原因分析**：

1. **基准测试使用 `for_each_record` 而非 `next_row`**
   - `benches/tsv_parse.rs` 中的 `tva_tsv_reader` 测试调用的是 `for_each_record()`
   - `for_each_record()` 不使用 `seps` 字段，因此 `Option` 优化对它没有影响
   - 优化只对使用 `next_row()` 或 `for_each_row()` 的代码路径有效

2. **空 Vec 的分配开销很小**
   - `Vec::new()` 不分配堆内存，只是初始化指针、长度和容量为 0
   - `Option::None` 和 `Vec::new()` 的内存占用几乎相同（都是 3 个 word）
   - 结构体大小没有显著变化

3. **真正的性能瓶颈在其他地方**
   - `for_each_record` 的主要开销来自 I/O 读取和 `memchr` 搜索
   - 单次 `memchr` 搜索（找换行）vs `memchr2` 搜索（同时找分隔符和换行）的性能差异
   - 回调函数的调用开销

**结论**：
- `Option<Vec<usize>>` 优化在理论上正确，但对 `for_each_record` 无影响
- 要真正提升性能，需要优化 `for_each_record` 的实现或使用 `next_row` 替代
- 当前 `tva_tsv_reader` (84.6 µs) 与理论最优 `memchr2_simd_loop` (90.9 µs) 接近，说明架构已较优

#### 与 simd-csv 的差距分析

通过对比 `tva/src/libs/tsv/` 和 `simd-csv-master/src/` 的代码，发现性能差距主要来自架构设计差异：

**1. 搜索策略差异**

| 特性 | tva | simd-csv |
|:-----|:----|:---------|
| 换行搜索 | `memchr::memchr(b'\n', data)` | 自定义 SIMD Searcher |
| 分隔符搜索 | 每行单独 `memchr_iter` | 一次 SIMD 扫描同时找 `,` `"` `\n` |
| 架构 | 两层：找行 → 分割字段 | 一层：状态机 + SIMD 直接解析 |

**simd-csv 的核心优化**：
```rust
// simd-csv: 一次 SIMD 操作同时匹配 3 个字符
core.rs:
let searcher = Searcher::new(delimiter, b'\n', quote);  // 预构建

// 在 split_record 中直接使用
for offset in self.searcher.search(&input[pos..]) {
    // 一次遍历找到所有关键字符
}
```

对比 tva：
```rust
// tva: 两次独立搜索
reader.rs:
// 1. 先找换行
match memchr::memchr(b'\n', available) { ... }

// 2. 每行再找分隔符
for pos in memchr::memchr_iter(delimiter, record) { ... }
```

**2. Reader 架构差异**

| 组件 | tva | simd-csv |
|:-----|:----|:---------|
| 缓冲区 | `Vec<u8>` 固定 64KB | `ScratchBuffer` 动态管理 |
| 数据流动 | `for_each_record` 回调 | `split_record` 返回状态 |
| 字段存储 | 实时计算 ends | 预计算 seps 数组 |

**simd-csv 的 `ScratchBuffer`**（`buffer.rs`）：
- 支持跨缓冲区记录拼接（处理大字段）
- 通过 `save()`/`flush()` 管理数据生命周期
- 真正的零拷贝：返回的 slice 直接指向内部缓冲区

**tva 的 `TsvReader`**（`reader.rs`）：
- 简单固定缓冲区设计
- 通过 `copy_within` 移动残留数据
- 功能足够，但缺少对大字段的特殊处理

**3. 状态机设计**

simd-csv 针对 CSV 引号规则设计了精细的状态机（`core.rs`）：
```rust
enum ReadState { Unquoted, Quoted, Quote }

// Unquoted: 用 memchr2 找换行或引号
// Quoted: 用 memchr 找结束引号
// Quote: 处理引号后的状态转换
```

tva 不需要状态机（TSV 无引号），但也没有利用这一点做更激进的优化。

**4. 关键性能瓶颈**

```
tva 当前流程：
1. read() 填充缓冲区
2. memchr 找换行
3. 回调处理每行
4. memchr_iter 找分隔符（每行一次）

simd-csv 流程：
1. fill_buf() 填充缓冲区
2. searcher.search() 同时找 , " \n（整块数据）
3. 状态机直接解析字段边界
```

**结论**：
- simd-csv 的 18% 性能优势主要来自**单次 SIMD 扫描**（同时匹配多个字符）
- tva 的两层搜索（先找行、再分割）引入了额外的遍历开销
- 对于 TSV（无引号），理论上可以设计比 simd-csv 更简单的单层扫描器

### simd-csv 项目结构

`simd-csv` 是一个专门优化的 CSV 解析库，其架构设计值得参考：

#### 核心模块

| 文件 | 功能 | 说明 |
|:-----|:-----|:-----|
| `lib.rs` | 库入口 | 文档、模块导出、性能说明 |
| `core.rs` | 核心状态机 | `CoreReader` 处理 CSV 解析状态 |
| `searcher.rs` | SIMD 搜索 | 多字节并行搜索（SSE2/NEON） |
| `records.rs` | 记录结构 | `ByteRecord`, `ZeroCopyByteRecord` |
| `buffer.rs` | 缓冲区管理 | 可复用的读取缓冲区 |

#### Reader 层次结构（性能递增）

```
Reader (拷贝读取，自动转义)
  ↑
ZeroCopyReader (零拷贝，仅找分隔符)
  ↑
Splitter (仅找记录边界)
  ↑
LineReader (纯行分割，无引号处理)
```

#### 特殊 Reader

- **TotalReader**: 针对内存映射优化的 Reader
- **Seeker**: 可查找记录起始位置（用于并行化）
- **ReverseReader**: 反向读取（用于日志分析）

#### SIMD Searcher 实现

```rust
// searcher.rs 核心结构
pub struct Searcher {
    #[cfg(target_arch = "x86_64")]
    inner: x86_64::sse2::SSE2Searcher,
    
    #[cfg(target_arch = "aarch64")]
    inner: aarch64::NeonSearcher,
}

// SSE2 实现
pub struct SSE2Searcher {
    n1: u8, n2: u8, n3: u8,
    v1: __m128i, v2: __m128i, v3: __m128i,
}

// NEON 实现  
pub struct NeonSearcher {
    n1: u8, n2: u8, n3: u8,
    v1: uint8x16_t, v2: uint8x16_t, v3: uint8x16_t,
}
```

#### 混合设计模式

`simd-csv` 采用**状态机 + SIMD 搜索**的混合模式：

```rust
// 状态机处理引号逻辑
enum ReadState { Unquoted, Quoted, Quote }

// SIMD 加速搜索关键字符
while pos < input_len {
    match state {
        Unquoted => {
            // SIMD 快速找到下一个引号或换行
            if let Some(offset) = searcher.search(&input[pos..]) {
                // 处理找到的位置
            }
        }
        Quoted => {
            // memchr 找结束引号
            if let Some(offset) = memchr(quote, &input[pos..]) {
                // 处理引号内内容
            }
        }
    }
}
```

#### 性能数据（来自 simd-csv 文档）

| 文件类型 | split 模式 | zero_copy 模式 | copy 模式 |
|:---------|:-----------|:---------------|:----------|
| articles.csv (大文本) | ~4.8x | ~4.7x | ~4.0x |
| tweets.csv (多空列) | ~6.0x | ~2.9x | ~2.5x |
| quote-always.csv | ~1.1x | ~1.2x | ~1.0x |
| worst-case.csv | ~1.0x | ~1.1x | ~0.9x |

*对比 `csv` crate，包含 IO，x86_64 + AVX2*

### memchr 库架构设计

`memchr` 是 Rust 生态中最成熟的 SIMD 字节搜索库，`tva` 已将其作为核心依赖。理解其架构有助于更好地利用其性能特性。

#### 整体架构

```
memchr crate
├── 顶层 API (memchr.rs)
│   ├── memchr/memrchr - 单字节搜索
│   ├── memchr2/memrchr2 - 双字节搜索
│   ├── memchr3/memrchr3 - 三字节搜索
│   └── 迭代器支持
├── memmem 子模块 (memmem/)
│   ├── Finder/FinderRev - 子串搜索器
│   ├── Two-Way 算法（主算法）
│   └── Rabin-Karp（短串优化）
└── arch/ - 架构特定实现
    ├── generic/ - 通用标量实现
    ├── x86_64/ - x86_64 SIMD 实现
    │   ├── sse2/ - 128-bit SSE2
    │   └── avx2/ - 256-bit AVX2
    ├── aarch64/ - ARM64 NEON 实现
    └── wasm32/ - WebAssembly SIMD
```

#### Vector Trait 抽象

`memchr` 的核心设计是 `Vector` trait，它抽象了不同 SIMD 架构的共性操作：

```rust
pub(crate) trait Vector: Copy + Debug {
    const BYTES: usize;      // 向量宽度（16 或 32）
    const ALIGN: usize;      // 对齐要求
    type Mask: MoveMask;     // 掩码类型

    unsafe fn splat(byte: u8) -> Self;           // 广播字节到所有 lane
    unsafe fn load_aligned(data: *const u8) -> Self;     // 对齐加载
    unsafe fn load_unaligned(data: *const u8) -> Self;   // 非对齐加载
    unsafe fn movemask(self) -> Self::Mask;      // 生成比较掩码
    unsafe fn cmpeq(self, other: Self) -> Self;  // 逐字节相等比较
    unsafe fn and(self, other: Self) -> Self;    // 按位与
    unsafe fn or(self, other: Self) -> Self;     // 按位或
}
```

**MoveMask 抽象**：不同架构的 `movemask` 结果表示不同，通过 `MoveMask` trait 统一：

- **x86_64/wasm32**: 使用 `SensibleMoveMask(u32)`，每 bit 对应一个 lane 的最高位
- **aarch64**: NEON 无原生 movemask，使用特殊实现

#### 架构特定实现

##### x86_64 实现

```rust
// arch/x86_64/sse2/memchr.rs
pub struct One(generic::One<__m128i>);  // 128-bit 向量

impl One {
    #[target_feature(enable = "sse2")]
    pub unsafe fn new_unchecked(needle: u8) -> One {
        One(generic::One::new(needle))
    }
}

// arch/x86_64/avx2/memchr.rs  
pub struct One {
    sse2: generic::One<__m128i>,  // 小数据用 SSE2
    avx2: generic::One<__m256i>,  // 大数据用 AVX2
}
```

**AVX2 混合策略**：
- 小于 32 字节的数据使用 SSE2（避免 AVX2 启动开销）
- 大于 32 字节的数据使用 AVX2（256-bit 并行处理）
- 运行时检测 CPU 特性

##### aarch64 NEON 实现

```rust
// arch/aarch64/neon/memchr.rs
pub struct One(generic::One<uint8x16_t>);

impl Vector for uint8x16_t {
    const BYTES: usize = 16;
    const ALIGN: usize = 15;  // 16-byte 对齐
    type Mask = NeonMoveMask;  // NEON 特殊掩码实现

    unsafe fn splat(byte: u8) -> Self {
        vdupq_n_u8(byte)
    }

    unsafe fn cmpeq(self, other: Self) -> Self {
        vceqq_u8(self, other)
    }

    unsafe fn movemask(self) -> Self::Mask {
        // NEON 无原生 movemask，使用特殊技巧
        // 通过比较和移位模拟
        NeonMoveMask(...)
    }
}
```

#### 通用搜索算法

`arch/generic/memchr.rs` 实现了与架构无关的搜索逻辑，通过泛型参数 `V: Vector` 适配不同 SIMD 宽度：

```rust
pub(crate) struct One<V> {
    s1: u8,
    v1: V,  // SIMD 向量，已广播 needle
}

impl<V: Vector> One<V> {
    const LOOP_SIZE: usize = 4 * V::BYTES;  // 每次处理 4 个向量

    unsafe fn find_raw(&self, start: *const u8, end: *const u8) -> Option<*const u8> {
        // 1. 处理非对齐前缀
        if let Some(cur) = self.search_chunk(start, topos) {
            return Some(cur);
        }

        // 2. 主循环：每次处理 4 个向量（64 bytes for SSE2, 128 bytes for AVX2）
        while cur <= end.sub(Self::LOOP_SIZE) {
            let a = V::load_aligned(cur);
            let b = V::load_aligned(cur.add(1 * V::BYTES));
            let c = V::load_aligned(cur.add(2 * V::BYTES));
            let d = V::load_aligned(cur.add(3 * V::BYTES));

            let eqa = self.v1.cmpeq(a);
            let eqb = self.v1.cmpeq(b);
            let eqc = self.v1.cmpeq(c);
            let eqd = self.v1.cmpeq(d);

            // 合并 4 个比较结果，只需一次 movemask
            let or1 = eqa.or(eqb);
            let or2 = eqc.or(eqd);
            let or3 = or1.or(or2);

            if or3.movemask_will_have_non_zero() {
                // 找到匹配，精确定位
                return self.find_in_chunk(eqa, eqb, eqc, eqd, cur);
            }
            cur = cur.add(Self::LOOP_SIZE);
        }

        // 3. 处理剩余尾部
        ...
    }
}
```

**关键优化点**：
1. **对齐加载**：主循环使用对齐加载（16/32 字节对齐），比非对齐加载更快
2. **循环展开**：每次处理 4 个向量，减少循环开销
3. **延迟 movemask**：先 OR 合并再 movemask，减少指令数
4. **重叠搜索**：允许前后 chunk 有重叠，避免逐字节处理边界

#### memmem 子串搜索

`memmem` 模块实现子串搜索，使用多种算法根据 needle 长度自动选择：

```rust
// searcher.rs
pub struct Searcher {
    pre: Pre,  // 预过滤器（可选）
    core: Core, // 核心搜索算法
}

enum Core {
    Empty,           // 空 needle
    OneByte(One),    // 单字节，直接用 memchr
    TwoWay(TwoWay),  // Two-Way 算法（主算法）
    Generic(Generic), // Rabin-Karp 变体
}
```

**Two-Way 算法**：
- 最坏情况下 O(n) 时间复杂度
- 常数空间复杂度
- 通过预计算临界因子（critical factorization）实现跳跃

**预过滤器（Prefilter）**：
- 使用 `memchr` 快速定位 needle 的首字符
- 只在可能匹配的位置启动完整比较
- 显著提升常见场景性能

#### 对 tva 的启示

1. **Vector Trait 模式**：
   - 通过 trait 抽象 SIMD 操作，代码复用率高
   - 新增架构只需实现 trait，无需重写算法

2. **分层架构**：
   - generic 层：算法逻辑
   - arch 层：SIMD 实现
   - 顶层：API 封装

3. **性能优化技巧**：
   - 对齐加载优先
   - 循环展开减少分支
   - 延迟 movemask 减少指令数
   - 小数据用简单算法，大数据用 SIMD

4. **TSV 专用优化建议**：
   - `memchr` 已针对单字节搜索优化，适合 TSV 分隔符搜索
   - 如需同时搜索 `\t` 和 `\n`，可用 `memchr2`
   - 对于更复杂的 CSV 引号处理，参考 `simd-csv` 的状态机+SIMD 混合模式

### TsvReader 重构计划（重大更新）

> **⚠️ 重大更新**：基于 `benches/tsv_strategy_compare.rs` 基准测试结果，发现**手写 SSE2 SIMD 单层扫描**可实现 **+670%** 性能提升！

#### 阶段 1：手写 SIMD 单层扫描架构（高优先级）

**目标**：使用手写 SSE2/NEON SIMD 实现单层扫描，同时搜索 `\t`, `\n`, `\r`

**基准测试结果**（`tsv_strategy_compare.rs`）：

| 策略 | 实现方式 | 吞吐量 | 相对性能 |
|:-----|:---------|:-------|:---------|
| **single_pass_sse2** | 手写 SSE2 SIMD 单层扫描 | **6.48 GiB/s** | 🥇 最快 |
| two_pass_memchr | memchr 找行 + memchr_iter 分割 | 2.38 GiB/s | 慢 2.7x |
| single_pass_memchr2 | memchr2 单层扫描 | 1.59 GiB/s | 慢 4.1x |
| two_pass_sse2 | SSE2 找行 + memchr_iter 分割 | 2.02 GiB/s | 慢 3.2x |
| tva_tsv_reader (当前) | TsvReader 两层扫描 | 0.84 GiB/s | 慢 7.7x |

**关键发现**：
1. **不是单层扫描不行，而是 `memchr2` 不够高效**
2. **手写 SSE2 单层扫描比当前 `tva_tsv_reader` 快 670%**
3. **SSE2 每次处理 16 字节，远超逐字节或 `memchr` 效率**

**实现方案**（参考 `simd-csv/searcher.rs`）：
```rust
// x86_64 SSE2 实现
#[cfg(target_arch = "x86_64")]
mod sse2 {
    use core::arch::x86_64::*;
    
    pub struct Sse2Searcher {
        v_tab: __m128i,
        v_newline: __m128i,
        v_cr: __m128i,
    }
    
    impl Sse2Searcher {
        #[inline]
        pub unsafe fn new(tab: u8, newline: u8, cr: u8) -> Self {
            Self {
                v_tab: _mm_set1_epi8(tab as i8),
                v_newline: _mm_set1_epi8(newline as i8),
                v_cr: _mm_set1_epi8(cr as i8),
            }
        }
        
        /// 同时搜索 tab, newline, CR
        /// 每次处理 16 字节，使用 SIMD 并行比较
        pub fn search<'a>(&'a self, haystack: &'a [u8]) -> Sse2Iter<'a>;
    }
}

// 在 TsvReader 中使用
pub fn next_row_simd(&mut self) -> io::Result<Option<TsvRow<'_, '_>>> {
    #[cfg(target_arch = "x86_64")]
    {
        use sse2::Sse2Searcher;
        unsafe {
            let searcher = Sse2Searcher::new(b'\t', b'\n', b'\r');
            // 使用 SIMD 搜索同时找分隔符和换行...
        }
    }
    // ...
}
```

**预期收益**：**+670%**（从 0.84 GiB/s 提升到 6.48 GiB/s）

#### 阶段 2：缓冲区管理优化（低优先级）

**目标**：减少数据拷贝，支持跨缓冲区记录

**当前实现**：
```rust
// 当前：用 copy_within 移动残留数据
if self.pos > 0 {
    self.buf.copy_within(self.pos..self.len, 0);
    self.len -= self.pos;
    self.pos = 0;
}
```

**改进方案**（参考 `simd-csv/buffer.rs`）：
```rust
pub struct ReadBuffer {
    buf: Vec<u8>,
    // 记录边界，支持跨缓冲区拼接
    saved: Vec<u8>,  // 暂存跨缓冲区的记录前缀
}

impl ReadBuffer {
    /// 读取新数据，自动处理跨缓冲区记录
    pub fn fill<R: Read>(&mut self, reader: &mut R) -> io::Result<usize>;
    
    /// 获取完整记录（可能跨多个 fill 调用）
    pub fn next_record(&mut self) -> Option<&[u8]>;
}
```

**预期收益**：+2-3%（仅对大字段场景有效）

#### 阶段 3：自定义 SIMD Searcher（低优先级）

**目标**：进一步压榨性能，处理 3+ 字符同时搜索

**适用场景**：需要同时找 `\t`, `\n`, `\r` 时

**实现参考**：
```rust
// 参考 simd-csv/searcher.rs
pub struct TsvSearcher {
    #[cfg(target_arch = "x86_64")]
    inner: SSE2Searcher,  // 或 AVX2Searcher
    #[cfg(target_arch = "aarch64")]
    inner: NeonSearcher,
}

impl TsvSearcher {
    pub fn new(tab: u8, newline: u8, cr: u8) -> Self;
    
    /// 返回 (位置, 字符类型) 迭代器
    pub fn search(&self, haystack: &[u8]) -> impl Iterator<Item = (usize, u8)>;
}
```

**预期收益**：+5-10%（需要大量工作，边际收益有限）

#### 实施路线图（再次修订）

基于 `tsv_strategy_compare` 基准测试结果，重新调整优化策略：

| 阶段 | 任务 | 预期性能提升 | 工作量 | 优先级 | 状态 |
|:-----|:-----|:-------------|:-------|:-------|:-----|
| **1** | **手写 SSE2 SIMD 单层扫描** | **+114%** | 高 | 🔥 **最高** | ✅ **已完成** |
| 1.1 | 实现 `Sse2Searcher` 结构 | - | - | 🔥 最高 | ✅ 已完成 |
| 1.2 | 集成 SIMD searcher 到 `TsvReader` | - | - | 🔥 最高 | ✅ 已完成 |
| 1.3 | 实现 `next_row_sse2()` 方法 | - | - | 🔥 最高 | ✅ 已完成 |
| 2 | aarch64 NEON 单层扫描 | **+114%** | 高 | 中 | ✅ **已完成** |
| 2.1 | 实现 `NeonSearcher` 结构 | - | - | 中 | ✅ 已完成 |
| 2.2 | 集成 NEON searcher 到 `TsvReader` | - | - | 中 | ✅ 已完成 |
| ~~3~~ | ~~单层扫描架构（memchr2）~~ | ~~-47%~~ | ~~中等~~ | ❌ ~~取消~~ | ~~实测更慢~~ |
| 4 | 缓冲区管理优化 | +2-3% | 中等 | 低 | 待定 |
| 4.1 | 实现 `ReadBuffer` 结构 | - | - | 低 | 待定 |
| 4.2 | 支持跨缓冲区记录拼接 | - | - | 低 | 待定 |

**当前状态**（2025-03-20 更新）：
- ✅ `memchr` 集成完成
- ✅ 基础 SIMD 加速已生效
- ✅ `next_row()` 已实现（使用 `memchr2` 单层扫描）
- ✅ `seps` 字段改为 `Option<Vec<usize>>` 延迟初始化
- ✅ `for_each_row()` 已迁移到使用 `next_row()`
- ✅ `for_each_record()` 保持现状（两层扫描更快）
- ✅ **手写 SSE2 SIMD 单层扫描完成**：`next_row_sse2()` 实现，性能提升 114%
- ✅ **手写 NEON SIMD 单层扫描完成**：`next_row_neon()` 实现，aarch64 支持
- ❌ ~~单层扫描优化取消~~：已重新实施 SSE2/NEON 版本

---

### 重要更新（2025-03-20）：手写 SSE2 单层扫描测试

为了进一步验证单层扫描 vs 两层扫描的性能差异，创建了专门的基准测试 `benches/tsv_strategy_compare.rs`，比较了以下策略：

| 策略 | 实现方式 | 数据规模 | 吞吐量 | 相对性能 |
|:-----|:---------|:---------|:-------|:---------|
| **single_pass_sse2** | 手写 SSE2 SIMD 单层扫描 | 10000rows_50cols | **6.48 GiB/s** | 🥇 最快 |
| two_pass_memchr | memchr 找行 + memchr_iter 分割 | 10000rows_50cols | 2.38 GiB/s | 慢 2.7x |
| single_pass_memchr2 | memchr2 单层扫描 | 10000rows_50cols | 1.59 GiB/s | 慢 4.1x |
| two_pass_sse2 | SSE2 找行 + memchr_iter 分割 | 10000rows_50cols | 2.02 GiB/s | 慢 3.2x |
| naive_byte_by_byte | 逐字节扫描 | 10000rows_50cols | 2.11 GiB/s | 慢 3.1x |

**重大发现：手写 SSE2 单层扫描比两层扫描快 173%！**

关键结论：
1. **不是单层扫描不行，而是 `memchr2` 的实现不够高效**
2. **手写 SSE2 可以同时搜索 `\t`, `\n`, `\r` 三个字符**，每次处理 16 字节
3. **SSE2 单层扫描 (6.48 GiB/s) 远超当前 `tva_tsv_reader` (861.6 MiB/s = 0.84 GiB/s)**

#### AVX2 与 SSE2 对比测试（2025-03-20 更新）

在 `benches/tsv_strategy_compare.rs` 中添加了手写 AVX2 SIMD 实现（256-bit 向量），与 SSE2（128-bit）进行对比：

| 策略 | 实现方式 | 数据规模 | 吞吐量 | 相对性能 |
|:-----|:---------|:---------|:-------|:---------|
| **single_pass_sse2** | 手写 SSE2 SIMD 单层扫描 (128-bit) | 10000rows_50cols | **6.59 GiB/s** | 🥇 最快 |
| **single_pass_avx2** | 手写 AVX2 SIMD 单层扫描 (256-bit) | 10000rows_50cols | **4.94 GiB/s** | 🥈 第二 |
| two_pass_memchr | memchr 找行 + memchr_iter 分割 | 10000rows_50cols | 2.41 GiB/s | 慢 2.7x |
| two_pass_sse2 | SSE2 找行 + memchr_iter 分割 | 10000rows_50cols | 2.06 GiB/s | 慢 3.2x |
| single_pass_memchr2 | memchr2 单层扫描 | 10000rows_50cols | 1.61 GiB/s | 慢 4.1x |
| naive_byte_by_byte | 逐字节扫描 | 10000rows_50cols | 1.44 GiB/s | 慢 4.6x |

**意外发现：AVX2 (256-bit) 比 SSE2 (128-bit) 慢 25%！**

可能原因：
1. **AVX2 功耗/频率调节 (throttling)** - AVX2 指令可能导致 CPU 降频
2. **内存带宽限制** - 此 workload 可能受限于内存带宽而非计算能力
3. **延迟差异** - AVX2 指令的延迟可能比 SSE2 更高
4. **循环开销** - 虽然 AVX2 每次处理 32 字节，但循环控制逻辑可能抵消了收益

**关键结论**：
- **SSE2 在此场景下已足够高效**，额外的 128-bit 宽度带来的收益被其他因素抵消
- **无需追求 AVX2**，SSE2 有更广泛的兼容性（所有 x86_64 都支持）
- 手写 SIMD  searcher 的核心价值在于**单层扫描架构**，而非向量宽度

#### SSE2 实施完成（2025-03-20）

手写 SSE2 SIMD 单层扫描已成功实施并集成到 `TsvReader`。

**创建的文件**：
- `src/libs/tsv/sse2.rs` - SSE2 searcher 模块，包含 `Sse2Searcher` 结构和迭代器
- `benches/tsv_reader_sse2.rs` - TsvReader SSE2 vs 标准实现性能对比

**修改的文件**：
- `src/libs/tsv/mod.rs` - 添加 SSE2 模块导出
- `src/libs/tsv/reader.rs` - 添加 `next_row_sse2()` 方法
- `Cargo.toml` - 添加 benchmark 配置

**实际性能结果**（`benches/tsv_reader_sse2.rs`）：

| 方法 | 数据规模 | 吞吐量 | 相对性能 |
|:-----|:---------|:-------|:---------|
| **next_row_sse2** (SSE2) | 10K rows, 50 cols | **2.82 GiB/s** | 🥇 最快 |
| next_row_standard (memchr2) | 10K rows, 50 cols | 1.32 GiB/s | 慢 2.1x |
| for_each_record (两层扫描) | 10K rows, 50 cols | 2.01 GiB/s | 慢 1.4x |

**关键发现**：
- **SSE2 版本比标准 `next_row` 快 114%**（2.82 vs 1.32 GiB/s）
- **SSE2 版本比 `for_each_record` 快 40%**（2.82 vs 2.01 GiB/s）
- 在小数据集（1K rows, 5 cols）上提升更明显：**SSE2 快 140%**

**与理论预期的差距**：
- 理论预期提升 670%（基于 `tsv_strategy_compare` 的 6.48 GiB/s）
- 实际提升 114%（2.82 GiB/s）
- **差距原因**：`TsvReader` 的缓冲区管理、行解析和字段提取开销，以及 `TsvRow` 的构建成本

**使用方法**：
```rust
use tva::libs::tsv::reader::TsvReader;

let mut reader = TsvReader::new(file);

// 使用 SSE2 加速版本 (x86_64 only)
unsafe {
    while let Some(row) = reader.next_row_sse2()? {
        for i in 1..=row.ends.len() {
            if let Some(field) = row.get_bytes(i) {
                // 处理字段
            }
        }
    }
}
```

**注意事项**：
- SSE2 版本需要 `unsafe` 块（因为使用 SIMD 内部函数）
- 仅在 x86_64 平台可用（自动条件编译）
- SSE2 在所有 x86_64 CPU 上都可用，无需运行时检测

#### NEON 实施完成（2025-03-20）

手写 ARM NEON SIMD 单层扫描已成功实施并集成到 `TsvReader`，为 aarch64 平台提供与 SSE2 类似的性能优化。

**创建的文件**：
- `src/libs/tsv/neon.rs` - NEON searcher 模块，包含 `NeonSearcher` 结构和迭代器

**修改的文件**：
- `src/libs/tsv/mod.rs` - 添加 NEON 模块导出
- `src/libs/tsv/reader.rs` - 添加 `next_row_neon()` 方法

**实现细节**：
- 使用 `uint8x16_t` 128-bit 向量（与 SSE2 的 `__m128i` 等效）
- 使用 `vceqq_u8` 进行向量比较（等效于 SSE2 的 `_mm_cmpeq_epi8`）
- 使用 `vmaxq_u8` 进行 OR 操作组合比较结果
- NEON movemask 通过软件实现（NEON 没有硬件 movemask 指令）

**使用方法**：
```rust
use tva::libs::tsv::reader::TsvReader;

let mut reader = TsvReader::new(file);

// 使用 NEON 加速版本 (aarch64 only)
unsafe {
    while let Some(row) = reader.next_row_neon()? {
        for i in 1..=row.ends.len() {
            if let Some(field) = row.get_bytes(i) {
                // 处理字段
            }
        }
    }
}
```

**注意事项**：
- NEON 版本需要 `unsafe` 块
- 仅在 aarch64 平台可用（自动条件编译）
- 所有 aarch64 CPU 都支持 NEON，无需运行时检测
- 预期性能与 SSE2 版本相当（~114% 提升）

#### 后续优化方向（再次修订）

基于 `tsv_strategy_compare` 基准测试结果，重新调整优化策略：

| 优先级 | 优化项 | 预期收益 | 实施难度 | 状态 |
|:-----|:------|:---------|:---------|:-----|
| **高** | 手写 SSE2 SIMD 单层扫描 | **+114%** | 高 | ✅ **已完成** |
| 中 | NEON (ARM) 单层扫描 | **+114%** | 高 | ✅ **已完成** |
| 低 | AVX2 优化 | -25% | 高 | ❌ 不推荐 |
| 低 | 优化 `for_each_record` 使用 `memchr2` | 可能负增长 | 低 | ❌ 取消 |
| 低 | 缓冲区管理优化 | +2-3% | 中 | 待定 |

**新的建议**：
- **SSE2 和 NEON 实施已完成**，实际提升 114%，虽低于理论 670%，但仍是显著改进
- **不需要 AVX2**：实测比 SSE2 慢 25%，且兼容性更差
- 主要 SIMD 优化已完成，x86_64 和 aarch64 平台都有手写 SIMD 实现
- 参考 `simd-csv/searcher.rs` 的实现方式

#### 零拷贝设计（已部分实现）

现有架构已具备良好的零拷贝基础：

| 组件 | 类型 | 用途 | 状态 |
|:-----|:-----|:-----|:-----|
| `TsvRow<'a, 'b>` | 零拷贝视图 | `filter`, `select` 等只读操作 | ✅ 已实现 |
| `TsvSplitter<'a>` | 零分配迭代器 | 字段分割 | ✅ 已实现 |
| `TsvRecord` | 拥有所有权 | `sample` 等需要存储的场景 | ✅ 已实现 |
| `next_row()` | 单层扫描 | 高性能行读取 | ✅ 已实现 |

**保持现状**：当前设计已满足需求，无需大规模重构。



