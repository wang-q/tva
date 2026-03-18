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

**3.1 管道操作符 (`|`)**

* **功能**: 使用 `_` 引用左侧表达式的结果，避免重复书写。
  ```python
  split(date, '/') | fmt('{}-{}-{}', _[2], _[0], _[1])
  ```
* **价值**: 提高表达式可读性，支持链式操作。
* **建议**: 在 `tva expr` 中实现管道操作符，需要扩展 AST 和求值器。

**3.2 待添加的函数**

以下函数尚未实现，可参考 `xan` 添加：

| 函数类别 | 函数名 | 功能 | 优先级 |
|:--------|:------|:-----|:------|
| **字符串** | `pad`/`lpad`/`rpad` | 字符串填充 | 中 |
| **列表** | `flatten` | 扁平化嵌套列表 | 中 |
| | `zip` | 合并多个列表 | 中 |
| | `take`/`drop` | 取/舍前 N 个元素 | 中 |
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

### 4. 列表/数组操作扩展

**建议添加**：
- `flatten([[1,2], [3,4]])` → `[1,2,3,4]`
- `zip([1,2], ["a","b"])` → `[[1,"a"], [2,"b"]]`
- `group_by([...], x => x.type)` - 按条件分组
- `take([1,2,3,4], 2)` / `drop([1,2,3,4], 2)`

### 优先级建议

| 优先级 | 改进项 | 原因 |
|:------|:------|:-----|
| 🔴 高 | 管道操作符 (`\|`) | 显著提升表达式可读性 |
| 🟡 中 | 方法调用语法 (`.`) | 提升用户体验 |
| 🟡 中 | 更多列表函数 (`flatten`, `zip`, `take`, `drop`) | 函数式编程核心 |
| 🟢 低 | 字符串填充 (`pad`) | 格式化输出 |
| 🟢 低 | 路径处理函数 | 特定场景 |
| 🔵 未来 | 类型系统重构 (`Arc`, `DateTime`) | 需要大量测试 |

## 参考项目: stringr (R 语言字符串处理)

`stringr` 是 R 语言中最流行的字符串处理包，提供了系统化、一致性的字符串操作接口。通过分析其功能设计，可以为 `tva expr` 的字符串函数提供补充建议。

### stringr 功能分类

stringr 将字符串函数分为四大类：

1. **字符操作** (Character manipulation): 操作字符串中的单个字符
2. **空白字符工具** (Whitespace tools): 添加、删除、操作空白字符
3. **本地化敏感操作** (Locale sensitive): 不同区域设置下表现不同
4. **模式匹配** (Pattern matching): 使用正则等引擎进行匹配

### 功能对比与建议

#### 已经有对应功能的 stringr 函数

以下 stringr 函数在 tva 中已有对应实现：

| stringr 函数 | tva 对应 | 说明 |
|:-------------|:---------|:-----|
| `str_detect(string, pattern, negate = FALSE)` | `contains()` / `regex_match()` | 检测匹配，`negate` 参数取反 |
| `str_ends(string, pattern, negate = FALSE)` | `ends_with()` | 检测后缀，`negate` 参数取反 |
| `str_length(string)` | `char_len()` | 字符数（codepoints） |
| `str_remove(string, pattern)` | `replace(string, pattern, "")` | 移除第一个匹配 |
| `str_remove_all(string, pattern)` | `regex_replace()` | 移除所有匹配 |
| `str_replace(string, pattern, replacement)` | `replace()` / `regex_replace()` | 替换第一个匹配，支持捕获组引用 |
| `str_replace_all(string, pattern, replacement)` | `regex_replace()` | 替换所有匹配，支持命名向量多模式替换 |
| `str_replace_na(string, replacement = "NA")` | `default()` | null 替换为默认值，默认替换为 "NA" |
| `str_split(string, pattern, n = Inf, simplify = FALSE)` | `split()` | 字符串分割，`n` 限制分割次数 |
| `str_starts(string, pattern, negate = FALSE)` | `starts_with()` | 检测前缀，`negate` 参数取反 |
| `str_sub(string, start = 1L, end = -1L)` | `substr()` | 提取子串，支持负索引从末尾计数 |
| `str_subset(string, pattern, negate = FALSE)` | `filter()` | 返回匹配元素，`negate` 返回不匹配 |
| `str_to_lower(string, locale = "en")` | `lower()` | 转小写 |
| `str_to_upper(string, locale = "en")` | `upper()` | 转大写 |
| `str_trim(string, side = c("both", "left", "right"))` | `trim()` | 去除空白，`side` 可选 "both"/"left"/"right" |
| `str_trunc(string, width, side = c("right", "left", "center"), ellipsis = "...")` | `truncate()` | 截断字符串，支持左/右/居中 |
| `word(string, start = 1L, end = start, sep = fixed(" "))` | `split()` + `nth()` | 提取第 n 个单词，支持自定义分隔符 |

#### 待实现的 stringr 函数

以下 stringr 函数在 tva 中尚无对应实现，按优先级分类：

**高优先级 (常用且难以用现有函数组合实现)**:

| stringr 函数 | 功能 | 说明 |
|:-------------|:-----|:-----|
| `str_pad(string, width, side = c("left", "right", "both"), pad = " ", use_width = TRUE)` | 字符串填充 | 对齐输出、固定宽度格式，数据展示刚需 |
| `str_squish(string)` | 压缩空白 | 去首尾 + 内部多空格变单空格，数据清洗常用 |
| `str_count(string, pattern = "")` | 统计匹配次数 | 返回整数向量，统计每行匹配数，难以用现有函数组合 |

**中优先级 (可用现有函数组合但较繁琐，或有部分替代方案)**:

| stringr 函数 | 功能 | 替代方案 |
|:-------------|:-----|:---------|
| `str_extract(string, pattern, group = NULL)` | 正则提取匹配 | `regex_extract()` 可部分替代，但 `str_extract` 支持 `group` 参数更灵活 |
| `str_extract_all(string, pattern, simplify = FALSE)` | 提取所有匹配 | 无直接替代，需循环实现 |
| `str_glue(..., .sep = "", .envir = parent.frame(), .trim = TRUE)` | 字符串插值 (`{var}` 语法) | 无直接替代，模板字符串功能 |
| `str_glue_data(.x, ..., .sep = "", .envir = parent.frame(), .na = "NA")` | 数据框字符串插值 | 无直接替代 |
| `str_to_title(string, locale = "en")` | 标题格式 | 每个单词首字母大写，locale 敏感 |
| `str_to_sentence(string, locale = "en")` | 句子格式 | 句首字母大写 |
| `str_which(string, pattern, negate = FALSE)` | 返回匹配索引 | 可用 `filter` + 行号实现，但不够直接 |
| `str_dup(string, times, sep = NULL)` | 重复字符串 | `join()` + `range()` 可组合实现 |
| `str_like(string, pattern, ignore_case = FALSE)` | SQL LIKE 匹配 | 可用正则替代，但 `LIKE` 语法更简单 |
| `str_escape(string)` | 转义正则元字符 | 无直接替代，但使用场景较窄 |

**低优先级 (特定场景，或可用现有方案较好替代)**:

| stringr 函数 | 功能 | 替代方案 |
|:-------------|:-----|:---------|
| `str_c(..., sep = "", collapse = NULL)` | 连接多个字符串 | `join()` 已能满足大部分需求 |
| `str_conv(string, encoding)` | 编码转换 | TSV 通常为 UTF-8，使用场景有限 |
| `str_equal(x, y, locale = "en", ignore_case = FALSE, ...)` | Unicode 等价比较 | 使用 Unicode 规范化规则比较字符串，场景较窄 |
| `str_flatten(string, collapse = "", last = NULL, na.rm = FALSE)` / `str_flatten_comma(string, last = NULL, na.rm = FALSE)` | 列表扁平化为字符串 | `join()` 已能较好替代 |
| `str_interp(string, env = parent.frame())` | 字符串插值 (旧版) | 被 `str_glue` 取代，无需单独实现 |
| `str_locate(string, pattern)` / `str_locate_all(string, pattern)` | 返回匹配位置 | 返回 start/end 矩阵，可与 `str_sub` 配合使用，但场景较窄 |
| `str_match(string, pattern)` / `str_match_all(string, pattern)` | 提取捕获组 | `regex_extract()` 可部分替代，返回矩阵场景较窄 |
| `str_order(x, decreasing = FALSE, na_last = TRUE, locale = "en", numeric = FALSE, ...)` / `str_sort(x, decreasing = FALSE, na_last = TRUE, locale = "en", numeric = FALSE, ...)` | 字符串排序/排序索引 | `sort()` 命令已能满足大部分排序需求 |
| `str_rank(x, locale = "en", numeric = FALSE, ...)` | 字符串排名 | 使用场景较窄 |
| `str_split_fixed(string, pattern, n)` | 分割为固定列数 | `split()` + `slice()` 可组合实现 |
| `str_split_i(string, pattern, i)` | 分割后取第 i 个 | `split()` + `nth()` 可直接替代 |
| `str_to_camel_case(string, first_upper = FALSE)` / `str_to_snake_case(string)` / `str_to_kebab_case(string)` | 命名格式转换 | 编程标识符转换，特定场景 |
| `str_unique(string, locale = "en", ignore_case = FALSE, ...)` | 去重 | `unique()` 命令已能满足需求 |
| `str_view(string, pattern = NULL, match = TRUE, html = FALSE, use_escapes = FALSE)` | 可视化匹配 | 调试工具，命令行场景较少使用 |
| `str_width(string)` | 显示宽度 | 等宽字体显示，场景较窄 |
| `str_word(string, start = 1L, end = start, sep = fixed(" "))` | 提取第 n 个单词 | `split()` + `nth()` 可直接替代 |
| `str_wrap(string, width = 80, indent = 0, exdent = 0, whitespace_only = TRUE)` | 文本自动换行 | 长文本格式化，场景较窄 |
| `fixed(pattern, ignore_case = FALSE)` / `coll(pattern, ignore_case = FALSE, locale = "en", ...)` / `regex(pattern, ignore_case = FALSE, multiline = FALSE, comments = FALSE, dotall = FALSE, ...)` / `boundary(type = c("character", "line_break", "sentence", "word"), ...)` | 模式匹配修饰符 | 当前正则支持已足够，修饰符复杂度较高 |
| `invert_match(loc)` | 反转匹配位置 | 与 `str_locate` 配合使用，场景较窄 |

#### 关于模式匹配引擎

stringr 支持四种模式匹配引擎，tva 目前主要支持简单的子串匹配：

| 引擎 | stringr | tva expr | 说明 |
|:-----|:--------|:---------|:-----|
| 正则表达式 | `regex()` | `regex_*` 函数 | 已部分支持 |
| 固定字符串 | `fixed()` | 默认行为 | 直接匹配 |
| 本地化匹配 | `coll()` | 无 | 考虑区域设置的比较 |
| 边界匹配 | `boundary()` | 无 | 字符/单词/句子/行边界 |

**建议**: 对于命令行工具场景，当前的正则支持已足够。`coll()` 和 `boundary()` 的复杂度较高，优先级较低。

#### 关于本地化 (Locale)

stringr 的以下函数支持 locale 参数：
- `str_to_upper/lower/title/sentence()` - 大小写转换
- `str_sort/order()` - 排序顺序

**建议**: tva 目前定位为数据处理工具而非全球化工具，locale 支持优先级较低。如需实现，可考虑通过环境变量而非函数参数控制。

## fmt 格式化函数规划

参考 Rust `format!` 宏和 Perl `q//` 的设计，使用 `%` 作为前缀，并支持多种成对分隔符（`()` `[]` `{}`），以避免与 bash 变量 `$`、字符串转义 `\` 和 GNU parallel 的 `{}` 冲突。

内部基于 Rust 的 `std::fmt` 格式化系统实现，支持其大部分格式功能。

### 函数签名

```rust
// 主格式化函数
fmt(template: string, ...args: any) -> string
```

### 格式说明符语法

支持三种成对分隔符，类似 Perl 的 `q//`：

```
%(...[:format_spec])     # 圆括号
%[...[:format_spec]]     # 方括号
%{...[:format_spec]}     # 花括号
```

分隔符内部可以包含同类型的单边符号，只要成对即可。

### 参数形式

| 形式 | 说明 | 示例 |
|:-----|:-----|:-----|
| `%()` | 按顺序使用下一个位置参数 | `fmt("%() %()", a, b)` |
| `%(n)` | 第 n 个位置参数（1-based） | `fmt("%(2) %(1)", a, b)` |
| `%(var)` | 引用 lambda 参数 | `fmt("%(name)")` |
| `%(@n)` | 第 n 列 | `fmt("%(@1) and %(@2)")` |
| `%(@var)` | 引用 `@var` | `fmt("%(@name)")` |

**分隔符选择：**
- 默认使用 `%()`，简洁直观
- 如果格式字符串包含 `()`，改用 `%[]` 或 `%{}`
- 内部可以嵌套同类型括号，只要成对：
  - `%(func(a, b))` → 引用名为 `func(a, b)` 的参数
  - `%[arr[0]]` → 引用名为 `arr[0]` 的参数
  - `%{obj{key}}` → 引用名为 `obj{key}` 的参数

**解析优先级：**
1. `%(@...)` → 列索引或 `@var`
2. `%(n)` → 位置参数（纯数字）
3. `%(var)` → lambda 参数
4. `%()` → 顺序位置参数

### 填充与对齐 (fill & align)

| 对齐 | 说明 | 示例 `%(:*<10)` |
|:-----|:-----|:----------------|
| `<` | 左对齐 | `hello*****` |
| `>` | 右对齐 | `*****hello` |
| `^` | 居中 | `**hello***` |

填充字符在对齐符号前指定，默认为空格。

### 符号 (sign)

| 符号 | 说明 | 示例 |
|:-----|:-----|:-----|
| `-` | 仅负数显示符号 (默认) | `-42` |
| `+` | 总是显示符号 | `+42`, `-42` |

### 替代形式 ('#')

在进制前缀显示基数标识：

| 类型 | 效果 | 示例 `%(:#x)` |
|:-----|:-----|:-------------|
| `x` | 添加 `0x` 前缀 | `0xff` |
| `X` | 添加 `0X` 前缀 | `0XFF` |
| `b` | 添加 `0b` 前缀 | `0b1010` |
| `o` | 添加 `0o` 前缀 | `0o77` |

### 宽度 (width)

最小字段宽度，不足则填充。

### 精度 (precision)

- 整数: 补零到指定宽度
- 浮点数: 小数位数
- 字符串: 最大字符数

### 类型说明符 (type)

| 类型 | 说明 | 示例 |
|:-----|:-----|:-----|
| 省略 | 默认显示 | 根据类型自动选择 |
| `b` | 二进制 | `1010` |
| `o` | 八进制 | `77` |
| `x` / `X` | 十六进制 | `ff` / `FF` |
| `e` / `E` | 科学计数法 | `1.23e+04` |
| `?` | 调试格式 | 带引号字符串 `"hello"` |

### 使用示例

```bash
# 基本格式化
tva expr -E 'fmt("Hello, %()!", "world")'           # "Hello, world!"
tva expr -E 'fmt("%() + %() = %()", 1, 2, 3)'        # "1 + 2 = 3"

# 位置参数 (1-based，与 parallel 一致)
tva expr -E 'fmt("%(2) %(1)", "world", "Hello")'    # "Hello world"

# 在 lambda 中使用参数引用
tva expr -E '
    map(["Alice", "Bob"], name => fmt("Hello, %(name)!"))
'
# ["Hello, Alice!", "Hello, Bob!"]

# 显式变量引用 %(@var)
tva expr -E '
    "Bob" as @name;
    fmt("Hello, %(@name)!")
'
# "Hello, Bob!"

# 变量与格式组合使用
tva expr -E '
    3.14159 as @pi;
    fmt("Pi = %(@pi:.2)")
'
# "Pi = 3.14"

tva expr -E '
    42 as @num;
    fmt("Hex: %(@num:#x), Bin: %(@num:b)")
'
# "Hex: 0x2a, Bin: 101010"

# 混合使用
tva expr -E '
    map([1, 2, 3, 4, 5], x => fmt("%(x) is %(:+)", x))
'
# 错误：lambda 参数 x 不在 fmt 的作用域内
# 正确用法：
tva expr -E '
    map([1, 2, 3], x => {
        x as @val;
        fmt("Value: %(@val)")
    })
'
# ["Value: 1", "Value: 2", "Value: 3"]

# 对齐与填充
tva expr -E 'fmt("%(:>10)", "hi")'                  # "        hi"
tva expr -E 'fmt("%(:*<10)", "hi")'                 # "hi********"
tva expr -E 'fmt("%(:^10)", "hi")'                  # "    hi    "

# 数字格式
tva expr -E 'fmt("%(:+)", 42)'                      # "+42"
tva expr -E 'fmt("%(:08)", 42)'                     # "00000042"
tva expr -E 'fmt("%(:.2)", 3.14159)'                # "3.14"
tva expr -E 'fmt("%(:>10.2)", 3.14159)'             # "      3.14"

# 进制转换
tva expr -E 'fmt("%(:b)", 42)'                      # "101010"
tva expr -E 'fmt("%(:x)", 255)'                     # "ff"
tva expr -E 'fmt("%(:#x)", 255)'                    # "0xff"
tva expr -E 'fmt("%(:08x)", 255)'                   # "000000ff"

# 字符串截断
tva expr -E 'fmt("%(:.5)", "hello world")'          # "hello"

# 调试格式
tva expr -E 'fmt("%(:?)", "hello")'                 # "\"hello\""

# 使用不同分隔符避免冲突
# 当格式字符串包含 () 时，改用 %[] 或 %{}
tva expr -E 'fmt("%[func(a, b)] = %[:.2]", 3.14159)'  # "func(a, b) = 3.14"
tva expr -E 'fmt("%{obj{key}}: %(:+)", 42)'           # "obj{key}: +42"

# 在 q() 中使用 %[] 避免转义
q(fmt(%[name] is %[age] years old))
```

### 在 expr 中使用列引用

```bash
# 使用 %(@n) 引用第 n 列（1-based）
echo -e "name\tage\nAlice\t25\nBob\t30" | tva expr -E 'fmt("%(@1) is %(@2) years old")'
# Alice is 25 years old
# Bob is 30 years old

# 使用 %(@name) 引用列名
echo -e "name\tage\nAlice\t25" | tva expr -E 'fmt("%(@name) is %(@age) years old")'
# Alice is 25 years old

# 混合列引用和位置参数
echo -e "product\tprice\nApple\t1.5" | tva expr -E 'fmt("%(@1): $%(:.2)", price)'
```

### 与 Rust format! 的差异

| 特性 | Rust | tva fmt | 说明 |
|:-----|:-----|:--------|:-----|
| 占位符 | `{}` | `%()` / `%[]` / `%{}` | 语法差异，内部转换 |
| 位置索引 | 0-based | 1-based | 与 GNU parallel 一致 |
| 命名参数 | `format!("{name}", name="val")` | 不支持 | 使用 `%(var)` 引用 lambda 参数替代 |
| 动态宽度 | `format!("{:>1$}", x, width)` | 不支持 | Rust 的 `$` 语法 |
| 动态精度 | `format!("{:.1$}", x, prec)` | 不支持 | Rust 的 `$` 语法 |
| 参数计数 | 编译期检查 | 运行时检查 | 模板与参数不匹配时返回错误 |

### 与 GNU parallel 的兼容性

GNU parallel 使用 `{}` 作为占位符（如 `{}` `{.}` `{/}` `{1}` 等）。tva 的 `fmt` 与之对比：

| 工具 | 占位符 | 示例 |
|:-----|:-------|:-----|
| GNU parallel | `{}` | `parallel echo {} ::: *.txt` |
| tva fmt | `%()` / `%[]` / `%{}` | `fmt("Hello, %()!", "world")` |

```bash
# 安全：parallel 的 {} 和 tva 的 %() 不会冲突
parallel 'tva expr -E "fmt(q(Processing: %[] at %[]), {}, now())"' ::: *.tsv

# 处理文件列表，生成格式化输出
parallel 'tva expr -E '"'"'fmt("File: %(1) (%.2 KB)", {}, size({}))'"'"'' ::: *.txt

# 结合 tva 其他命令
seq 1 10 | parallel 'tva expr -E '"'"'fmt("Processing item %(:03) of %()", {}, 10)'"'"'

# 多列数据处理
cat data.tsv | parallel --colsep '\t' 'tva expr -E '"'"'fmt("%(1): %(2) -> %(3)", {1}, {2}, {3})'"'"'
```

### 实现建议

基于 Rust `std::fmt` 实现，但替换占位符语法：

```rust
pub fn fmt(args: &[Value]) -> Result<String> {
    let template = args[0].as_str()?;
    // 将 % 语法转换为 Rust format 语法后调用标准库
    // 1. 解析模板中的 %(n) 和 % 占位符
    // 2. 映射 1-based 索引到 0-based
    // 3. 使用 write! 宏生成结果
}
```

### 实现优先级

1. **Phase 1**: 基本 `%()` 占位符、1-based 位置参数、默认格式化
2. **Phase 2**: 对齐 `<`/`>`/`^`、填充、宽度、精度
3. **Phase 3**: 符号 `+`/`-`、替代形式 `#`、进制 `b`/`o`/`x`/`X`
4. **Phase 4**: 科学计数法 `e`/`E`、调试格式 `?` 
