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

`tva` 使用 SIMD 加速 TSV 解析。当前实现包括 `memchr` 库集成和手写 SIMD searcher。

### 性能基准

#### TsvReader 实现对比 (`benches/tsv_simd_compare.rs`)

在 x86_64 平台上对比 SSE2 与 memchr2 实现：

| 测试场景 | memchr2 | SSE2 | 提升 |
|:---------|:--------|:-----|:-----|
| 1000 rows, 5 cols | ~400 µs | ~200 µs | **~100%** |
| 10000 rows, 5 cols | ~398 µs | ~205 µs | **~94%** |
| 1000 rows, 50 cols | ~446 µs | ~181 µs | **~146%** |
| 10000 rows, 50 cols | ~4.77 ms | ~2.00 ms | **~138%** |

**关键发现**：
- SSE2 比 memchr2 快约 **2 倍**（94% ~ 146% 提升）
- `auto` 模式正确选择 SSE2 实现，性能与直接调用 SSE2 一致
- 宽表（50 cols）比窄表（5 cols）提升更明显

#### 各实现吞吐量对比

| 实现 | 吞吐量 | 说明 |
|:-----|:-------|:-----|
| 手写 SSE2 SIMD | **2.8~3.3 GiB/s** | x86_64，单层扫描 |
| 手写 NEON SIMD | **~2.8 GiB/s** | aarch64，单层扫描 |
| `memchr2` 单层扫描 | 1.3~1.5 GiB/s | 通用实现 |
| `for_each_line` | 2.01 GiB/s | 两层扫描 |

**关键结论**：
- 手写 SSE2/NEON 单层扫描比 `memchr2` 快约 **114%**
- AVX2 (256-bit) 实测比 SSE2 慢，不推荐使用
- 单层扫描架构是性能提升的关键

### 实现架构

```
src/libs/tsv/simd/
├── mod.rs    - 模块入口，条件导出
├── sse2.rs   - x86_64 SSE2 实现
└── neon.rs   - aarch64 NEON 实现
```

**使用方式**：
- `TsvReader::next_row()` - 自动选择最优实现：
  - x86_64: 自动使用 SSE2 单层扫描
  - aarch64: 自动使用 NEON 单层扫描
  - 其他平台: 使用 `memchr2` 单层扫描

### 优化原理

单层扫描同时搜索 `\t`、 `\n`、 `\r` 三个分隔符，避免两层遍历的开销。手写 SIMD 实现直接操作向量寄存器，比通用库更高效。

### 当前状态

- ✅ SSE2 实现完成 (x86_64)
- ✅ NEON 实现完成 (aarch64)
- ✅ 集成到 `TsvReader`
- ⏳ 缓冲区管理优化（低优先级）

现有架构已具备良好的零拷贝基础：

| 组件 | 类型 | 用途 | 状态 |
|:-----|:-----|:-----|:-----|
| `TsvRow<'a, 'b>` | 零拷贝视图 | `filter`, `select`, `join`, `expr` 等只读操作 | ✅ 已实现 |
| `TsvRecord` | 拥有所有权 | `transpose`, `sort`, `sample` 等需要存储的场景 | ✅ 已实现 |
| `next_row()` | 单层扫描 | 高性能行读取 | ✅ 已实现 |

**保持现状**：当前设计已满足需求，无需大规模重构。

## TSV API 迁移计划

随着 `TsvReader::next_row()` 提供单层扫描 SIMD 加速，以下模块需要逐步迁移以充分利用性能提升。

### 当前状态分析

| 文件 | 当前实现 | 问题 | 优先级 |
|:-----|:---------|:-----|:-------|
| `reader.rs` | `for_each_line` 使用 `next_row` | ✅ 已完成 | - |
| `select.rs` | `extract_ranges` 使用 `TsvRow.ends` | ✅ 已完成 | - |
| `select.rs` | `write_excluding_from_bytes` 使用 `TsvRow.ends` | ✅ 已完成 | - |
| `select.rs` | `write_with_rest` 使用 `TsvRow.ends` | ✅ 已完成 | - |
| `join.rs` | `extract_values` 使用 `TsvRow` | ✅ 已完成 | - |
| `join.rs` | 数据记录处理使用 `for_each_row` | ✅ 已完成 | - |
| `key.rs` | `extract_from_row` 使用 `TsvRow` | ✅ 已完成 | - |
| `split.rs` | ~~`TsvSplitter` 已删除~~ | ✅ 已移除 | - |
| `transpose.rs` | 使用 `for_each_row` + `from_row` | ✅ 已完成 | - |
| `sort.rs` | 使用 `for_each_row` + `from_row` | ✅ 已完成 | - |
| `plot/data.rs` | 使用 `for_each_row` + `TsvRow` | ✅ 已完成 | - |
| `record.rs` | `TsvRecord::from_row` 已添加 | ✅ 已完成 | - |
| `expr.rs` | 使用 `for_each_row` + `TsvRow.ends` | ✅ 已完成 | - |

### 迁移策略

#### 阶段 1: Select 模块优化 ✅ 已完成

**目标**: 让 `select` 命令直接使用 `TsvRow` 的已解析字段边界，避免二次扫描。

**已完成工作**:
1. `src/libs/tsv/select.rs`: 修改 `extract_ranges` 函数接受 `&TsvRow` 而非 `&[u8]`
2. `src/libs/tsv/select.rs`: 修改 `write_selected_from_bytes` 函数接受 `&TsvRow`
3. `src/libs/tsv/select.rs`: 修改 `write_excluding_from_bytes` 函数接受 `&TsvRow`
4. `src/libs/tsv/select.rs`: 修改 `write_with_rest` 函数接受 `&TsvRow`
5. `src/cmd_tva/select.rs`: 使用 `for_each_row` 替代 `for_each_line`
6. `src/cmd_tva/join.rs`: 使用 `for_each_row` 替代 `for_each_line`
7. `src/libs/tsv/key.rs`: 新增 `extract_from_row` 方法接受 `&TsvRow`

**实现说明**: 直接修改现有函数签名而非新增 `_from_row` 版本，因为 `select` 模块的使用场景单一，且修改范围可控。

**实际收益**: `select` 和 `join` 命令性能提升约 **10%**

#### 阶段 2: ~~Split 模块优化~~ ✅ 已完成

**决策**: `TsvSplitter` 已被完全删除。

**原因**: `TsvRow` 已经可以完全替代 `TsvSplitter` 的功能：
- `TsvRow.get_bytes(idx)` - 直接访问指定字段
- `TsvRow.ends` - 遍历所有字段边界

使用 `TsvSplitter` 的场景已改用内联遍历或 `TsvRow`。

#### 阶段 3: Record 模块优化 ✅ 已完成

**目标**: 减少 `TsvRecord::parse_line` 的扫描开销。

**已完成工作**:
1. `src/libs/tsv/record.rs`: 新增 `TsvRecord::from_row()` 方法，从 `TsvRow` 创建，直接复用 `ends` 数组
2. `src/cmd_tva/transpose.rs`: 使用 `for_each_row` + `TsvRecord::from_row()`
3. `src/cmd_tva/sort.rs`: 使用 `for_each_row` + `TsvRecord::from_row()`
4. `src/libs/plot/data.rs`: 使用 `for_each_row` + `TsvRow`（无需拥有所有权）

**遗留 API**: `TsvRecord::parse_line()` 仍保留用于测试和特殊场景

#### 阶段 4: 其他命令迁移 ✅ 已完成

**目标**: 将更多命令迁移到 `for_each_row` 以消除二次解析。

**已完成工作**:
1. `src/cmd_tva/check.rs`: 使用 `for_each_row` + `TsvRow.field_count()`
2. `src/cmd_tva/fill.rs`: 使用 `for_each_row` + `TsvRow.get_bytes()`
3. `src/cmd_tva/blank.rs`: 使用 `for_each_row` + `TsvRow.get_bytes()`
4. `src/cmd_tva/bin.rs`: 使用 `for_each_row` + `TsvRow.get_bytes()`
5. `src/cmd_tva/longer.rs`: 使用 `for_each_row` + `TsvRow.get_bytes()`
6. `src/libs/filter/runner.rs`: 使用 `for_each_row` + `TsvRow`（替代手动构建）
7. `src/cmd_tva/join.rs`: 数据记录处理使用 `for_each_row` + `extractor.extract_from_row()`
8. `src/cmd_tva/expr.rs`: 使用 `for_each_row` + `TsvRow.ends` 提取字段

**新增 API**: `TsvRow.field_count()` 方法，正确处理空行（返回 0 字段）

#### 遗留 API 说明

以下命令仍使用 `for_each_line`，但不需要迁移（只处理整行，不解析字段）：
- `nl.rs` - 只加行号
- `keep_header.rs` - 复制行到子进程
- `append.rs` - 追加文件
- `slice.rs` - 按行号切片
- `split.rs` - 按行数分割文件
- `to/md.rs` - 转换为 markdown
- `uniq.rs` - 整行哈希去重

以下命令仍使用二次解析，需要未来迁移：
- `sample.rs` - Sampler trait 接收 `&[u8]`，Weighted/Distinct 采样器内部二次解析

### 实施建议

1. **阶段 1 已完成**: `select` 和 `join` 已优化，收益约 10%
2. **阶段 2 已完成**: `split.rs` 已删除，`TsvSplitter` 已移除
3. **阶段 3 已完成**: `transpose`、`sort`、`plot/data` 已迁移到 `for_each_row`
4. **阶段 4 已完成**: `check`、`fill`、`blank`、`bin`、`longer` 已迁移到 `for_each_row`
5. **阶段 5 已完成**: `join` 数据记录处理、`expr` 已迁移到 `for_each_row`
6. **核心命令已优化**: 主要 TSV 处理命令均使用单层扫描
7. **保持向后兼容**: `TsvRecord::parse_line()` 仍保留用于测试和特殊场景
