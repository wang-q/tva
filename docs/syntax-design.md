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

#### AWK 内置函数参考

AWK 提供了丰富的内置函数，主要分为以下几类：

**数值函数**:
*   `atan2(y, x)` - 返回 y/x 的反正切值（弧度）
*   `cos(x)` - 余弦函数
*   `exp(x)` - 指数函数 e^x
*   `int(x)` - 取整（向零截断）
*   `log(x)` - 自然对数
*   `rand()` - 返回 [0,1) 之间的随机数
*   `sin(x)` - 正弦函数
*   `sqrt(x)` - 平方根
*   `srand([x])` - 设置随机数种子

**字符串函数**:
*   `gsub(regex, repl [, target])` - 全局替换
*   `index(s, t)` - 返回子串 t 在 s 中的位置（从1开始）
*   `length([s])` - 字符串长度
*   `match(s, regex [, arr])` - 正则匹配，返回匹配位置
*   `split(s, arr [, fs [, seps]])` - 分割字符串到数组
*   `sprintf(fmt, ...)` - 格式化字符串
*   `sub(regex, repl [, target])` - 替换第一个匹配
*   `substr(s, i [, n])` - 提取子串
*   `tolower(s)` / `toupper(s)` - 大小写转换

**时间函数 (gawk 扩展)**:
*   `mktime(datespec [, utc])` - 将日期字符串转为时间戳
*   `strftime([fmt [, ts [, utc]]])` - 格式化时间
*   `systime()` - 返回当前时间戳

**位操作函数 (gawk 扩展)**:
*   `and(v1, v2, ...)` - 按位与
*   `compl(val)` - 按位取反
*   `lshift(val, n)` / `rshift(val, n)` - 左/右移位
*   `or(v1, v2, ...)` - 按位或
*   `xor(v1, v2, ...)` - 按位异或

#### AWK 特殊模式: BEGIN 与 END

AWK 提供了两个特殊模式 `BEGIN` 和 `END`，用于在程序执行前后执行初始化或清理操作：

**BEGIN 模式**:
*   在处理任何输入记录之前执行一次
*   常用于初始化变量、设置字段分隔符 `FS`、打印表头等
*   程序可以包含多个 `BEGIN` 规则，按出现顺序执行
*   如果程序只有 `BEGIN` 规则而没有其他规则，处理完 `BEGIN` 后程序直接退出

**END 模式**:
*   在所有输入处理完毕后执行一次
*   常用于输出汇总统计信息
*   程序可以包含多个 `END` 规则，按出现顺序执行
*   即使程序只有 `END` 规则，也会读取输入（以便 `NR` 和 `FNR` 等变量有意义）

**示例**:
```awk
awk '
    BEGIN { print "Analysis Start"; FS = "," }
    /pattern/ { count++ }
    END { print "Total matches:", count }
' data.txt
```

**BEGINFILE 与 ENDFILE (gawk 扩展)**:
*   `BEGINFILE` - 在处理每个文件之前执行
*   `ENDFILE` - 在处理每个文件之后执行
*   适用于多文件处理时的文件级初始化和清理

### 1.3 xan (Moonblade): 动态表达式引擎 (Dynamic Expression Engine)

xan 内置了一个名为 **Moonblade** 的动态脚本语言，其设计类似于简化版的 Python/JavaScript，专为 CSV 处理优化。

*   **核心机制**:
    *   **Parser & Interpreter**: 基于 PEG 解析器 (Pest) 构建 AST，采用树遍历解释器 (Tree-Walker Interpreter)。
    *   **Concretization (具体化)**: 在执行前进行静态分析优化，例如将列名引用静态解析为索引，避免运行时的哈希查找开销。
    *   **Arc-based Types**: 使用 `Arc<T>` 管理字符串和复杂对象，尽可能减少深拷贝，适合流式处理。

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

#### 语法参考 (Syntax Reference)

基于 `xan` 的文档整理，Moonblade 的核心语法如下：

1.  **基础类型 (Types)**:
    *   **数值**: `1`, `10_000`, `0.5`
    *   **布尔**: `true`, `false`
    *   **空值**: `null`
    *   **字符串**: `"hello"`, `'hello'`, `` `hello` `` (支持转义 `\n`, `\t` 等)
    *   **二进制**: `b"hello"`
    *   **正则**: `/pattern/i`
    *   **集合**: List `[1, 2]`, Map `{"a": 1, b: 2}`

2.  **列引用 (Columns)**:
    *   **直接引用**: `count` (仅限字母数字下划线且不以数字开头)
    *   **函数引用**: `col("Name")`, `col(0)` (索引支持负数), `col("text", 1)` (同名列的第二个)
    *   **安全引用**: `col?("Name")` (不存在则返回 null)

3.  **操作符 (Operators)**:
    *   **算术**: `+`, `-`, `*`, `/`, `%`, `//` (整除), `**` (幂)
    *   **数值比较**: `==`, `!=`, `<`, `<=`, `>`, `>=`
    *   **字符串比较**: `eq`, `ne`, `lt`, `le`, `gt`, `ge` (Perl 风格)
    *   **字符串拼接**: `++`
    *   **逻辑**: `&&` (`and`), `||` (`or`), `!` (`not`), `in`, `not in`
    *   **管道**: `|` (使用 `_` 指代左侧结果，如 `trim(name) | len(_)`)

4.  **函数调用 (Functions)**:
    *   **普通调用**: `trim(name)`
    *   **方法调用**: `name.trim()` (等价于 `trim(name)`)
    *   **命名参数**: `read(path, encoding="utf8")`

5.  **索引与切片 (Indexing & Slicing)**:
    *   `list[0]`, `list[-1]`
    *   `list[1:3]`, `list[:2]`, `list[1:]`
    *   `map["key"]` 或 `map.key`

6.  **高阶函数 (Higher-order)**:
    *   Lambda 表达式: `map(numbers, x -> x + 2)`

7.  **命名表达式 (Named Expressions)**:
    *   用于 `map`/`agg` 等命令: `sum(count) as total_count`
    *   解构: `full_name.split(" ") as (first, last)`

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

#### 语法参考 (Syntax Reference)

基于 `jq` 的官方文档整理，核心语法如下：

1.  **基础类型 (Types)**:
    *   **数值**: `42`, `3.14`
    *   **布尔**: `true`, `false`
    *   **空值**: `null`
    *   **字符串**: `"hello"`, `"hello\nworld"`, `"price: \(10 * 2)"` (插值)
    *   **集合**: Array `[1, 2, 3]`, Object `{"key": "value", "id": 1}`

2.  **核心过滤器 (Core Filters)**:
    *   **Identity**: `.` (返回当前输入)
    *   **Object Identifier-Index**: `.foo`, `.["foo"]` (字段访问)
    *   **Generic Object Index**: `.[<string>]` (支持动态 key)
    *   **Array Index**: `.[0]`, `.[-1]`
    *   **Array/String Slice**: `.[1:3]`, `.[1:]` (切片)
    *   **Iterator**: `.[]`, `.foo[]` (展开为流)
    *   **Optional**: `.foo?` (出错时不报错，返回空)

3.  **组合与控制流 (Combination & Control)**:
    *   **Pipe**: `|` (将左侧输出作为右侧输入)
    *   **Comma**: `,` (产生多个输出，如 `.a, .b`)
    *   **Parenthesis**: `(. + 2) * 5` (分组)
    *   **Conditionals**: `if A then B else C end`, `if A then B elif C then D else E end`
    *   **Try-Catch**: `try .a catch .b`
    *   **Suppress Errors**: `?` (如 `try .a` 等价于 `.a?`)

4.  **操作符 (Operators)**:
    *   **算术**: `+`, `-`, `*`, `/`, `%`
    *   **比较**: `==`, `!=`, `<`, `>`, `<=`, `>=`
    *   **逻辑**: `and`, `or`, `not`
    *   **Alternative**: `//` (如 `.a // .b`，若 .a 为 null/false 则返回 .b)
    *   **Update**: `|=`, `+=`, `-=` (如 `.count += 1`)

5.  **构造 (Construction)**:
    *   **Array**: `[.a, .b]` (收集流为数组)
    *   **Object**: `{id: .id, name}` (简写 `{name}` 等价于 `{name: .name}`)

6.  **函数 (Functions)**:
    *   **标准库**: `length`, `keys`, `has("key")`, `map(f)`, `select(f)`
    *   **字符串**: `split(" ")`, `join(", ")`, `tostring`, `tonumber`
    *   **正则**: `test("regex")`, `match("regex")`, `capture("regex")`
    *   **归约**: `reduce .[] as $item (init; update)`

7.  **变量 (Variables)**:
    *   **定义**: `... as $var | ...`
    *   **引用**: `$var`

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

#### 语法参考 (Syntax Reference)

基于 `Tera` 的文档整理，核心语法如下：

1.  **变量与表达式**:
    *   **输出**: `{{ name }}`, `{{ user.age + 1 }}`
    *   **字面量**: `true`, `42`, `3.14`, `"hello"`, `[1, 2]`, `{a: 1}`

2.  **过滤器 (Filters)**:
    *   **调用**: `{{ name | upper | replace(from="x", to="y") }}`
    *   **常用**: `round`, `length`, `date`, `json_encode`

3.  **控制流**:
    *   **条件**: `{% if user.is_admin %} ... {% elif user.is_mod %} ... {% else %} ... {% endif %}`
    *   **循环**: `{% for item in items %} {{ loop.index }} - {{ item }} {% endfor %}`

4.  **测试 (Tests)**:
    *   **语法**: `{% if number is odd %}`
    *   **常用**: `defined`, `string`, `iterable`, `starting_with`

5.  **宏 (Macros)**:
    *   **定义**: `{% macro input(type="text") %} <input type="{{type}}"> {% endmacro %}`
    *   **调用**: `{{ macros::input(type="password") }}`

6.  **空白控制**:
    *   `{%-` (去除左侧空白), `-%}` (去除右侧空白)

### 1.5 TVA 的定位与设计哲学

**核心理念：单线程极致性能 + 外部并行工具**

`tva` 坚持在绝大多数数据处理场景下采用**单线程**模型。这并非技术限制，而是基于 Unix 哲学的主动选择：
1.  **专注 (Do One Thing Well)**: `tva` 专注于流式数据的解析、转换与统计，将并行调度的复杂性交给专业的工具（如 GNU Parallel）。
2.  **避免重复造轮子**: GNU Parallel 已经是一个极其成熟、功能强大的并行任务调度器。与其在 `tva` 内部实现复杂的线程池和任务分发，不如让 `tva` 成为 Parallel 的最佳拍档。
3.  **确定性与简单性**: 单线程模型使得数据处理顺序（Order）天然确定，调试更简单，且极大地降低了内存管理的复杂度和开销（无锁、零拷贝更容易实现）。

**设计推论**：
*   **Shell 友好性**: 由于 `tva` 表达式通常在命令行中运行且被引号包裹，语法设计应尽量避免触发 Shell 的变量替换（如 `$var`），以减少用户转义的负担。
*   当用户需要并行处理时，通常是 `parallel` 调用 `tva` 的模式（例如 `parallel "tva ... {}"`），因此 `tva` 的内部语法不应干扰 `parallel` 的参数替换机制。

## 2. TVA 格式设计提案 (Proposal)

设计一种既有模板的轻量级，又有一定逻辑处理能力的格式。

### 2.1 设计原则

**简洁性 (Conciseness)**: 对于常见操作（如列引用），语法应尽可能短。
**类型感知 (Type-aware)**: 能够在需要时识别数字、日期等类型，但在默认情况下作为字节串处理以保持速度。

### 2.2 语法草案 (Draft)

#### 2.2.1 列引用 (Column Reference)

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
- **视觉区分**：明显区别于普通变量和字符串字面量

#### 2.2.2 字面量 (Literals)

| 类型 | 语法 | 示例 |
| :--- | :--- | :--- |
| 整数 | 数字序列 | `42`, `-10`, `1_000_000` |
| 浮点数 | 小数点 | `3.14`, `-0.5`, `1e10` |
| 字符串 | 单/双引号 | `"hello"`, `'world'` |
| 布尔 | `true` / `false` | `true`, `false` |
| 空值 | `null` | `null` |

#### 2.2.3 操作符 (Operators)

操作符按优先级从高到低排列：

1.  `()` - 分组，如 `(@price + @tax) * @qty`
2.  `-` (一元) - 负号，如 `-@price`, `-(10 + 5)`
3.  `**` - 乘方，如 `2 ** 10`, `@base ** @exp`
4.  `*`, `/`, `%` - 乘除模，如 `@price * 1.1`
5.  `+`, `-` (二元) - 加减，如 `@a + @b`
6.  `++` - 字符串拼接，如 `@first ++ " " ++ @last`
7.  `==`, `!=`, `<`, `<=`, `>`, `>=` - 数值比较，如 `@age >= 18`
8.  `and`, `or`, `not` / `&&`, `||`, `!` - 逻辑与或非，如 `@age > 18 and @age < 65`
9.  `|` - 管道（函数链，最低优先级），如 `@name | trim() | upper()`

**注意**：
- 无隐式类型转换，字符串比较需显式使用函数
- 管道操作符 `|` 作为分隔符，将左侧结果作为第一个参数传递给右侧函数
- 管道中的函数可以直接调用，无需 `_` 占位符

#### 2.2.4 函数调用 (Function Call)

-   **前缀调用**: `func(arg1, arg2, ...)`，如 `trim(@name)`, `substr(@desc, 0, 50)`
-   **管道调用（单参数函数）**: `arg | func()`，如 `@name | trim() | upper()`
    -   单参数函数在管道中可省略 `_` 占位符
-   **管道调用（多参数函数）**: `arg | func(_, arg2)`，如 `@name | substr(_, 0, 5)`
    -   使用 `_` 表示管道左侧的值作为第一个参数

管道左侧的值会作为函数的第一个参数传入。

#### 2.2.5 完整示例

```bash
# 过滤：价格大于100且库存大于0
@price > 100 && @stock > 0

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
```

### 2.3 函数库提案 (Standard Library Proposal)

#### 1. 字符串处理 (String Manipulation)

| 函数 | 签名 | 说明 | 状态 | 参考 |
| :--- | :--- | :--- | :--- | :--- |
| `trim` | `trim(str) -> str` | 去除首尾空白 | ✅ 已实现 | Tera/xan/jq |
| `upper` | `upper(str) -> str` | 转大写 | ✅ 已实现 | Tera/xan |
| `lower` | `lower(str) -> str` | 转小写 | ✅ 已实现 | Tera/xan |
| `len` | `len(str) -> int` | 字节长度 | ✅ 已实现 | Tera/jq |
| `substr` | `substr(str, start, len) -> str` | 获取子串 | ✅ 已实现 | SQL/Go |
| `split` | `split(str, pat) -> list` | 按分隔符分割 | ✅ 已实现 | xan/jq |
| `contains` | `contains(str, substr) -> bool` | 是否包含子串 | ✅ 已实现 | - |
| `starts_with` | `starts_with(str, prefix) -> bool` | 是否以指定前缀开头 | ✅ 已实现 | - |
| `ends_with` | `ends_with(str, suffix) -> bool` | 是否以指定后缀结尾 | ✅ 已实现 | - |
| `replace` | `replace(str, from, to) -> str` | 字符串替换 | 📝 待实现 | Tera/xan |
| `truncate` | `truncate(str, len, end?) -> str` | 截断字符串 | 📝 待实现 | Tera |
| `wordcount` | `wordcount(str) -> int` | 单词计数 | 📝 待实现 | Tera |
| `char_len` | `char_len(str) -> int` | 字符数 (UTF-8) | 📝 待实现 | - |

#### 2. 数值与转换 (Numeric & Conversion)

| 函数 | 签名 | 说明 | 状态 | 参考 |
| :--- | :--- | :--- | :--- | :--- |
| `abs` | `abs(num) -> num` | 绝对值 | ✅ 已实现 | Tera/xan |
| `round` | `round(num) -> int` | 四舍五入 | ✅ 已实现 | Tera/xan |
| `min` | `min(a, b, ...) -> num` | 最小值 | ✅ 已实现 | xan |
| `max` | `max(a, b, ...) -> num` | 最大值 | ✅ 已实现 | xan |
| `int` | `int(val) -> int` | 解析为整数 | ✅ 已实现 | Python/Tera |
| `float` | `float(val) -> float` | 解析为浮点数 | ✅ 已实现 | Python/Tera |
| `ceil` | `ceil(num) -> int` | 向上取整 | 📝 待实现 | Tera |
| `floor` | `floor(num) -> int` | 向下取整 | 📝 待实现 | Tera |
| `sqrt` | `sqrt(num) -> float` | 平方根 | 📝 待实现 | - |
| `pow` | `pow(base, exp) -> float` | 幂运算 | 📝 待实现 | - |

#### 3. 逻辑与控制 (Logic & Control)

| 函数 | 签名 | 说明 | 状态 | 参考 |
| :--- | :--- | :--- | :--- | :--- |
| `if` | `if(cond, then, else)` | 三元表达式 | ✅ 已实现 | Excel/SQL |
| `default` | `default(val, fallback)` | 空值回退 | ✅ 已实现 | Tera/jq |
| `print` | `print(val, label?)` | 调试打印（输出到 stderr，返回原值） | 📝 待实现 | - |

#### 4. 列表/数组函数 (List Functions)

| 函数 | 签名 | 说明 | 状态 | 参考 |
| :--- | :--- | :--- | :--- | :--- |
| `join` | `join(list, sep) -> str` | 将列表连接成字符串 | 📝 待实现 | xan/jq |
| `first` | `first(list) -> val` | 获取第一个元素 | 📝 待实现 | xan/jq |
| `last` | `last(list) -> val` | 获取最后一个元素 | 📝 待实现 | xan/jq |
| `nth` | `nth(list, n) -> val` | 获取第 n 个元素 | 📝 待实现 | - |
| `reverse` | `reverse(list) -> list` | 反转列表 | 📝 待实现 | - |
| `sort` | `sort(list) -> list` | 排序 | 📝 待实现 | - |
| `unique` | `unique(list) -> list` | 去重 | 📝 待实现 | - |
| `slice` | `slice(list, start, end) -> list` | 切片 | 📝 待实现 | - |

#### 5. 正则表达式 (Regex)

*注：正则操作通常开销较大，应谨慎使用。*

| 函数 | 签名 | 说明 | 状态 | 参考 |
| :--- | :--- | :--- | :--- | :--- |
| `regex_match` | `regex_match(str, pattern) -> bool` | 是否匹配 | 📝 待实现 | xan/jq |
| `regex_extract` | `regex_extract(str, pattern, group?) -> str` | 提取捕获组 | 📝 待实现 | xan |
| `regex_replace` | `regex_replace(str, pattern, to) -> str` | 正则替换 | 📝 待实现 | xan/jq |

#### 6. 编码与哈希 (Encoding & Hashing)

| 函数 | 签名 | 说明 | 状态 | 参考 |
| :--- | :--- | :--- | :--- | :--- |
| `md5` | `md5(str) -> str` | MD5 哈希 | 📝 待实现 | xan |
| `sha256` | `sha256(str) -> str` | SHA256 哈希 | 📝 待实现 | xan |
| `base64` | `base64(str) -> str` | Base64 编码 | 📝 待实现 | xan/jq |
| `unbase64` | `unbase64(str) -> str` | Base64 解码 | 📝 待实现 | xan/jq |

#### 7. 日期时间 (Date & Time)

| 函数 | 签名 | 说明 | 状态 | 参考 |
| :--- | :--- | :--- | :--- | :--- |
| `now` | `now() -> str` | 当前时间 (RFC3339) | 📝 待实现 | Tera |
| `strptime` | `strptime(str, fmt) -> datetime` | 字符串转时间 | 📝 待实现 | Python/jq |
| `strftime` | `strftime(datetime, fmt) -> str` | 时间转字符串 | 📝 待实现 | Python/jq |

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

### 3.3 实施阶段

#### Phase 1: 核心基础设施 (已完成)

**目标**: 建立表达式解析和执行的基础框架

**已完成任务**:
-   ✅ 项目结构搭建
-   ✅ 语法定义 (grammar.pest)
-   ✅ AST 定义 (ast.rs)
-   ✅ Pest Parser 实现
-   ✅ 错误处理 (ParseError, EvalError)

**验收标准**:
-   ✅ 能正确解析简单表达式: `@1 + @2`, `-5`, `2 ** 10`
-   ✅ 支持完整操作符优先级
-   ✅ 60+ 单元测试全部通过

#### Phase 2: 运行时和函数库 (已完成)

**目标**: 实现表达式执行和核心函数库

**已完成任务**:
-   ✅ Value 类型系统 (value.rs)
-   ✅ 解释器实现 (interpreter.rs)
-   ✅ 函数注册系统 (functions/mod.rs)
-   ✅ 核心函数实现 (15个函数)

**已实现的函数**:
-   字符串: `trim`, `upper`, `lower`, `len`, `substr`, `split`, `contains`, `starts_with`, `ends_with`
-   数值: `abs`, `round`, `min`, `max`, `int`, `float`
-   逻辑: `if`, `default`

#### Phase 3: 与 TVA 集成 (部分完成)

**目标**: 将表达式引擎集成到现有命令中

**已完成**:
-   ✅ `tva eval` 命令 - 表达式求值测试工具

**待实现**:
-   `tva filter -E` - 表达式过滤
-   `tva mutate` - 计算新列/修改列

#### Phase 4: 函数库扩展

**目标**: 丰富函数库，覆盖更多使用场景

**待实现函数清单**:

**字符串函数**:
-   `replace(s, from, to)` - 替换子字符串
-   `truncate(s, length, end)` - 截断字符串
-   `wordcount(s)` - 单词计数
-   `regex_match(s, pattern)` - 正则匹配
-   `regex_replace(s, pattern, replacement)` - 正则替换

**数值函数**:
-   `ceil(n)` / `floor(n)` - 向上/向下取整
-   `sqrt(n)` - 平方根
-   `pow(n, exp)` - 幂运算

**列表函数**:
-   `join(list, sep)` - 将列表连接成字符串
-   `first(list)` / `last(list)` - 获取首/尾元素
-   `reverse(list)` - 反转列表
-   `sort(list)` - 排序

#### Phase 5: 高级语法 (可选)

**目标**: 扩展表达式能力

**待实现**:
-   管道变量绑定: `@name | split(" ") as parts | first(parts)`
-   正则字面量: `@email | test(/.*@.*\.com/)`
-   空值处理操作符: `@nickname // @username`

### 3.4 技术选型

| 组件 | 选择 | 理由 |
| :--- | :--- | :--- |
| Parser Generator | `pest` | 成熟的 PEG 解析器，xan/tera 都在使用 |
| Parser Algorithm | Pratt Parser | 处理操作符优先级的标准方案 |
| String Storage | `Arc<str>` | 零拷贝共享，减少内存分配 |
| Error Handling | `thiserror` | 与现有代码库一致 |
| Regex | `regex` crate | Rust 生态标准，性能优秀 |

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
