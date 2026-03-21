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
    * **文件路径处理**: `dirname`, `basename`, `extname`, `barename`。abspath
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

* **实现**: `cmd/parallel.rs` (~1600 行)
* **核心设计**: 采用 **"External Parallelism"** 模式，不修改单个命令的内部实现，而是通过一个通用的 `parallel` 子命令来并行化任意操作。

**关键机制**:

1. **任务分发策略** (线程分配算法):
    * 当输入文件数 >= 线程数时：每个文件一个任务
    * 当线程数 > 文件数时：利用 CSV/BGZF 的 seek 能力将大文件切分为多个 `FileChunk`
    * 通过 `simd_seeker().segments(t)` 实现基于字节偏移的精确分块

2. **预处理管道** (两种模式):
    * `-P, --preprocess`: 使用 xan 子命令管道 (如 `"search -s name John | slice -l 10"`)
    * `-H, --shell-preprocess`: 使用 shell 管道 (`$SHELL -c`)，更灵活但 Windows 不支持

3. **子命令实现**:
    * `count`: 行数统计 (支持 `--source-column` 输出每个文件的计数)
    * `cat`: 并行预处理并合并结果 (带缓冲区控制 `--buffer-size`)
    * `freq`: 并行频率统计，使用 `Counter` 数据结构合并结果
    * `stats`: 并行统计计算，使用 `Stats` 结构合并
    * `agg`/`groupby`: 并行聚合，使用 `AggregationProgram`/`GroupAggregationProgram`
    * `map`: 并行处理并输出到指定模板文件 (如 `'{}_freq.csv'`)

4. **进程管理** (`ProcessManager`):
    * 使用 `rayon` 线程池进行并行执行
    * 支持进度条 (`indicatif` 的 `MultiProgress`)
    * 子进程错误捕获和优雅退出 (通过 `Children` 结构管理)

5. **数据合并模式**:
    * **计数/求和型**: 使用 `AtomicU64` 或 `Mutex<BTreeMap>` 合并
    * **频率表型**: `FrequencyTables::merge()` 合并多个 `Counter`
    * **统计型**: `StatsTables::merge()` 合并 `Stats` 结构
    * **分组聚合型**: `GroupAggregationProgram::merge()` 合并分组结果

**对 `tva` 的启示**:

* **优势**: 
    * 无需修改现有命令代码即可并行化
    * 利用 SIMD CSV 解析器的 seek 能力实现文件分块
    * 支持 BGZF 索引文件的高效随机访问
* **挑战**:
    * 需要处理 header 在分块后的正确传递
    * 输出顺序控制 (`cat` 命令的 `--buffer-size`)
    * 错误处理和子进程管理复杂度较高
* **建议**: 
    * 第一阶段：实现基于文件粒度的并行 (类似 `xan parallel count *.tsv`)
    * 第二阶段：结合 `tva split` 实现大文件分块并行
    * 利用 TSV 无引号特性，可以比 CSV 更简单地实现字节级分块

#### par parallel/partition

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

|       场景       |  当前实现   | Arc 优化  |    加速比     |   结论   |
|:--------------:|:-------:|:-------:|:----------:|:------:|
|   String 克隆    | 407 µs  |  35 µs  | **11.6x**  | ✅ 显著提升 |
|    List 克隆     | 399 µs  |  36 µs  | **11.1x**  | ✅ 显著提升 |
|  `take()` 函数   | 48.3 ms | 1.38 ms |  **35x**   | ✅ 显著提升 |
| `reverse()` 函数 | 47.2 ms | 1.35 ms |  **35x**   | ✅ 显著提升 |
|  `slice()` 函数  | 2.67 ms | 2.67 ms |   **1x**   | ⚠️ 持平  |
|  `sort()` 函数   | 1.33 ms | 64.9 ms | **0.02x**  | ❌ 显著下降 |
| `unique()` 函数  | 3.97 ms | 426 ms  | **0.009x** | ❌ 显著下降 |
| `filter()` 函数  | 1.08 ms | 30.9 ms | **0.035x** | ❌ 显著下降 |
|   `map()` 函数   | 1.49 ms | 77.7 ms | **0.019x** | ❌ 显著下降 |

**分析**:

* **Arc 有优势的场景**: 纯克隆操作（不修改数据）和频繁参数传递的场景。
  `Arc` 仅递增引用计数（O(1)），而直接克隆需要深拷贝（O(n)）。

* **Arc 无优势的场景**: 需要遍历并创建新列表的操作（`sort`, `filter`, `map`, `unique`）。
  这些操作需要 `list.iter().cloned().collect()`，比直接 `list.clone()` 慢得多。
  此外，`Arc<Vec<T>>` 无法直接获取可变引用，需要 `Arc::make_mut` 或重新分配 Vec。

**字符串操作基准测试** (`benches/string_arc.rs`):

|      函数      |  当前实现   | Arc 优化  |    加速比    |   结论    |
|:------------:|:-------:|:-------:|:---------:|:-------:|
|  String 克隆   | 407 µs  |  35 µs  | **11.6x** | ✅ 显著提升  |
|  `split()`   | 47.2 ms | 1.35 ms |  **35x**  | ✅ 显著提升  |
| `replace()`  | 114 µs  | 96.6 µs | **1.2x**  | ⚠️ 轻微提升 |
|  `concat()`  | 4.01 ms | 2.58 ms | **1.6x**  | ✅ 一定提升  |
|  `upper()`   | 26.2 µs | 22.5 µs | **1.2x**  | ⚠️ 轻微提升 |
| `take_str()` | 1.66 ms | 1.61 ms | **1.03x** | ⚠️ 基本持平 |
|  `substr()`  | 1.73 ms | 1.94 ms | **0.89x** | ⚠️ 轻微下降 |

* **字符串操作分析**:
    * `split()` 受益于 Arc，因为需要频繁克隆参数
    * `replace()`, `upper()` 主要开销在字符串操作本身，Arc 优势不明显
    * `take_str()`, `substr()` 主要开销在新字符串分配，Arc 优势被抵消

**对 `tva` 的启示**:

* 如果 `expr` 命令主要使用 `take`, `first`, `last`, `nth` 等访问操作，`Arc` 优化是有价值的。
* 如果频繁使用 `sort`, `filter`, `map` 等转换操作，当前实现可能更合适。
* 字符串操作：`split()` 受益明显，其他操作收益有限。
* **最终建议**: 考虑到 `tva` 的核心是 TSV 文本处理，当前 `Value` 类型设计已足够高效，暂不引入 `Arc`
  增加复杂度。

## 代码结构优化建议

基于对25个命令文件的代码分析，以下是改进建议：

### 1. TsvRow 迭代器支持

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

### 3. 字段解析代码重复

**现状**: 多个命令包含类似的字段名检测逻辑：

```rust
// sample.rs, select.rs, uniq.rs 等均有类似代码
fn contains_field_names(spec: &str) -> bool {
    for part in spec.split(',') {
        let trimmed = part.trim();
        // ... 重复的逻辑
    }
}
```

**建议**: 将 `contains_field_names` 函数移至 `src/libs/tsv/fields.rs` 的 `FieldResolver` 中，作为公共方法。

### 7. 输出刷新策略不一致

**现状**: 部分命令支持 `--line-buffered`，但实现方式略有不同：

**建议**: 统一 line-buffered 处理逻辑，考虑在 `src/libs/io.rs` 中提供包装 writer。

---

## 表达式语言设计建议

基于对 `src/libs/expr/` 表达式引擎的深入分析，以下是语言设计层面的改进建议：

### 1. 优先级与结合性修正

**问题**: 幂运算 `**` 目前是左结合的，但数学上应该是右结合：

```rust
// 当前: 2 ** 3 ** 2 被解析为 (2 ** 3) ** 2 = 64
// 期望: 2 ** 3 ** 2 应该解析为 2 ** (3 ** 2) = 512
```

**建议**: 为幂运算单独实现右结合构建函数。

### 2. 管道操作符扩展

**当前设计**:
```rust
expr | func()           // 等价于 func(expr)
expr | func(_, arg)     // 等价于 func(expr, arg)
```

**建议**: 支持更多占位符位置，实现多参数管道：
```rust
@sep | replace(_, "-", _)   // replace(@sep, "-", @sep)
```

### 3. 字符串插值

**当前**: 字符串拼接需要 `++` 操作符： `"Hello, " ++ @name ++ "!"`

**建议**: 添加字符串插值语法：
```rust
// JavaScript/Python f-string 风格
`Hello, ${@name}!`
// 或 Ruby 风格
"Hello, #{@name}!"
```

### 4. 空值处理（Null Safety）

**建议**: 引入空值传播操作符，避免显式空值检查：
```rust
// 类似 JavaScript 的 ?. 和 ??
@user?.name ?? "Anonymous"

// 或链式空值传播
@a?.b?.c  // 如果 @a 或 @a.b 为 null，整个表达式为 null
```

### 5. 模式匹配（高级特性）

**建议**: 为复杂数据处理添加轻量级模式匹配：
```rust
match @status {
    "active" => @score * 2,
    "pending" => @score,
    _ => 0
}
```

### 6. 类型转换明确化

**建议**: 提供显式转换函数，同时保留隐式转换作为默认行为：
```rust
@value::int      // 显式转整数
@value::float    // 显式转浮点
@value::string   // 显式转字符串
@value::parse()  // 自动解析（当前行为）
```

### 7. 错误处理改进

**当前**: `Bare` 标识符在解析时创建，但延迟到运行时报错。

**建议**: 在解析阶段就拒绝裸标识符，提供更清晰的错误信息：
```rust
// 当前：解析成功，运行时失败
price + 1   // Parse OK → Runtime Error

// 建议：解析时直接失败
price + 1   // Parse Error: Unknown identifier 'price', did you mean '@price'?
```

### 8. 函数调用统一化

**建议**: 统一函数调用和方法调用语法，支持字面量方法调用：
```rust
// 所有调用都是方法调用风格
upper(@name)            // 可以写成 @name | upper()
5.abs()                 // 字面量也支持方法调用
[1,2,3].len()           // 列表支持方法调用
```

### 9. 列表推导式

**建议**: 添加类似 Python 的列表推导语法：
```rust
[x * 2 for x in @nums if x > 0]
// 或 SQL 风格
@nums | filter(_, @it > 0) | map(_, @it * 2)
```

### 10. 性能优化建议

**建议**: 对于可交换/可结合的操作，考虑展平为 n-ary 节点，减少递归开销：
```rust
// 当前: 嵌套 Binary 节点
((a + b) + c) + d

// 优化: Nary 节点
Nary { op: Add, exprs: [a, b, c, d] }
```

### 优先级汇总

| 优先级 | 建议 | 影响 |
|--------|------|------|
| 高 | 修复 `**` 右结合性 | 正确性 |
| 高 | 解析期拒绝裸标识符 | 用户体验 |
| 中 | 字符串插值 | 易用性 |
| 中 | 空值传播 | 健壮性 |
| 中 | 统一函数/方法调用 | 一致性 |
| 低 | 模式匹配 | 表达能力 |
| 低 | 列表推导 | 表达能力 |

---

## 表达式求值递归优化方案

基于对 `src/libs/expr/runtime/mod.rs` 的分析，结合 tva **只有匿名函数（lambda）**且主要用于 TSV 数据处理的特点，以下是实用的优化策略。

### 关键洞察：为什么递归不是主要问题

1. **无具名函数定义**：lambda 无法直接自引用，天然避免了无限递归
2. **TSV 场景限制**：典型表达式是简单的列运算（`@price * @qty`），而非深层嵌套
3. **高阶函数替代**：`map`/`filter`/`reduce` 覆盖了 95% 的循环需求

```rust
// 典型 tva 表达式 - 无需递归
@data | filter(_, @age > 18) | map(_, @name.upper())

// 聚合用 reduce - 无需递归
reduce(@nums, 0, (acc, x) => acc + x)
```

### 1. 真正需要优化的地方

#### 1.1 展平左结合表达式（高优先级）

**问题**：`1+2+3+...+N` 产生 O(N) 嵌套深度

```rust
// 当前 AST 结构（嵌套）
Binary {
    op: Add,
    left: Binary {
        op: Add,
        left: Binary { op: Add, left: 1, right: 2 },
        right: 3
    },
    right: 4
}
```

**优化**：在构建阶段展平

```rust
// ast.rs - 无需新增节点类型，优化 builder 即可
// builder/binary.rs
pub fn build_additive(pair: Pair<Rule>) -> Result<Expr, ParseError> {
    let inner: Vec<Pair<Rule>> = pair.into_inner().collect();
    let mut exprs: Vec<Expr> = inner
        .into_iter()
        .filter(|p| p.as_rule() != Rule::op_add && p.as_rule() != Rule::op_sub)
        .map(|p| super::build_expr(p))
        .collect::<Result<Vec<_>, _>>()?;
    
    if exprs.len() == 1 {
        return Ok(exprs.into_iter().next().unwrap());
    }
    
    // 直接构建左折叠的树，但控制深度
    // 或使用迭代求值的特殊处理
    Ok(fold_left_iterative(exprs, BinaryOp::Add))
}

// runtime/mod.rs - 特殊处理长链
Expr::Binary { op, left, right } => {
    // 检测是否是长链（可选优化）
    if should_flatten_eval(op, left) {
        return eval_binary_chain(op, left, right, ctx);
    }
    // 正常递归...
}

fn eval_binary_chain(
    op: BinaryOp,
    left: &Expr,
    right: &Expr,
    ctx: &mut EvalContext,
) -> Result<Value, EvalError> {
    // 收集链上所有表达式
    let mut exprs = vec![];
    collect_chain(left, op, &mut exprs);
    exprs.push(right.clone());
    
    // 迭代求值
    let mut result = eval(&exprs[0], ctx)?;
    for expr in &exprs[1..] {
        let val = eval(expr, ctx)?;
        result = apply_binary_op(op, result, val)?;
    }
    Ok(result)
}
```

#### 1.2 列表字面量的迭代构建

```rust
// 问题：[1,2,3,...,1000] 递归构建列表
Expr::List(elements) => {
    let values: Result<Vec<Value>, EvalError> =
        elements.iter().map(|e| eval(e, ctx)).collect();
    Ok(Value::List(values?))
}

// 优化：已经是迭代（map），但可以用 with_capacity 减少分配
Expr::List(elements) => {
    let mut values = Vec::with_capacity(elements.len());
    for e in elements {
        values.push(eval(e, ctx)?);
    }
    Ok(Value::List(values))
}
```

### 2. 安全防护（必做）

#### 2.1 递归深度限制

```rust
use std::cell::RefCell;

thread_local! {
    static RECURSION_DEPTH: RefCell<usize> = RefCell::new(0);
}

const MAX_RECURSION_DEPTH: usize = 1000;

pub fn eval(expr: &Expr, ctx: &mut EvalContext) -> Result<Value, EvalError> {
    RECURSION_DEPTH.with(|depth| {
        let mut d = depth.borrow_mut();
        *d += 1;
        if *d > MAX_RECURSION_DEPTH {
            *d -= 1;  // 清理
            return Err(EvalError::RecursionLimitExceeded(MAX_RECURSION_DEPTH));
        }
        drop(d);
        
        let result = eval_internal(expr, ctx);
        *depth.borrow_mut() -= 1;
        result
    })
}
```

#### 2.2 辅助函数迭代化

```rust
// contains_underscore 改为迭代
fn contains_underscore(expr: &Expr) -> bool {
    let mut stack = vec![expr];
    while let Some(e) = stack.pop() {
        match e {
            Expr::Underscore => return true,
            Expr::Call { args, .. } | Expr::MethodCall { args, .. } => {
                stack.extend(args.iter());
            }
            Expr::List(items) => stack.extend(items.iter()),
            Expr::Unary { expr, .. } => stack.push(expr),
            Expr::Binary { left, right, .. } => {
                stack.push(left);
                stack.push(right);
            }
            Expr::Pipe { left, right } => {
                stack.push(left);
                match right.as_ref() {
                    PipeRight::Call { args, .. } | PipeRight::CallWithPlaceholder { args, .. } => {
                        stack.extend(args.iter());
                    }
                }
            }
            _ => {}
        }
    }
    false
}
```

### 3. 常量折叠（编译期优化）

```rust
// builder/constant_fold.rs
pub fn fold_constants(expr: Expr) -> Expr {
    match expr {
        Expr::Binary { op, left, right } => {
            let left = fold_constants(*left);
            let right = fold_constants(*right);
            
            match (&left, &right) {
                (Expr::Int(a), Expr::Int(b)) => match op {
                    BinaryOp::Add => return Expr::Int(a + b),
                    BinaryOp::Sub => return Expr::Int(a - b),
                    BinaryOp::Mul => return Expr::Int(a * b),
                    BinaryOp::Div if *b != 0 => return Expr::Int(a / b),
                    _ => {}
                },
                (Expr::Float(a), Expr::Float(b)) => match op {
                    BinaryOp::Add => return Expr::Float(a + b),
                    BinaryOp::Sub => return Expr::Float(a - b),
                    BinaryOp::Mul => return Expr::Float(a * b),
                    BinaryOp::Div => return Expr::Float(a / b),
                    _ => {}
                },
                _ => {}
            }
            
            Expr::Binary { 
                op, 
                left: Box::new(left), 
                right: Box::new(right) 
            }
        }
        
        Expr::Unary { op, expr } => {
            let expr = fold_constants(*expr);
            match (&op, &expr) {
                (UnaryOp::Neg, Expr::Int(n)) => return Expr::Int(-n),
                (UnaryOp::Neg, Expr::Float(n)) => return Expr::Float(-n),
                (UnaryOp::Not, Expr::Bool(b)) => return Expr::Bool(!b),
                _ => {}
            }
            Expr::Unary { op, expr: Box::new(expr) }
        }
        
        Expr::List(elements) => {
            let folded: Vec<Expr> = elements.into_iter()
                .map(fold_constants)
                .collect();
            Expr::List(folded)
        }
        
        _ => expr
    }
}

// 在 parse() 后调用
pub fn parse(input: &str) -> Result<Expr, ParseError> {
    let expr = parse_raw(input)?;
    Ok(fold_constants(expr))
}
```

### 4. 不需要的优化

| 优化方案 | 不需要的原因 |
|----------|--------------|
| 显式栈求值器 | 复杂度太高，TSV 场景用不上 |
| 尾递归优化 | 无具名函数，lambda 无法自引用 |
| Y 组合子 | 可以用 `reduce` 替代递归 |
| Trampolining | 无用户定义的递归函数 |

### 5. 优化优先级

| 优先级 | 方案 | 实现 | 收益 |
|--------|------|------|------|
| 高 | 递归深度限制 | 10 行代码 | 安全防护 |
| 高 | 常量折叠 | 50 行代码 | 编译期加速 |
| 中 | 展平二元表达式 | 30 行代码 | 长链性能 |
| 中 | 辅助函数迭代化 | 20 行代码 | 消除栈风险 |
| 低 | 显式栈求值 | 200+ 行 | 不必要 |

### 6. 核心原则

> **保持简单**：tva 的表达式语言设计目标是**简单高效的数据处理**，不是通用编程语言。

```rust
// 好的 tva 表达式 - 声明式、无递归
@data 
    | filter(_, @age >= 18) 
    | map(_, @salary * 1.1) 
    | reduce(@, 0, (sum, x) => sum + x)

// 不需要支持 - 复杂的递归算法
// fib = n => if(n <= 1, n, fib(n-1) + fib(n-2))  // 不支持！
```

真正的递归需求（如树遍历）应该在 Rust 层实现为内置函数，而不是让用户用 lambda 写递归。
