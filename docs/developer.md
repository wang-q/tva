# 开发者指南

本文档旨在为 `tva` 的开发者提供技术背景、架构设计思路以及未来演进路线。

## changelog

```bash
git log v0.2.4..HEAD > gitlog.txt
git diff v0.2.4 HEAD -- "*.rs" "*.md" > gitdiff.txt

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

## 深度技术分析

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

## 计划中的功能

### 核心功能增强 (Core Enhancements)

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

## tva 与 xan 命令对比分析

通过对比 tva 和 xan 的命令集，以下是 tva 尚未实现但可能有价值的功能：

### 高优先级（建议尽快实现）

| 命令 | xan 对应 | 功能描述 | 价值 |
| :--- | :--- | :--- | :--- |
| **frequency** | `freq` | 频次统计表 | 快速了解数据分布，配合 `hist` 可视化 |
| **rename** | `rename` | 重命名列 | 数据清洗基础功能 |

### 低优先级（特定场景）

| 命令 | xan 对应 | 功能描述 | 价值 |
| :--- | :--- | :--- | :--- |
| **progress** | `progress` | 显示处理进度 | 大文件用户体验 |
| **search** | `search` | 多模式搜索（Aho-Corasick） | 高性能关键词搜索 |
| **separate** | `separate` | 使用正则拆分列 | 比 `unpack` 更灵活 |
| **window** | `window` | 滑动窗口计算 | 时间序列分析 |

### 暂不推荐实现

| 命令 | xan 对应 | 原因 |
| :--- | :--- | :--- |
| `eval` | `eval` | 需要表达式引擎，架构复杂 |
| `input` | `input` | 功能简单，可用其他方式替代 |
| `map` | `map` | 需要表达式引擎 |
| `parallel` | `parallel` | 架构复杂，可后期考虑 |
| `transform` | `transform` | 需要表达式引擎 |

## tva plot 子命令未来扩展

以下 plot 子命令已实现：
*   **散点图**: `tva plot point` - 支持散点、折线、路径、回归线、按颜色分组
*   **箱线图**: `tva plot box` - 支持分组、多列、异常值显示
*   **二维分箱热力图**: `tva plot bin2d` - 自动分箱，字符密度表示计数

**其他 plot 子命令计划**:
*   **直方图**: 增加 `tva plot hist` 命令（水平条形图）

### Hist (直方图) 补充分析

除了 `plot`，`xan` 还提供了一个专门的 `hist` 命令 (`xan/src/cmd/hist.rs`)，用于绘制水平条形图（Horizontal Bar Charts）。

*   **定位差异**:
    *   `plot`: 通用绘图工具，支持散点图、折线图、垂直条形图，基于 `ratatui`，交互性强，适合复杂数据探索。
    *   `hist`: 专注于**频次分布可视化**，通常配合 `freq` 或 `bins` 命令使用。它不使用 `ratatui`，而是直接通过 Unicode 字符（如 `█`, `▌`）在标准输出打印，更轻量，适合管道操作。
*   **核心逻辑**:
    *   **数据模型**: 期望输入包含 `field` (可选), `value` (标签), `count` (数值) 三列。
    *   **渲染**: 手动计算每个条形的宽度，使用 `Scale` 进行线性或对数缩放，并处理颜色（Rainbow/Category/Stripes）。
    *   **特色功能**: 支持日期补全 (`--dates`)，自动填充缺失的日期并设为 0；支持间隙压缩 (`--compress-gaps`)，隐藏连续的 0 值。

## 表达式引擎实施计划 (Expression Engine Roadmap)

基于对 `xan/moonblade` 和 `tera` 的深度分析，我们计划为 `tva` 构建一个轻量级、高性能的表达式引擎。

### 参考架构分析

#### 1. xan/moonblade 架构

**核心组件**:
-   **grammar.pest**: PEG 语法定义，使用 `pest` 解析器生成器
-   **parser.rs**: Pratt Parser 实现，处理操作符优先级
-   **interpreter.rs**: 树遍历解释器 (Tree-Walker Interpreter)
-   **functions.rs**: 内置函数库 (~200+ 函数)
-   **special_functions.rs**: 编译期和运行期特殊函数 (如 `col`, `header`)
-   **error.rs**: 统一的错误处理 (ConcretizationError, EvaluationError)
-   **types**: DynamicValue 动态类型系统

**关键设计模式**:
-   **Concretization (具体化)**: 执行前静态分析，将列名解析为索引
-   **Arc-based 字符串**: 减少深拷贝
-   **Pratt Parser**: 处理复杂操作符优先级
-   **GlobalVariables**: 支持全局变量槽位

#### 2. Tera 架构

**核心组件**:
-   **tera.pest**: 模板语法定义
-   **ast.rs**: AST 节点定义
-   **whitespace.rs**: 空白字符控制逻辑
-   **Renderer**: 树遍历渲染器

**关键设计模式**:
-   **Whitespace Control**: `{%-` 和 `-%}` 精确控制空白
-   **Filter Chain**: `value | filter1 | filter2`
-   **Macro System**: 模板级别的函数抽象

### TVA 表达式引擎设计

#### 设计原则

1.  **极简主义**: 只实现最核心的功能，避免过度设计
2.  **性能优先**: 零拷贝、预编译、静态分析
3.  **Shell 友好**: 语法避免与 Shell 特殊字符冲突
4.  **类型安全**: 显式类型转换，无隐式转换

#### 语法规范 (基于 syntax-design.md)

```rust
// 列引用
@1, @2, @col_name, @"col name"

// 字面量
42, 3.14, "hello", 'world', true, false, null

// 操作符 (按优先级)
()          // 分组
*, /, %     // 乘除模
+, -        // 加减
++          // 字符串拼接
==, !=, <, <=, >, >=  // 数值比较
eq, ne, lt, le, gt, ge // 字符串比较
and, or, not // 逻辑
|           // 管道 (最低优先级)

// 函数调用
trim(@name)
@name | trim(_)
@name | split(_, " ")

// 完整示例
@first_name | trim(_) | upper(_) ++ " " ++ @last_name | trim(_) | upper(_)
```

### 实施阶段

#### Phase 1: 核心基础设施 (2-3 周)

**目标**: 建立表达式解析和执行的基础框架

**任务清单**:

1.  **项目结构搭建** ✅
    ```
    src/libs/expr/
    ├── mod.rs              # 模块入口，公开 API
    ├── parser/
    │   ├── mod.rs          # 解析器入口（Pest 实现）
    │   ├── grammar.pest    # Pest PEG 语法定义
    │   └── ast.rs          # AST 节点定义
    ├── runtime/
    │   ├── mod.rs          # 运行时入口（求值器）
    │   └── value.rs        # Value 类型系统
    └── tests/              # 集成测试目录
        ├── mod.rs          # 测试模块入口
        ├── basic.rs        # 基础功能测试
        └── errors.rs       # 错误处理测试
    ```

    **测试策略**:
    -   单元测试放在各模块的 `#[cfg(test)]` 中，与源码在一起
    -   集成测试放在 `tests/` 子目录下，按功能分类（如 `basic.rs`, `errors.rs`）
    -   解析器测试覆盖所有语法规则和边界情况
    -   求值测试包含类型转换、错误处理
    -   CLI 集成测试放在顶层 `tests/` 目录（如 `cli_*.rs`）
    -   模糊测试确保零 panic（参考 `tests/torture/`）

2.  **语法定义 (grammar.pest)** ✅
    -   字面量: int, float, string (单/双/反引号), bool, null
    -   列引用: `@` 前缀（支持索引 `@1` 和名称 `@name`）
    -   操作符: 算术 (`+`, `-`, `*`, `/`, `%`, `**`)、比较 (`==`, `!=`, `<`, `<=`, `>`, `>=`)、逻辑 (`&&`/`and`, `||`/`or`, `!`/`not`)
    -   一元运算符: `-` (负号), `!`/`not` (逻辑非)
    -   函数调用: 前缀调用 `func(arg1, arg2)`
    -   括号分组: `(expr)`

3.  **AST 定义 (ast.rs)** ✅
    ```rust
    pub enum Expr {
        ColumnRef(ColumnRef),           // @1, @name
        Int(i64),                       // 整数
        Float(f64),                     // 浮点数
        String(String),                 // 字符串
        Bool(bool),                     // true/false
        Null,                           // null
        Unary { op: UnaryOp, expr: Box<Expr> },           // -expr, !expr
        Binary { op: BinaryOp, left: Box<Expr>, right: Box<Expr> },  // expr + expr
        Call { name: String, args: Vec<Expr> },           // func(args)
    }
    ```

4.  **Pest Parser 实现** ✅
    -   使用 Pest PEG 语法生成解析器
    -   处理操作符优先级和结合性
    -   语法规则层级: primary → unary → power → multiplicative → additive → comparison → logical_and → logical_or

5.  **错误处理** ✅
    ```rust
    pub enum ParseError { ... }     // 解析错误（使用 thiserror）
    pub enum EvalError { ... }      // 运行时求值错误
    ```

**验收标准**:
-   ✅ 能正确解析简单表达式: `@1 + @2`, `-5`, `2 ** 10`
-   ✅ 支持完整操作符优先级: 一元运算符 → 乘方 → 乘除模 → 加减 → 比较 → 逻辑
-   ✅ 错误信息清晰（使用 `thiserror`）
-   ✅ 单元测试覆盖率: 60 个测试全部通过

#### Phase 2: 运行时和函数库 (2-3 周)

**目标**: 实现表达式执行和核心函数库

**任务清单**:

1.  **DynamicValue 类型系统 (value.rs)**
    ```rust
    pub enum DynamicValue {
        None,
        Bool(bool),
        Int(i64),
        Float(f64),
        String(Arc<str>),  // Arc 避免拷贝
        List(Vec<DynamicValue>),
    }
    ```

2.  **解释器实现 (interpreter.rs)**
    -   树遍历执行 AST
    -   上下文管理 (当前行、列索引)
    -   管道执行逻辑
    -   短路求值 (and/or)

3.  **函数注册系统 (functions/mod.rs)**
    ```rust
    pub type Function = fn(BoundArgs) -> Result<DynamicValue, RuntimeError>;
    
    pub struct FunctionRegistry {
        functions: HashMap<String, (Function, Arity)>,
    }
    ```

4.  **核心函数实现**
    -   字符串: `trim`, `upper`, `lower`, `substr`, `split`, `len`
    -   数值: `abs`, `round`, `min`, `max`, `int`, `float`
    -   逻辑: `default`, `if`
    -   正则: `regex_match`, `regex_replace`

5.  **编译期优化 (concretize.rs)**
    -   列名解析为索引 (参考 xan 的 `col` 函数)
    -   常量折叠
    -   函数内联 (简单函数)

**验收标准**:
-   能执行完整表达式: `@price * (1 + @tax_rate)`
-   函数库覆盖核心使用场景
-   性能基准: 简单表达式 < 1μs/行

#### Phase 3: 与 TVA 集成 (2 周)

**目标**: 将表达式引擎集成到现有命令中

**任务清单**:

1.  **filter 命令增强**
    ```bash
    # 当前: 基于预定义操作符
    tva filter -f price -gt 100
    
    # 新增: 表达式支持
    tva filter -E '@price > 100 and @stock > 0'
    ```

2.  **新增 mutate 命令**
    ```bash
    # 计算新列
    tva mutate -e 'total = @price * @qty'
    
    # 修改现有列
    tva mutate -e '@name = @name | trim(_) | upper(_)'
    ```

3.  **新增 eval 命令**
    ```bash
    # 表达式求值
    tva eval -e '@price * 1.1' --header price
    
    # 结合输入数据
    cat data.tsv | tva eval -e '@1 + @2'
    ```

4.  **性能优化**
    -   表达式预编译 (每文件一次)
    -   缓冲区复用
    -   SIMD 加速 (字符串操作)

**验收标准**:
-   filter 支持表达式过滤
-   mutate 命令可用
-   性能不劣于专用命令 (如 `tva filter -e` vs `tva filter -f`)

#### Phase 4: 高级功能 (可选, 2-3 周)

**目标**: 扩展表达式能力

**任务清单**:

1.  **索引和切片**
    ```rust
    @list[0]      // 索引访问
    @list[1:3]    // 切片
    @map["key"]   // Map 访问
    ```

2.  **变量绑定 (as)**
    ```rust
    @full_name | split(_, " ") as parts | parts[0] ++ parts[1]
    ```

3.  **高阶函数 (简化版)**
    ```rust
    map(@items, x -> x * 2)
    filter(@items, x -> x > 10)
    ```

4.  **正则字面量**
    ```rust
    @email | test(/.*@.*\.com/)
    ```

5.  **空值处理操作符**
    ```rust
    @nickname // @username   // 如果为空，使用默认值
    ```

### 技术选型

| 组件 | 选择 | 理由 |
| :--- | :--- | :--- |
| Parser Generator | `pest` | 成熟的 PEG 解析器，xan/tera 都在使用 |
| Parser Algorithm | Pratt Parser | 处理操作符优先级的标准方案 |
| String Storage | `Arc<str>` | 零拷贝共享，减少内存分配 |
| Error Handling | `thiserror` | 与现有代码库一致 |
| Regex | `regex` crate | Rust 生态标准，性能优秀 |

### 风险和对策

| 风险 | 影响 | 对策 |
| :--- | :--- | :--- |
| 解析性能不足 | 高 | 使用 Pratt Parser，预编译表达式 |
| 内存占用过高 | 中 | Arc 共享字符串，缓冲区复用 |
| 功能过度设计 | 中 | 严格遵循极简原则，分阶段实施 |
| 与现有命令冲突 | 低 | 保持向后兼容，新增 `-e` 选项 |

### 参考资源

-   **xan/moonblade**: `xan-0.56.0/src/moonblade/`
    -   `grammar.pest`: 完整的 PEG 语法定义
    -   `parser.rs`: Pratt Parser 实现
    -   `interpreter.rs`: 解释器和求值逻辑
    -   `functions.rs`: 丰富的函数库参考
    -   `error.rs`: 错误处理模式

-   **tera**: `tera-1.20.1/src/parser/`
    -   `tera.pest`: 模板语法定义
    -   `whitespace.rs`: 空白字符处理
    -   `ast.rs`: AST 节点设计
