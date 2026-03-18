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

#### 4. 类型系统设计 (Type System)

`xan` 的 `DynamicValue` 设计值得借鉴：

**5.1 廉价克隆 (Cheap Clone)**

* 使用 `Arc` 包装 `List`, `Map`, `String`, `Bytes` 等可变长数据。
* 保证 `DynamicValue` 不超过 16 字节，克隆成本极低。

**5.2 丰富的类型**

* 支持 `List`, `Map`, `Regex`, `DateTime`, `Bytes` 等高级类型。
* 支持序列化/反序列化（`Serialize`/`Deserialize` trait）。

**4.3 对 `tva` 的启示**

* 当前 `tva` 的 `Value` 类型相对简单，未来可考虑：
  * 使用 `Arc` 优化大对象（如长字符串、列表）的克隆性能。
  * 添加 `DateTime` 类型，支持更强大的时间处理。
  * 添加 `Bytes` 类型，支持二进制数据处理。

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
| 🔴 高 | 管道操作符 (`\|`) | 显著提升表达式可读性 |
| 🟡 中 | 方法调用语法 (`.`) | 提升用户体验 |
| 🟢 低 | 字符串填充 (`pad`) | 格式化输出 |
| 🟢 低 | 路径处理函数 | 特定场景 |
| 🔵 未来 | 类型系统重构 (`Arc`, `DateTime`) | 需要大量测试 |

## 扩展 List 相关函数

参考 Scala List、Haskell Data.List、F# List 与 Kotlin Collections API 设计，针对 TVA expr 的 TSV 行数据处理场景，筛选出**真正需要实现**的列表操作函数。

### 核心原则

TVA expr 用于处理 TSV 中的单行数据，典型场景：
- `split()` 字符串得到列表 → 处理 → `join()` 输出
- 构造列表进行数据转换
- 简单的列表查询和过滤

**实现标准**：
- ✅ **需要实现**：无法通过现有函数简单组合，或组合后性能/可读性极差
- ❌ **不需要实现**：可用现有函数一行组合实现，或场景不符

### 需要实现的函数（核心）

以下函数**无法**用现有函数简单高效地组合实现：

#### Generic Functions（同时支持 String 和 List）

| 优先级 | 函数 | 示例 | 说明 | 状态 |
|--------|------|------|------|------|
| **P1** | `is_empty` | `is_empty([])` → `true` / `is_empty("")` → `true` | 判断是否为空 | ✅ 已实现 |
| **P1** | `take` | `take([1,2,3,4], 2)` → `[1,2]` / `take("hello", 2)` → `"he"` | 取前 n 个元素 | ✅ 已实现 |
| **P1** | `drop` | `drop([1,2,3,4], 2)` → `[3,4]` / `drop("hello", 2)` → `"llo"` | 删除前 n 个元素 | ✅ 已实现 |
| **P1** | `contains` | `contains([1,2,3], 2)` → `true` / `contains("hello", "ll")` → `true` | 判断是否包含 | ✅ 已实现 |

#### List-only Functions（仅列表操作）

| 优先级 | 函数 | 示例 | 说明 | 状态 |
|--------|------|------|------|------|
| **P1** | `flatten` | `flatten([[1,2], [3,4]])` → `[1,2,3,4]` | 扁平化嵌套列表 | ✅ 已实现 |
| **P1** | `zip` | `zip([1,2], ["a","b"])` → `[[1,"a"], [2,"b"]]` | 多列表拉链组合 | ✅ 已实现 |
| **P2** | `partition` | `partition([1,2,3,4], x -> x % 2 == 0)` → `[[2,4], [1,3]]` | 按条件分区为两个列表 | ✅ 已实现 |
| **P2** | `flat_map` | `flat_map([1,2], x -> [x, x*2])` → `[1,2,2,4]` | 映射并扁平化 | ✅ 已实现 |
| **P2** | `grouped` | `grouped([1,2,3,4,5], 2)` → `[[1,2], [3,4], [5]]` | 按大小分块 | ✅ 已实现 |

### 可用现有函数组合实现（不需要单独实现）

以下函数**可以**用现有函数组合实现，提供文档示例即可：

#### Generic Functions（同时支持 String 和 List）

| 函数 | 组合方案 |
|------|----------|
| `if_empty(value, default)` | 已有 `default(val, fallback)` 函数 |

#### List-only Functions（仅列表操作）

| 函数 | 组合方案 |
|------|----------|
| `take_last(list, n)` | `reverse(list) \|> take(n) \|> reverse()` |
| `drop_last(list, n)` | `reverse(list) \|> drop(n) \|> reverse()` |
| `find(list, pred)` | `filter(list, pred) \|> first()` |
| `find_back(list, pred)` | `filter(list, pred) \|> last()` |
| `find_index(list, pred)` | `zip(range(len(list)), list) \|> filter((i, x) -> pred(x)) \|> first() \|> first()` |
| `exists(list, pred)` | `len(filter(list, pred)) > 0` |
| `forall(list, pred)` | `len(filter(list, pred)) == len(list)` |
| `count(list, pred)` | `len(filter(list, pred))` |
| `index_of(list, val)` | `find_index(list, x -> x == val)` |
| `last_index_of(list, val)` | `find_index(reverse(list), x -> x == val)` |
| `sum(list)` | `reduce(list, 0, (a, b) -> a + b)` |
| `sum_of(list, f)` | `sum(map(list, f))` |
| `avg(list)` | `sum(list) / len(list)` |
| `avg_of(list, f)` | `avg(map(list, f))` |
| `min(list)` / `max(list)` | `reduce(list, first(list), (a, b) -> if(a < b, a, b))` |
| `min_of(list, f)` / `max_of(list, f)` | `min(map(list, f))` / `max(map(list, f))` |
| `min_by(list, f)` / `max_by(list, f)` | `sort_by(list, f) \|> first()` / `last()` |
| `filter_not(list, pred)` | `filter(list, x -> !pred(x))` |
| `distinct_by(list, f)` | 复杂组合，但使用频率低 |
| `indexed(list)` | `zip(range(len(list)), list)` |
| `pairwise(list)` | `sliding(list, 2)` |
| `union(a, b)` | `unique(concat(a, b))` |
| `intersect(a, b)` | `filter(a, x -> contains(b, x)) \|> unique()` |
| `difference(a, b)` | `filter(a, x -> !contains(b, x))` |
| `sliding(list, n)` | TSV 行数据处理极少需要滑动窗口 |

### 不需要实现的函数（场景不符）

以下函数在 TVA expr 的 TSV 行数据处理场景中**不适用**：

| 函数 | 原因 |
|------|------|
| `zip_with` | 可用 `zip` + `map` 组合 |
| `truncate` | TSV 行数据通常不需要安全 take |
| `drop_while` | 与 `take_while` 对称，但使用频率低 |
| `length` | 已有 `len()` 泛型函数 |
| `none` | 可用 `!exists(...)` 实现 |
| `span` | 可用 `take_while` + `drop_while` 组合 |
| `group_by` | 返回对象结构，与 TVA 表达式简化原则冲突 |
| `product` | 纯数学函数，TSV 数据处理极少用到 |
| `count_by` | 返回对象结构，TSV 行处理不需要 |
| `scan` | 保留中间结果的 reduce，数学/算法场景使用 |
| `choose` | 可用 `filter` + `map` 组合实现 |
| `split_into` | 可用 `grouped` 或数学计算实现 |
| `intersperse` / `intercalate` | 可用 `join` 配合分隔符实现 |
| `transpose` | 矩阵操作，TSV 行数据处理不需要 |
| `sort_on` | 已有 `sort_by`，功能重复 |
| `inits` / `tails` | 生成所有前缀/后缀，惰性序列场景 |
| `zip_with_next` | 可用 `sliding(2)` 替代 |
| `is_prefix` / `is_suffix` / `is_infix` | 子序列匹配，字符串处理场景 |
| `slice` | 已有 `slice` 函数 |
| `insert_at` / `remove_at` / `update_at` | 原地修改操作，不符合函数式风格 |
| `split_at` | 可用 `take` + `drop` 组合实现 |
| `init` | 可用 `range` + `map` 组合实现 |
| `replicate` | 可用 `range` + `map` 或字面量构造 |
| `singleton` | 可用 `[x]` 字面量直接构造 |
| `unfold` | 无限序列生成，TSV 行处理不需要 |
| `shuffled` / `random` / `random_sample` | 随机操作，需要引入随机状态 |
| `binary_search` | 有序列表查找，TSV 行处理极少需要 |

### 设计参考

- **Scala List**: 面向对象的函数式集合，方法调用风格 `list.map(f)`，惰性求值
- **Haskell Data.List**: 纯函数式，函数作为参数 `map f list`，惰性求值，数学严谨
- **F# List**: .NET 平台上的函数式列表，强调实用性和安全性（`tryXxx` 函数）
- **Kotlin Collections**: JVM 平台上的实用主义设计，丰富的扩展函数，运算符重载
- **TVA 风格**: 保持函数式风格，列表作为第一个参数 `map(list, f)`，与管道操作符配合
