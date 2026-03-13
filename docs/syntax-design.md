# 语法设计与表达式语言规划

本文档旨在规划 `tva` 的数据处理格式与表达式语法。我们将首先分析现有的优秀工具（如 GNU Parallel 和 xan/moonblade）的设计模式，然后基于 `tva` 的高性能、零拷贝目标，引出我们自己的格式设计。

## 1. 现有方案分析 (Landscape Analysis)

### 1.1 GNU Parallel: 模板替换 (Template Substitution)

GNU Parallel 采用的是基于**字符串替换**的占位符机制，特别适合构建 Shell 命令。

*   **核心机制**: 预定义占位符，在执行时进行字符串插值。
*   **占位符语法**:
    *   `{1}`, `{2}`: 按索引引用列。
    *   `{-1}`: 倒序引用。
    *   `{col_name}`: 引用表头列名。
    *   `{= perl_expr =}`: 嵌入 Perl 表达式进行即时求值。
*   **优点**:
    *   直观，符合 Shell 脚本习惯。
    *   适合"构造命令"的场景。
*   **缺点**:
    *   本质是文本处理，缺乏类型系统（Type System）。
    *   难以处理复杂的逻辑运算（除非嵌入 Perl，但会有性能损耗）。

### 1.2 AWK: 记录与字段处理模型 (Record and Field Processing Model)

AWK 是 Unix 文本处理的经典工具，其核心设计围绕**记录 (Records)** 和**字段 (Fields)** 的概念展开。

*   **核心机制**:
    *   **记录分割**: 输入被分割为记录（默认按行，由 `RS` 变量控制）。`NR` 记录当前记录号，`FNR` 记录当前文件的记录号。
    *   **字段分割**: 每个记录进一步分割为字段（默认按空白，由 `FS` 变量控制）。`NF` 记录当前记录的字段数。
    *   **字段引用**: 使用 `$n` 语法引用第 n 个字段，`$0` 表示整行记录。字段号可以是变量或表达式（如 `$NF` 表示最后一个字段）。

*   **语法特性**:
    *   **模式-动作**: `pattern { action }` 结构，模式匹配时执行动作。
    *   **内置变量**: `NR`, `NF`, `FS`, `RS`, `OFS`, `ORS` 等控制输入输出行为。
    *   **隐式类型转换**: 字符串和数值根据上下文自动转换。

*   **优点**:
    *   **简洁高效**: 针对文本处理优化，单行代码即可完成复杂任务。
    *   **流式处理**: 天然支持逐行处理，内存占用低。
    *   **广泛可用**: 几乎所有 Unix-like 系统都预装 AWK。

*   **缺点**:
    *   **语法古老**: 语法与现代语言差异较大，学习曲线陡峭。
    *   **类型系统薄弱**: 隐式转换虽方便但也容易出错。
    *   **功能有限**: 标准 AWK 缺乏现代功能（如 JSON 支持、丰富的字符串函数）。

### 1.3 xan (Moonblade): 动态表达式引擎 (Dynamic Expression Engine)

xan 内置了一个名为 **Moonblade** 的动态脚本语言，其设计类似于简化版的 Python/JavaScript，专为 CSV 处理优化。

*   **核心机制**:
    *   **Parser & Interpreter**: 基于 PEG 解析器 (Pest) 构建 AST，采用树遍历解释器 (Tree-Walker Interpreter)。
    *   **Concretization (具体化)**: 在执行前进行静态分析优化，例如将列名引用静态解析为索引，避免运行时的哈希查找开销。
    *   **Arc-based Types**: 使用 `Arc<T>` 管理字符串和复杂对象，尽可能减少深拷贝，适合流式处理。
-
*   **语法特性**:
    *   **列即变量**: 直接使用列名作为变量（如 `count > 10`），或通过 `col("name")`/`col(index)` 引用。
    *   **操作符区分**: 区分字符串 (`eq`, `++`) 和数值 (`==`, `+`) 操作符（类 Perl 设计），避免隐式转换带来的歧义。
    *   **丰富的内置函数**: 涵盖 String, Math, Date, Regex, Hashing 等，支持 CSS Selector 风格的 HTML 抓取。
    *   **高阶函数**: 支持 `map`, `filter`, `reduce` 及 Lambda 表达式（如 `map(list, x -> x + 1)`）。

*   **优点**:
    *   **表达力极强**: 能够处理复杂的业务逻辑（如 JSON 解析、日期计算、模糊匹配）。
    *   **相对高性能**: 相比通用脚本语言（Python/Lua），它针对 CSV 结构（Header/Row）进行了专门优化。
*   **缺点**:
    *   **实现复杂度高**: 维护一套完整的解释器、类型系统和函数库成本较高。
    *   **运行时开销**: 尽管有优化，但在处理每一行时仍需进行 AST 遍历和动态分发，相比纯 Native 代码有一定损耗。

### 1.3 jq: JSON 处理器与流式计算 (JSON Processor & Stream Computing)

`jq` 是处理 JSON 数据的事实标准工具，其核心是一个专门设计的栈式虚拟机（Stack-based VM），支持回溯（Backtracking）和生成器（Generators）。

*   **核心机制**:
    *   **Bytecode Compiler & VM**: `jq` 将查询（Filters）编译为字节码，并在 `execute.c` 中的解释器循环中执行。这与 `xan` 的 Tree-walker 不同，提供了更紧凑的执行模型。
    *   **Backtracking (回溯)**: `jq` 的杀手级特性。通过 `FORK` 和 `BACKTRACK` 指令，一个输入可以产生零个、一个或多个输出（例如 `.[]`）。这使得 `jq` 本质上是一个流式生成器引擎。
    *   **Reference Counting (引用计数)**: 数据通过 `jv` 结构体表示（定义在 `jv.h`），使用引用计数管理内存。这允许高效的"浅拷贝"（Copy-on-write），但在高频分配下会有一定开销。

*   **架构设计**:
    *   **`jv` 类型系统**: 统一的 JSON 值表示（Tagged Union），包含 Null, Bool, Number, String, Array, Object。
    *   **Hybrid Stdlib**: 大量标准库函数（如 `map`, `select`, `recurse`）直接用 `jq` 语言编写（`builtin.jq`），只有底层原语（如 `type`, `keys`）用 C 实现。这使得扩展库非常容易。
    *   **Stack Machine**: 虚拟机维护值栈（Value Stack）和调用栈（Frame Stack），支持闭包（Closures）和复杂的控制流。

*   **语法特性**:
    *   **Filters**: 一切皆过滤器。输入 -> 过滤器 -> 输出流。
    *   **管道 (`|`)**: 组合过滤器的主要方式。`a | b` 将 `a` 的**每一个**输出作为 `b` 的输入运行。
    *   **Context (`.`)**: 当前上下文值的显式引用。
    *   **变量 (`$var`)**: 通过 `... as $x | ...` 绑定变量，这在 `jq` 中是词法作用域的。

*   **优点**:
    *   **极简的流式语法**: 处理嵌套 JSON 数据极其高效。
    *   **强大的组合能力**: 通过管道和回溯，可以轻松实现笛卡尔积、全排列等复杂逻辑。
    *   **成熟生态**: 几乎所有环境都预装，无依赖。

*   **缺点**:
    *   **性能瓶颈**: 相比于专门的列式引擎（如 xan/polars），`jq` 的逐个对象处理和解释器开销较大。
    *   **心智模型**: 回溯机制对于习惯传统编程的用户来说较难理解（例如 `debug` 为什么会打印多次）。

### 1.4 Tera: 模板引擎与 Rust 生态 (Template Engine & Rust Ecosystem)

`Tera` 是 Rust 生态中最流行的模板引擎之一，深受 Jinja2 和 Django 模板的启发。虽然它主要用于生成 HTML，但其设计模式对表达式语言的设计有很高的参考价值。

*   **核心机制**:
    *   **Pest Parser**: 使用 PEG (Parsing Expression Grammar) 解析器 (`pest`) 定义语法 (`tera.pest`)，生成 AST (`ast.rs`)。这提供了极高的语法灵活性。
    *   **Renderer & CallStack**: 采用 Tree-walker 模式。通过 `CallStack` (`call_stack.rs`) 管理 `StackFrame` (`stack_frame.rs`)，处理作用域、循环 (`ForLoop`) 和宏调用。
    *   **JSON-centric**: 数据模型完全构建在 `serde_json::Value` 之上。`Context` 本质上是 `BTreeMap<String, Value>`。这意味着它能无缝处理任何可序列化为 JSON 的 Rust 数据。

*   **架构设计**:
    *   **Function/Filter/Test Traits**: 扩展性极强。通过实现 `Function`, `Filter`, `Test` trait (`builtins/mod.rs`)，可以轻松注入自定义逻辑。
    *   **Inheritance (继承)**: 支持 `extends` 和 `block`，允许模板复用和覆盖，这在处理复杂文档结构时非常有用。
    *   **Whitespace Control**: 通过 `{%-` 和 `-%}` 精确控制空白字符 (`parser/whitespace.rs`)，这对生成对格式敏感的文本（如代码或特定数据格式）至关重要。

*   **语法特性**:
    *   **Delimiters**: 变量 `{{ ... }}`, 标签 `{% ... %}`, 注释 `{# ... #}`。
    *   **Filters**: 管道风格 `value | filter(arg=1)`。
    *   **Control Flow**: 丰富的 `if`, `for`, `include`, `macro` 支持。
    *   **Tests**: `is` 操作符，如 `if variable is defined`。

*   **优点**:
    *   **类型安全与动态性的平衡**: 基于 Rust 但提供动态类型体验 (JSON Value)。
    *   **极其成熟的语法**: Jinja2 风格经过了时间的考验，用户认知负担小。
    *   **错误处理**: 提供了详细的错误定位和上下文 (`errors.rs`)。

*   **缺点**:
    *   **Allocation Heavy**: 这里的渲染过程涉及大量的 `String` 分配和 `Value` 克隆 (Clone-on-write via `Cow`)，对于高性能流式数据处理（如 `tva` 的目标）来说，开销可能过大。
    *   **Text-oriented**: 核心目标是拼接文本，而不是数据结构的转换。

## 2. TVA 表达式设计

### 2.1 设计原则与哲学

**核心理念：单线程极致性能 + 外部并行工具**

`tva` 坚持在绝大多数数据处理场景下采用**单线程**模型。这并非技术限制，而是基于 Unix 哲学的主动选择：
1.  **专注 (Do One Thing Well)**: `tva` 专注于流式数据的解析、转换与统计，将并行调度的复杂性交给专业的工具（如 GNU Parallel）。
2.  **避免重复造轮子**: GNU Parallel 已经是一个极其成熟、功能强大的并行任务调度器。与其在 `tva` 内部实现复杂的线程池和任务分发，不如让 `tva` 成为 Parallel 的最佳拍档。
3.  **确定性与简单性**: 单线程模型使得数据处理顺序（Order）天然确定，调试更简单，且极大地降低了内存管理的复杂度和开销（无锁、零拷贝更容易实现）。

**设计原则**：
*   **简洁性 (Conciseness)**: 对于常见操作（如列引用），语法应尽可能短。
*   **类型感知 (Type-aware)**: 能够在需要时识别数字、日期等类型，但在默认情况下作为字节串处理以保持速度。
*   **Shell 友好 (Shell-friendly)**: 语法设计避免与 Shell 特殊字符冲突（如 `$`, `!`, `` ` ``），减少用户转义负担。由于表达式通常在命令行中运行且被引号包裹，应尽量避免触发 Shell 的变量替换（如 `$var`）。
*   **流式处理 (Streaming)**: 表达式按行求值，无全局状态，适合大数据处理。
*   **错误处理 (Error Handling)**: 默认采用宽容模式，无效操作返回 `null` 而非报错，但可通过严格模式开关改变行为。
*   **一致性 (Consistency)**: 与现有工具（如 jq、xan）保持相似性，降低学习成本。
*   **Parallel 兼容**: 当用户需要并行处理时，通常是 `parallel` 调用 `tva` 的模式（例如 `parallel "tva ... {}"`），因此 `tva` 的内部语法不应干扰 `parallel` 的参数替换机制。

### 2.2 语法草案 (Draft)

#### 2.2.1 字面量 (Literals)

| 类型 | 语法 | 示例 |
| :--- | :--- | :--- |
| 整数 | 数字序列 | `42`, `-10`, `1_000_000` |
| 浮点数 | 小数点 | `3.14`, `-0.5`, `1e10` |
| 字符串 | 单/双引号 | `"hello"`, `'world'` |
| 布尔 | `true` / `false` | `true`, `false` |
| 空值 | `null` | `null` |
| 列表 | 方括号 | `[1, 2, 3]`, `["a", "b"]` |

#### 2.2.2 列引用 (Column Reference)

使用 `@` 前缀引用列，避免与 Shell 变量冲突：

| 语法 | 含义 | 示例 |
| :--- | :--- | :--- |
| `@0` | 整行内容 | `@0` 表示当前行所有列 |
| `@1`, `@2` | 1-based 列索引 | `@1` 表示第1列 |
| `@col_name` | 列名引用 | `@price` 表示 price 列 |
| `@"col name"` | 含空格的列名 | `@"user name"` |

**设计理由**：
- **Shell 友好**：`@` 在 bash/zsh 中无特殊含义，无需转义
- **简洁高效**：仅需 2 个字符（`Shift+2`）

#### 2.2.3 变量绑定 (Variable Binding)

使用 `as` 关键字将表达式结果绑定到变量，在后续管道中复用：

```rust
// 基本语法
@price * @qty as @total | @total * (1 + @tax_rate)

// 复用中间结果
@name | split(" ") as @parts | first(@parts) ++ "." ++ last(@parts)

// 多变量绑定
@price as @p | @qty as @q | @p * @q
```

**规则**：
- 变量在当前行内有效，进入下一行自动清空
- 可遮蔽同名列引用（`@price as @price`）
- 列引用优先检查变量，再回退到列名查找

**设计说明**：
- 采用 `as` 与管道语义一致（"将左侧结果命名为..."）
- 统一使用 `@` 前缀，降低认知负担
- 参考 jq 语法但去掉 `$` 避免 Shell 冲突

#### 2.2.4 操作符 (Operators)

操作符按优先级从高到低排列：

* `()` - 分组，如 `(@price + @tax) * @qty`
* `-` (一元) - 负号，如 `-@price`, `-(10 + 5)`
* `**` - 乘方，如 `2 ** 10`, `@base ** @exp`
* `*`, `/`, `%` - 乘除模，如 `@price * 1.1`
* `+`, `-` (二元) - 加减，如 `@a + @b`
* `++` - 字符串拼接，如 `@first ++ " " ++ @last`
* `==`, `!=`, `<`, `<=`, `>`, `>=` - 数值比较，如 `@age >= 18`
* `and`, `or`, `not` - 逻辑与或非，如 `@age > 18 and @age < 65`
* `|` - 管道（函数链，最低优先级），如 `@name | trim() | upper()`

**注意**：
- 无隐式类型转换，字符串比较需显式使用函数
- 管道操作符 `|` 作为分隔符，将左侧结果作为第一个参数传递给右侧函数

#### 2.2.5 类型系统 (Type System)

TVA 采用动态类型系统，运行时自动识别类型：

| 类型 | 说明 | 转换规则 |
| :--- | :--- | :--- |
| `Int` | 64位有符号整数 | 字符串解析失败时返回 `null` |
| `Float` | 64位浮点数 | 整数可自动提升为浮点数 |
| `String` | UTF-8 字符串 | 数字/布尔可显式转换为字符串 |
| `Bool` | 布尔值 | 空字符串、0、`null` 为假，其余为真 |
| `Null` | 空值 | 表示缺失或无效数据 |
| `List` | 异构列表 | 元素可以是任意类型 |

**类型转换**：
- **显式转换**: 使用 `int()`, `float()`, `string()` 函数
- **数值运算**: 整数与浮点数混合运算时，结果提升为浮点数
- **字符串拼接**: `++` 操作符会将操作数转为字符串
- **比较操作**: 同类型比较，不同类型始终返回 `false`

**列引用类型**：
- 列引用 `@col` 默认返回 `String`（原始字节）
- 数值运算时自动尝试解析为数字，解析失败视为 `null`
- 可用 `int(@col)` 或 `float(@col)` 显式指定类型

#### 2.2.6 函数调用 (Function Call)

-   **前缀调用**: `func(arg1, arg2, ...)`，如 `trim(@name)`, `substr(@desc, 0, 50)`
-   **管道调用（单参数函数）**: `arg | func()`，如 `@name | trim() | upper()`
    -   单参数函数在管道中可省略 `_` 占位符
-   **管道调用（多参数函数）**: `arg | func(_, arg2)`，如 `@name | substr(_, 0, 5)`
    -   使用 `_` 表示管道左侧的值作为第一个参数

#### 2.2.7 表达式 (Expression)

TVA 表达式由以下元素组合而成：

**原子元素**：
- **列引用**: `@1`, `@col_name` - 输入数据的列值
- **变量**: `@var_name` - 通过 `as` 绑定的变量
- **字面量**: `42`, `"hello"`, `true`, `null`, `[1, 2, 3]` - 常量值
- **函数调用**: `func(args...)` - 内置函数

**组合方式**：
- **操作符组合**: `@a + @b`, `@x > 10 and @y < 20`
- **管道组合**: `@name | trim() | upper()`
- **变量绑定**: `expr as @var | @var + 1`
- **函数嵌套**: `if(@age > 18, "adult", "minor")`

**求值规则**：
- 表达式按操作符优先级从左到右求值
- 管道操作符 `|` 优先级最低，用于连接多个处理步骤
- 变量绑定 `as` 在管道中立即生效，后续步骤可引用

**多表达式**：
- 使用 `;` 分隔多个表达式，按顺序依次求值
- 只有最后一个表达式的值会被输出，前面的表达式用于副作用（如变量绑定、调试打印）
- 示例：`@price as @p; @qty as @q; @p * @q`

**输出行为**：
- 在 `tva eval` 中，最后一个表达式的值会打印到标准输出
- `print(val, ...)` 函数将多个参数依次输出，返回最后一个参数的值。如果 print() 是最后一个表达式，不会双重打印。
- 示例：`@price | print("price:", _); @qty | print("qty:", _); @price * @qty`

#### 2.2.8 注释 (Comments)

TVA 支持行注释，以 `//` 开头：

```rust
// 计算总价
@price * @qty as @total | @total * (1 + @tax_rate)  // 含税
```

**注意**：注释仅在表达式内部有效，命令行中的注释由 Shell 处理。

#### 2.2.9 完整示例

```bash
# 过滤：价格大于100且库存大于0
@price > 100 and @stock > 0

# 计算：总价（含税）
@price * (1 + @tax_rate)

# 字符串处理：格式化姓名
@first_name | trim() | upper() ++ " " ++ @last_name | trim() | upper()

# 条件：根据年龄分组
if(@age < 18, "minor", if(@age < 60, "adult", "senior"))

# 正则匹配：验证邮箱格式
regex_match(@email, "^[\\w.-]+@[\\w.-]+\\.\\w+$")

# 默认值：处理可能为空的字段
default(@nickname, @username)

# 单参数函数管道
@name | trim() | upper()

# 多参数函数管道（使用 _ 占位符）
@desc | substr(_, 0, 50) | upper()

// 多表达式与变量绑定
@price as @p; @qty as @q; @p * @q

// 复杂管道与分号组合
@price | int() as @p | @p * 2; @qty | int() as @q | @q * 3; @p + @q

// 使用注释解释逻辑
@total | int() as @t;  // 转为整数
@tax | float() as @r;  // 税率转为浮点数
@t * (1 + @r)          // 计算含税总价
```

### 2.3 函数库参考 (Standard Library Reference)

#### 1. 字符串处理 (String Manipulation)

| 函数 | 说明 | 参考 |
| :--- | :--- | :--- |
| `trim(str) -> str` | 去除首尾空白 | Tera/xan/jq |
| `upper(str) -> str` | 转大写 | Tera/xan |
| `lower(str) -> str` | 转小写 | Tera/xan |
| `len(str) -> int` | 字节长度 | Tera/jq |
| `substr(str, start, len) -> str` | 获取子串 | SQL/Go |
| `split(str, pat) -> list` | 按分隔符分割 | xan/jq |
| `contains(str, substr) -> bool` | 是否包含子串 | - |
| `starts_with(str, prefix) -> bool` | 是否以指定前缀开头 | - |
| `ends_with(str, suffix) -> bool` | 是否以指定后缀结尾 | - |
| `replace(str, from, to) -> str` | 字符串替换 | Tera/xan |
| `truncate(str, len, end?) -> str` | 截断字符串 | Tera |
| `wordcount(str) -> int` | 单词计数 | Tera |
| `char_len(str) -> int` | 字符数 (UTF-8) | - |

#### 2. 数值与转换 (Numeric & Conversion)

| 函数 | 说明 | 参考 |
| :--- | :--- | :--- |
| `abs(num) -> num` | 绝对值 | Tera/xan |
| `round(num) -> int` | 四舍五入 | Tera/xan |
| `min(a, b, ...) -> num` | 最小值 | xan |
| `max(a, b, ...) -> num` | 最大值 | xan |
| `int(val) -> int` | 解析为整数 | Python/Tera |
| `float(val) -> float` | 解析为浮点数 | Python/Tera |
| `ceil(num) -> int` | 向上取整 | Tera |
| `floor(num) -> int` | 向下取整 | Tera |
| `sqrt(num) -> float` | 平方根 | - |
| `pow(base, exp) -> float` | 幂运算 | - |
| `sin(num) -> float` | 正弦函数（弧度） | AWK |
| `cos(num) -> float` | 余弦函数（弧度） | AWK |
| `tan(num) -> float` | 正切函数（弧度） | AWK |
| `ln(num) -> float` | 自然对数 | AWK |
| `log10(num) -> float` | 常用对数（以10为底） | AWK |
| `exp(num) -> float` | 指数函数 e^x | AWK |

#### 3. 逻辑与控制 (Logic & Control)

| 函数 | 说明 | 参考 |
| :--- | :--- | :--- |
| `if(cond, then, else)` | 三元表达式 | Excel/SQL |
| `default(val, fallback)` | 空值回退 | Tera/jq |
| `print(val, ...)` | 打印到 stdout，返回最后一个参数 | - |
| `eprint(val, ...)` | 打印到 stderr，返回最后一个参数 | - |

#### 4. 列表/数组函数 (List Functions)

*注：以下函数操作的是表达式中的 `List` 类型（如 `split()` 返回的列表），与 `stats` 命令的列级聚合不同。*

| 函数 | 说明 | 参考 |
| :--- | :--- | :--- |
| `join(list, sep) -> str` | 将列表连接成字符串 | xan/jq |
| `first(list) -> val` | 获取第一个元素 | xan/jq |
| `last(list) -> val` | 获取最后一个元素 | xan/jq |
| `nth(list, n) -> val` | 获取第 n 个元素 | - |
| `reverse(list) -> list` | 反转列表 | - |
| `sort(list) -> list` | 排序 | - |
| `unique(list) -> list` | 去重 | - |
| `slice(list, start, end?) -> list` | 切片 | - |
| `reduce(list, init, op) -> val` | 列表归约 | xan/jq |

#### 5. 正则表达式 (Regex)

*注：正则操作通常开销较大，应谨慎使用。*

| 函数 | 说明 | 参考 |
| :--- | :--- | :--- |
| `regex_match(str, pattern) -> bool` | 是否匹配 | xan/jq |
| `regex_extract(str, pattern, group?) -> str` | 提取捕获组 | xan |
| `regex_replace(str, pattern, to) -> str` | 正则替换 | xan/jq |

#### 6. 编码与哈希 (Encoding & Hashing)

| 函数 | 说明 | 参考 |
| :--- | :--- | :--- |
| `md5(str) -> str` | MD5 哈希 | xan |
| `sha256(str) -> str` | SHA256 哈希 | xan |
| `base64(str) -> str` | Base64 编码 | xan/jq |
| `unbase64(str) -> str` | Base64 解码 | xan/jq |

#### 7. 日期时间 (Date & Time)

| 函数 | 说明 | 参考 |
| :--- | :--- | :--- |
| `now() -> datetime` | 当前时间 (UTC) | Tera |
| `strptime(str, fmt) -> datetime` | 字符串转时间 | Python/jq |
| `strftime(datetime, fmt) -> str` | 时间转字符串 | Python/jq |

### 2.4 高级表达式特性

#### 2.4.1 管道操作符

管道操作符 `|` 用于函数链式调用：

```rust
// 单参数函数管道（_ 可省略）
@name | trim() | upper()

// 多参数函数管道（使用 _ 占位符）
@desc | substr(_, 0, 50) | upper()

// 复杂管道
@email | trim() | lower() | regex_match(_, ".*@.*\\.com")
```

#### 2.4.2 多表达式

使用 `;` 分隔多个表达式，按顺序依次求值：

```rust
// 多表达式与变量绑定
@price as @p; @qty as @q; @p * @q

// 复杂管道与分号组合
@price | int() as @p | @p * 2; @qty | int() as @q | @q * 3; @p + @q

// 使用注释解释逻辑
@total | int() as @t;  // 转为整数
@tax | float() as @r;  // 税率转为浮点数
@t * (1 + @r)          // 计算含税总价
```

**规则**：
- 每个表达式可以产生副作用（如变量绑定）
- 只有最后一个表达式的值被返回

#### 2.4.3 方法调用语法

方法调用是函数调用的语法糖：

```rust
// 方法调用等价于函数调用
@name.trim()           // 等价于 trim(@name)
@price.round()         // 等价于 round(@price)

// 方法链
@name.trim().upper().substr(0, 5)
// 等价于 substr(upper(trim(@name)), 0, 5)

// 带参数的方法调用
@name.substr(0, 5)     // 等价于 substr(@name, 0, 5)
@price.pow(2)          // 等价于 pow(@price, 2)
```

## 3. 实施路线图 (Implementation Roadmap)

基于对 `xan/moonblade` 和 `tera` 的深度分析，我们计划为 `tva` 构建一个轻量级、高性能的表达式引擎。

### 3.1 参考架构分析

#### 3.1.1 xan/moonblade 架构

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

#### 3.1.2 Tera 架构

**核心组件**:
-   **tera.pest**: 模板语法定义
-   **ast.rs**: AST 节点定义
-   **whitespace.rs**: 空白字符控制逻辑
-   **Renderer**: 树遍历渲染器

**关键设计模式**:
-   **Whitespace Control**: `{%-` 和 `-%}` 精确控制空白
-   **Filter Chain**: `value | filter1 | filter2`
-   **Macro System**: 模板级别的函数抽象

### 3.2 TVA 表达式引擎设计

#### 3.2.1 设计原则

1.  **极简主义**: 只实现最核心的功能，避免过度设计
2.  **性能优先**: 零拷贝、预编译、静态分析
3.  **Shell 友好**: 语法避免与 Shell 特殊字符冲突
4.  **类型安全**: 显式类型转换，无隐式转换

#### 3.2.2 项目结构

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
└── functions/
    └── mod.rs          # 函数注册和实现
```

### 3.3 实施路线图

表达式引擎已完整实现，共 54 个函数，支持变量绑定、管道操作、方法调用等高级特性。

#### 已实现功能概览

| 类别 | 函数 | 数量 |
| :--- | :--- | :--- |
| 字符串 | `trim`, `upper`, `lower`, `len`, `substr`, `split`, `contains`, `starts_with`, `ends_with`, `replace`, `truncate`, `wordcount`, `char_len` | 13 |
| 数值 | `abs`, `round`, `min`, `max`, `int`, `float`, `ceil`, `floor`, `sqrt`, `pow` | 10 |
| 逻辑 | `if`, `default`, `print`, `eprint` | 4 |
| 列表 | `join`, `first`, `last`, `nth`, `reverse`, `sort`, `unique`, `slice`, `reduce` | 9 |
| 正则 | `regex_match`, `regex_extract`, `regex_replace` | 3 |
| 编码哈希 | `md5`, `sha256`, `base64`, `unbase64` | 4 |
| 日期时间 | `now`, `strptime`, `strftime` | 3 |
| 方法调用 | `@name.trim()`, `@price.round()` | 语法特性 |

**新增依赖**:
-   `regex = "1.11"` - 正则表达式支持
-   `md5 = "0.7"`, `sha2 = "0.10"`, `base64 = "0.22"` - 编码哈希
-   `chrono = "0.4"` - 日期时间处理

#### 待完成任务

**`tva mutate` - 计算新列/修改列**（优先实现，相对简单）

*设计*:
```bash
tva mutate -E "@price * 1.1 as @new_price" data.tsv
tva mutate -E "@first ++ ' ' ++ @last as @full_name" data.tsv
```

*实现要点*:
-   复用现有 `TsvReader` 读取输入
-   解析表达式，计算新列值
-   输出原始列 + 新列（或替换现有列）
-   支持 `--header` 模式自动添加新列表头

*工作量*: 较小（1-2天）
*状态*: 📝 待实现

---

**`tva filter -E` - 表达式过滤**（复杂，需深度整合现有 filter 模块）

*设计*:
```bash
tva filter -E "@price > 100 and @stock > 0" data.tsv
tva filter -E "@name | contains(_, 'John')" data.tsv
```

*实现规划*:

1.  **架构改造** (`engine.rs`, `config.rs`)
    -   新增 `TestKind::Expression { expr: Expr }` 测试类型
    -   `FilterConfig` 添加 `expression: Option<String>` 字段
    -   `-E` 参数与现有过滤条件互斥

2.  **Builder 适配** (`builder.rs`)
    -   表达式解析为 AST
    -   列名静态解析（`@price` → 索引）

*工作量*: 中等（3-5天）
*状态*: 📝 待实现
    -   支持预编译优化

3.  **Runner 集成** (`runner.rs`)
    -   行处理循环中创建 `EvalContext`
    -   表达式求值返回 bool 决定行是否保留
    -   复用现有 header/label/invert 逻辑

4.  **CLI 集成** (`cmd_tva/filter.rs`)
    -   添加 `-E, --expression` 参数
    -   参数互斥验证

*工作量*: 较大（3-5天）

*依赖*: 建议先完成 `tva mutate` 验证表达式引擎在命令集成中的稳定性

### 3.4 技术选型

| 组件 | 选择 | 理由 |
| :--- | :--- | :--- |
| Parser Generator | `pest` | 成熟的 PEG 解析器，xan/tera 都在使用 |
| Parser Algorithm | Pratt Parser | 处理操作符优先级的标准方案 |
| String Storage | `Arc<str>` | 零拷贝共享，减少内存分配 |
| Error Handling | `thiserror` | 与现有代码库一致 |
| Regex | `regex` crate | Rust 生态标准，性能优秀 |

### 3.4 实现要点

#### 3.4.1 EvalContext 结构

```rust
pub struct EvalContext<'a> {
    pub row: &'a [String],
    pub headers: Option<&'a [String]>,
    pub variables: HashMap<String, Value>,  // 行级变量存储
}
```

#### 3.4.2 管道求值的 AST 表示

```rust
pub enum PipeRight {
    Call { name: String, args: Vec<Expr> },           // func() - 左侧值作为首参
    CallWithPlaceholder { name: String, args: Vec<Expr> },  // func(_, arg2)
}
```

### 3.5 风险和对策

| 风险 | 影响 | 对策 |
| :--- | :--- | :--- |
| 解析性能不足 | 高 | 使用 Pratt Parser，预编译表达式 |
| 内存占用过高 | 中 | Arc 共享字符串，缓冲区复用 |
| 功能过度设计 | 中 | 严格遵循极简原则，分阶段实施 |
| 与现有命令冲突 | 低 | 保持向后兼容，新增 `-E` 选项 |

### 3.6 参考资源

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
