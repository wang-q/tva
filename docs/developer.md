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

测试数据：3000 行 TSV，每行 5 个字段

| 实现 | 时间 | 吞吐量 | 相对性能 |
|:-----|:-----|:-------|:---------|
| **simd-csv** | 61.6 µs | **1.15 GiB/s** | 🥇 最快 |
| **tva_tsv_reader** | 72.9 µs | **993 MiB/s** | 🥈 快 (比 simd-csv 慢 18%) |
| csv crate | 102.4 µs | 708 MiB/s | 中等 |
| memchr2 (多字符) | 90.0 µs | 805 MiB/s | 较好 |
| memchr + 重用缓冲区 | 91.2 µs | 795 MiB/s | 较好 |
| TsvRecord 结构 | 105.3 µs | 688 MiB/s | 中等 |
| std::split 迭代器 | 182.9 µs | 396 MiB/s | 较慢 |
| 手动字节循环 | 170.0 µs | 426 MiB/s | 较慢 |
| naive split + collect | 436.0 µs | 166 MiB/s | 🐢 最慢 |

**关键发现**：
- `simd-csv` 比 `tva_tsv_reader` 快约 **18%**，差距主要来自自定义 SIMD Searcher
- `tva_tsv_reader` 比 `csv` crate 快约 **40%**
- `memchr` 重用的缓冲区方法比 naive split 快 **4.8 倍**

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

### TsvReader 重构计划

基于与 `simd-csv` 的对比分析，制定以下重构方案，目标是将 TSV 解析性能提升 15-20%，接近 simd-csv 水平。

#### 阶段 1：单层扫描架构（高优先级）

**目标**：消除两层遍历（找行 + 分割字段）的开销

**当前问题**：
```rust
// reader.rs 当前实现 - 两次遍历
for_each_record(|record| {          // 第1次：memchr 找 \n
    for pos in memchr_iter(\t, record) {  // 第2次：每行 memchr_iter 找 \t
        ...
    }
})
```

**改进方案**：
```rust
// 新架构 - 单次遍历
pub struct TsvReader<R> {
    reader: R,
    buf: Vec<u8>,
    // 新增：预计算的分隔符位置缓存
    seps: Vec<usize>,
    // 新增：当前记录在缓冲区中的范围
    record_start: usize,
    record_end: usize,
}

impl<R: Read> TsvReader<R> {
    /// 同时找 \n 和 \t，一次遍历完成
    pub fn next_row(&mut self) -> Option<TsvRow> {
        // 1. 从当前位置开始扫描
        // 2. 用 memchr2 找 \n 或 \t
        // 3. 遇到 \t 记录位置到 seps
        // 4. 遇到 \n 返回 TsvRow
    }
}
```

**关键技术点**：
- 使用 `memchr2(b'\t', b'\n', data)` 同时搜索两个字符
- 维护 `seps` 数组缓存分隔符位置，避免重复扫描
- `TsvRow` 直接引用 `seps` 切片，无需重新计算

#### 阶段 2：缓冲区管理优化（中优先级）

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

**预期收益**：在阶段 1 基础上再提升 5-8%

#### 实施路线图

| 阶段 | 任务 | 预期性能提升 | 工作量 | 优先级 |
|:-----|:-----|:-------------|:-------|:-------|
| 1 | 单层扫描架构（memchr2） | +12-15% | 中等 | **高** |
| 1.1 | 重构 `TsvReader` 核心循环 | - | - | 高 |
| 1.2 | 更新 `TsvRow` 以支持预计算 seps | - | - | 高 |
| 1.3 | 更新所有使用 `for_each_record` 的调用点 | - | - | 中 |
| 2 | 缓冲区管理优化 | +3-5% | 中等 | 中 |
| 2.1 | 实现 `ReadBuffer` 结构 | - | - | 中 |
| 2.2 | 支持跨缓冲区记录拼接 | - | - | 低 |
| 3 | 自定义 SIMD Searcher | +5-8% | 高 | 低 |
| 3.1 | x86_64 SSE2 实现 | - | - | 低 |
| 3.2 | aarch64 NEON 实现 | - | - | 低 |

**当前状态**：
- ✅ `memchr` 集成完成
- ✅ 基础 SIMD 加速已生效
- ⏳ 阶段 1 准备开始（单层扫描架构）

#### 阶段 1 详细设计

**API 设计**：
```rust
pub struct TsvReader<R> {
    reader: R,
    buf: Vec<u8>,
    pos: usize,
    len: usize,
    eof: bool,
    // 新增：分隔符位置缓存（复用分配）
    seps: Vec<usize>,
}

impl<R: Read> TsvReader<R> {
    /// 读取下一行，返回 TsvRow（零拷贝视图）
    pub fn next_row(&mut self) -> io::Result<Option<TsvRow>> {
        self.seps.clear();
        
        loop {
            let available = &self.buf[self.pos..self.len];
            
            // 使用 memchr2 同时找 \t 和 \n
            let mut iter = memchr::Memchr2::new(b'\t', b'\n', available);
            
            while let Some(offset) = iter.next() {
                let byte = available[offset];
                let abs_pos = self.pos + offset;
                
                if byte == b'\t' {
                    // 记录分隔符位置
                    self.seps.push(abs_pos);
                } else {
                    // 找到换行，返回行
                    self.seps.push(abs_pos); // 包含换行位置作为结束标记
                    let row = self.make_row();
                    self.pos = abs_pos + 1;
                    return Ok(Some(row));
                }
            }
            
            // 需要更多数据
            if self.eof {
                // 处理最后无换行的数据
                if self.pos < self.len {
                    let row = self.make_row();
                    self.pos = self.len;
                    return Ok(Some(row));
                }
                return Ok(None);
            }
            
            self.fill_buffer()?;
        }
    }
    
    fn make_row(&self) -> TsvRow {
        TsvRow {
            line: &self.buf[self.record_start..self.record_end],
            ends: &self.seps,
        }
    }
}
```

**兼容性策略**：
- 保留 `for_each_record` 作为兼容层（内部调用 `next_row`）
- 逐步迁移各命令使用新 API
- 保持 `TsvRow` 接口不变，调用方无需修改

**测试策略**：
1. 单元测试：覆盖空文件、无换行结尾、大字段、跨缓冲区记录
2. 集成测试：与现有 `benches/tsv_parse.rs` 对比性能
3. 回归测试：所有现有 CLI 测试必须通过

#### 零拷贝设计（已部分实现）

现有架构已具备良好的零拷贝基础：

| 组件 | 类型 | 用途 |
|:-----|:-----|:-----|
| `TsvRow<'a, 'b>` | 零拷贝视图 | `filter`, `select` 等只读操作 |
| `TsvSplitter<'a>` | 零分配迭代器 | 字段分割 |
| `TsvRecord` | 拥有所有权 | `sample` 等需要存储的场景 |

**保持现状**：当前设计已满足需求，无需额外优化。



