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

## 已完成的工作

### SIMD 优化

- **手写 SIMD 实现**: SSE2 (x86_64) 和 NEON (aarch64) 单层扫描，达到 **1.63 GiB/s** 吞吐量
- **架构设计**: `DelimiterSearcher` trait 统一平台抽象，泛型函数消除重复代码
- **关键优化**: 仅搜索 `\t` 和 `\n`，CR 后处理减少寄存器压力

### API 演进

| 组件 | 类型 | 用途 |
|:-----|:-----|:-----|
| `TsvRow` | 零拷贝视图 | `filter`, `select`, `join`, `expr` 等只读操作 |
| `TsvRecord` | 拥有所有权 | `transpose`, `sort` 等需要存储的场景 |
| `for_each_line` | 纯行处理 | `nl`, `slice`, `uniq` 等（使用 `0xFF` 技巧避免字段分配） |

**迁移状态**: 所有核心命令已迁移到 `for_each_row`，消除二次解析。`TsvSplitter` 已移除，功能由 `TsvRow` 替代。

## 代码结构优化建议

基于对25个命令文件的代码分析，以下是改进建议：

### 2. 提取 Header 处理公共函数

**现状**: 多个命令重复类似的 header 写入逻辑：
```rust
for line in &header_info.lines {
    writer.write_all(line)?;
    writer.write_all(b"\n")?;
}
if let Some(ref column_names) = header_info.column_names_line {
    writer.write_all(column_names)?;
    writer.write_all(b"\n")?;
}
```

**建议**: 在 `libs/tsv/header.rs` 中添加辅助函数：
```rust
pub fn write_header_info(writer: &mut dyn Write, info: &HeaderInfo) -> io::Result<()>
pub fn write_header_with_suffix(writer: &mut dyn Write, info: &HeaderInfo, suffix: Option<&[u8]>) -> io::Result<()>
```

### 4. TsvRow 迭代器支持

**现状**: 命令中常见模式：
```rust
for col_idx in 0..row.field_count() {
    let bytes = row.get_bytes(col_idx + 1).unwrap_or(b"");
    // ...
}
```

**建议**: 为 `TsvRow` 添加字段迭代器：
```rust
impl TsvRow {
    pub fn iter_fields(&self) -> impl Iterator<Item = &[u8]> + '_ {
        // 返回字段迭代器，简化循环代码
    }
}
```

## 字段解析与 Header 处理改进提案

### 现状问题分析

1. **API 不一致**
   - `parse_field_list_with_header` 返回 `Result<Vec<usize>, String>`，但需要 `Option<&Header>`
   - `resolve_fields_from_header` 接受字节，但内部仍创建 `Header` 对象
   - 命令中需要手动处理 `column_names_bytes` 和 `Header` 的转换

2. **Header 处理的复杂性**
   - 有 `HeaderInfo`（来自 reader）、`Header`（字段解析）、`HeaderConfig`（CLI 参数）三个相关结构
   - `select.rs` 中甚至需要手动构建 `TsvRow` 来写 header

3. **字段解析的重复逻辑**
   - 每个命令都要写类似的模式：
     ```rust
     let indices = if let Some(ref names) = column_names_bytes {
         resolve_fields_from_header(spec, names, '\t')?
     } else {
         parse_numeric_field_list(spec)?
     };
     ```

4. **--fields 与 header 的耦合问题**
   - 有些命令需要 header 来解析字段名，有些只需要数字索引
   - `header_args_with_columns()` 和 `header_args()` 区分不清晰

### 改进方案

#### 统一字段解析 API（推荐）

创建一个统一的 `FieldResolver` 结构，封装所有解析逻辑：

```rust
// libs/tsv/fields.rs
pub struct FieldResolver {
    header_bytes: Option<Vec<u8>>,
    delimiter: char,
}

impl FieldResolver {
    pub fn new(header_bytes: Option<Vec<u8>>, delimiter: char) -> Self {
        Self { header_bytes, delimiter }
    }
    
    /// 解析字段列表，自动判断是否使用 header
    pub fn resolve(&self, spec: &str) -> Result<Vec<usize>, String> {
        match &self.header_bytes {
            Some(bytes) => resolve_fields_from_header(spec, bytes, self.delimiter),
            None => parse_numeric_field_list_preserve_order(spec),
        }
    }
    
    /// 获取 header 字段名（用于生成输出 header）
    pub fn column_names(&self) -> Option<Vec<String>> {
        self.header_bytes.as_ref().map(|bytes| {
            let s = std::str::from_utf8(bytes).ok()?;
            Some(s.split(self.delimiter).map(|f| f.to_string()).collect())
        }).flatten()
    }
}
```

**命令中使用：**
```rust
// 简化后的命令代码
let resolver = FieldResolver::new(column_names_bytes, '\t');
let indices = resolver.resolve(&config.names_from)?;
let output_name = resolver.column_names()
    .and_then(|names| names.get(idx).cloned())
    .unwrap_or_else(|| format!("field{}", idx));
```

**进阶用法：结合 header 读取**

`FieldResolver` 可以与 header 读取结合，进一步简化命令代码：

```rust
// libs/cli.rs
pub fn resolve_fields_with_header(
    reader: &mut TsvReader,
    header_config: &HeaderConfig,
    field_specs: &[String],
) -> anyhow::Result<(Vec<Vec<usize>>, Option<Vec<u8>>)> {
    // 读取 header
    let column_names_bytes = if header_config.enabled {
        let header_info = reader.read_header_mode(header_config.mode)?;
        header_info.column_names_line
    } else {
        None
    };
    
    // 使用 FieldResolver 解析所有字段规范
    let resolver = FieldResolver::new(column_names_bytes.clone(), '\t');
    let all_indices: Vec<Vec<usize>> = field_specs
        .iter()
        .map(|spec| resolver.resolve(spec))
        .collect::<Result<_, _>>()?;
    
    Ok((all_indices, column_names_bytes))
}
```

### 实施计划

#### 阶段 1: 实现新 API（保留旧 API）

在 `libs/tsv/fields.rs` 中添加 `FieldResolver` 结构（见上方「改进方案」），不修改现有函数。

#### 阶段 2: 试点命令（选择 2-3 个命令试用）

选择结构简单的命令进行试点：

| 命令 | 选择理由 |
|-----|---------|
| `blank.rs` | 字段解析逻辑简单，只有 `--field` 一个参数 |
| `fill.rs` | 与 `blank.rs` 类似，便于对比验证 |
| `uniq.rs` | 已部分迁移，只需切换到新 API |

**试点步骤**:
1. 修改命令使用 `FieldResolver`
2. 运行该命令的所有测试
3. 验证功能正常

#### 阶段 3: 全面迁移

按以下顺序迁移所有使用字段解析的命令：

| 批次 | 命令 | 复杂度 |
|-----|------|-------|
| 1 | `select.rs`, `join.rs` | 中等（已部分优化） |
| 2 | `wider.rs`, `stats.rs` | 高（需要输出 header 名称） |
| 3 | `sample.rs`, `split.rs` | 高（特殊解析需求） |
| 4 | 其他命令 | 低（简单替换） |

#### 阶段 4: 清理旧 API

所有命令迁移完成后：

1. **标记旧函数为 deprecated**:
   ```rust
   #[deprecated(since = "0.4.0", note = "Use FieldResolver instead")]
   pub fn resolve_fields_from_header(...) { ... }
   ```

2. **一个版本后移除**:
   - 删除 `resolve_fields_from_header`
   - 删除 `parse_field_list_with_header`（如果不再内部使用）
   - 删除 `Header` 结构（如果 `FieldResolver` 完全替代）

3. **更新文档**:
   - 更新 API 文档
   - 更新开发者指南
   - 添加迁移指南

### 当前状态

- [x] 阶段 1: 实现 `FieldResolver` (已完成，包含单元测试)
- [x] 阶段 2: 试点命令 (`blank.rs`, `fill.rs`, `uniq.rs`) (已完成，所有测试通过)
- [x] 阶段 3: 全面迁移 (已完成)
  - [x] `select.rs` - 已迁移，56/56 测试通过
  - [x] `wider.rs` - 已迁移，52/52 测试通过
  - [x] `longer.rs` - 已迁移，28/28 测试通过
  - [x] `sample.rs` - 已迁移，108/108 测试通过
  - [x] `join.rs` - 已迁移，81/81 测试通过
  - [x] `stats.rs` - 已迁移，207/207 测试通过
- [x] 阶段 4: 清理旧 API (已完成)
  - [x] 标记旧函数为 deprecated: `parse_field_list_with_header`, `parse_field_list_with_header_preserve_order`, `resolve_fields_from_header`
  - [x] 迁移 `filter/builder.rs` 到 `FieldResolver`
  - [x] 迁移 `plot/data.rs` 到 `FieldResolver`
  - [ ] 未来版本移除旧API（仅内部使用保留）
