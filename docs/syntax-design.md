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
    *   适合“构造命令”的场景。
*   **缺点**:
    *   本质是文本处理，缺乏类型系统（Type System）。
    *   难以处理复杂的逻辑运算（除非嵌入 Perl，但会有性能损耗）。

### 1.2 xan (Moonblade): 动态表达式引擎 (Dynamic Expression Engine)

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
    *   **Reference Counting (引用计数)**: 数据通过 `jv` 结构体表示（定义在 `jv.h`），使用引用计数管理内存。这允许高效的“浅拷贝”（Copy-on-write），但在高频分配下会有一定开销。

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
8.  `eq`, `ne`, `lt`, `le`, `gt`, `ge` - 字符串比较，如 `@name eq "Alice"`
9.  `and`, `or`, `not` - 逻辑与或非，如 `@age > 18 and @age < 65`
10. `|` - 管道（函数链，最低优先级），如 `@name | trim(_) | upper(_)`

**注意**：
- 无隐式类型转换，字符串比较需显式使用函数
- 管道操作符 `|` 作为分隔符，将左侧结果传递给右侧函数
- 使用 `_` 在函数中引用管道左侧结果（如 `@name | trim(_) | upper(_)`）

#### 2.2.4 函数调用 (Function Call)

-   **前缀调用**: `func(arg1, arg2, ...)`，如 `trim(@name)`, `substr(@desc, 0, 50)`
-   **管道调用**: `arg | func(_)`，如 `@name | trim(_) | upper(_)`
-   **管道（多参数）**: `arg | func(_, arg2)`，如 `@name | split(_, " ")`

#### 2.2.5 完整示例

```bash
# 过滤：价格大于100且库存大于0
@price > 100 && @stock > 0

# 计算：总价（含税）
@price * (1 + @tax_rate)

# 字符串处理：格式化姓名
@first_name | trim(_) | upper(_) ++ " " ++ @last_name | trim(_) | upper(_)

# 条件：根据年龄分组
if(@age < 18, "minor", if(@age < 60, "adult", "senior"))

# 正则匹配：验证邮箱格式
regex_match(@email, "^[\\w.-]+@[\\w.-]+\\.\\w+$")

# 默认值：处理可能为空的字段
default(@nickname, @username)
```

### 2.3 函数库提案 (Standard Library Proposal)

#### 1. 字符串处理 (String Manipulation)

| 函数 | 签名 | 说明 | 零拷贝? | 参考 |
| :--- | :--- | :--- | :--- | :--- |
| `trim` | `trim(str) -> str` | 去除首尾空白 | ✅ | Tera/xan/jq |
| `split` | `split(str, pat) -> list<str>` | 按分隔符分割 | ✅ (Iterator) | xan/jq |
| `substr` | `substr(str, start, len?) -> str` | 获取子串 | ✅ | SQL/Go |
| `upper` | `upper(str) -> str` | 转大写 | ❌ (Alloc) | Tera/xan |
| `lower` | `lower(str) -> str` | 转小写 | ❌ (Alloc) | Tera/xan |
| `replace` | `replace(str, from, to) -> str` | 字符串替换 | ❌ (Alloc) | Tera/xan |
| `len` | `len(str) -> int` | 字节长度 | - | Tera/jq |
| `char_len` | `char_len(str) -> int` | 字符数 (UTF-8) | - | - |

#### 2. 数值与转换 (Numeric & Conversion)

| 函数 | 签名 | 说明 | 参考 |
| :--- | :--- | :--- | :--- |
| `int` | `int(val, default?) -> int` | 解析为整数，失败可选返回默认值 | Python/Tera |
| `float` | `float(val, default?) -> float` | 解析为浮点数 | Python/Tera |
| `abs` | `abs(num) -> num` | 绝对值 | Tera/xan |
| `round` | `round(num, precision?) -> float` | 四舍五入 | Tera/xan |
| `min` | `min(a, b, ...)` | 最小值 | xan |
| `max` | `max(a, b, ...)` | 最大值 | xan |

#### 3. 逻辑与控制 (Logic & Control)

| 函数 | 签名 | 说明 | 参考 |
| :--- | :--- | :--- | :--- |
| `default` | `default(val, fallback)` | 空值回退 (Coalesce) | Tera (`| default`) / jq (`//`) |
| `if` | `if(cond, then, else)` | 三元表达式函数版 | Excel/SQL |
| `reduce` | `reduce(list, init, (acc, x) -> ...)` | 归约/折叠 (模拟循环) | xan/jq |

#### 4. 正则表达式 (Regex)

*注：正则操作通常开销较大，应谨慎使用。*

| 函数 | 签名 | 说明 | 参考 |
| :--- | :--- | :--- | :--- |
| `regex_match` | `regex_match(str, regex) -> bool` | 是否匹配 | xan/jq (`test`) |
| `regex_extract` | `regex_extract(str, regex, group?) -> str` | 提取捕获组 | xan (`capture`) |
| `regex_replace` | `regex_replace(str, regex, to) -> str` | 正则替换 | xan/jq |

#### 5. 编码与哈希 (Encoding & Hashing)

| 函数 | 签名 | 说明 | 参考 |
| :--- | :--- | :--- | :--- |
| `md5` | `md5(str) -> str` | MD5 哈希 | xan |
| `sha256` | `sha256(str) -> str` | SHA256 哈希 | xan |
| `base64` | `base64(str) -> str` | Base64 编码 | xan/jq |
| `unbase64` | `unbase64(str) -> str` | Base64 解码 | xan/jq |

#### 6. 日期时间 (Date & Time)

| 函数 | 签名 | 说明 | 参考 |
| :--- | :--- | :--- | :--- |
| `now` | `now() -> date` | 当前时间 | Tera |
| `strptime` | `strptime(str, fmt) -> date` | 字符串转时间 | Python/jq |
| `strftime` | `strftime(date, fmt) -> str` | 时间转字符串 | Python/jq |

