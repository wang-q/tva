# Other Tools Analysis

This document analyzes existing solutions for data processing expression languages, providing context for TVA's design decisions.

## GNU Parallel: Template Substitution

GNU Parallel uses a **string replacement** based placeholder mechanism, particularly suited for building Shell commands.

*   **Core Mechanism**: Predefined placeholders with string interpolation at execution time.
*   **Placeholder Syntax**:
    *   `{1}`, `{2}`: Column reference by index.
    *   `{-1}`: Reverse index reference.
    *   `{col_name}`: Header column name reference.
    *   `{= perl_expr =}`: Embedded Perl expression for immediate evaluation.
*   **Pros**:
    *   Intuitive, fits Shell scripting habits.
    *   Suitable for "command construction" scenarios.
*   **Cons**:
    *   Essentially text processing, lacks a type system.
    *   Difficult to handle complex logic (unless embedding Perl, which has performance costs).

## AWK: Record and Field Processing Model

AWK is a classic Unix text processing tool centered around **Records** and **Fields**.

*   **Core Mechanism**:
    *   **Record Splitting**: Input is split into records (default by line, controlled by `RS`). `NR` tracks current record number, `FNR` tracks current file's record number.
    *   **Field Splitting**: Each record is further split into fields (default by whitespace, controlled by `FS`). `NF` tracks field count.
    *   **Field Reference**: `$n` syntax references the nth field, `$0` represents the entire record. Field numbers can be variables or expressions (e.g., `$NF` for last field).

*   **Syntax Features**:
    *   **Pattern-Action**: `pattern { action }` structure executes when pattern matches.
    *   **Built-in Variables**: `NR`, `NF`, `FS`, `RS`, `OFS`, `ORS` control I/O behavior.
    *   **Implicit Type Conversion**: Strings and numbers convert automatically based on context.

*   **Pros**:
    *   **Concise and Efficient**: Optimized for text processing, complex tasks in single lines.
    *   **Streaming**: Naturally supports line-by-line processing with low memory usage.
    *   **Widely Available**: Pre-installed on almost all Unix-like systems.

*   **Cons**:
    *   **Ancient Syntax**: Differs significantly from modern languages, steep learning curve.
    *   **Weak Type System**: Implicit conversions convenient but error-prone.
    *   **Limited Features**: Standard AWK lacks modern features (JSON support, rich string functions).

## xan (Moonblade): Dynamic Expression Engine

xan includes **Moonblade**, a dynamic scripting language similar to simplified Python/JavaScript, optimized for CSV processing.

*   **Core Mechanism**:
    *   **Parser & Interpreter**: PEG parser (Pest) builds AST, uses Tree-Walker Interpreter.
    *   **Concretization**: Static analysis optimization before execution, e.g., resolving column names to indices to avoid runtime hash lookups.
    *   **Arc-based Types**: Uses `Arc<T>` for strings and complex objects to minimize deep copies, suitable for streaming.

*   **Syntax Features**:
    *   **Columns as Variables**: Use column names directly (e.g., `count > 10`), or `col("name")`/`col(index)`.
    *   **Operator Distinction**: Distinguishes string (`eq`, `++`) and numeric (`==`, `+`) operators (Perl-like design), avoiding implicit conversion ambiguity.
    *   **Rich Built-ins**: String, Math, Date, Regex, Hashing, plus CSS Selector-style HTML scraping.
    *   **Higher-Order Functions**: Supports `map`, `filter`, `reduce` with Lambda expressions (e.g., `map(list, x => x + 1)`).

*   **Pros**:
    *   **Extremely Expressive**: Handles complex business logic (JSON parsing, date calculations, fuzzy matching).
    *   **Relatively High Performance**: Optimized for CSV structure (Header/Row) compared to general scripting languages.

*   **Cons**:
    *   **High Implementation Complexity**: Maintaining a complete interpreter, type system, and function library is costly.
    *   **Runtime Overhead**: Despite optimizations, AST traversal and dynamic dispatch per row have some cost compared to pure Native code.

## jq: JSON Processor & Stream Computing

`jq` is the de facto standard for JSON processing, built on a specialized stack-based VM with backtracking and generators.

*   **Core Mechanism**:
    *   **Bytecode Compiler & VM**: Compiles queries (Filters) to bytecode executed in an interpreter loop. Unlike xan's Tree-walker, this provides a more compact execution model.
    *   **Backtracking**: jq's killer feature. Via `FORK` and `BACKTRACK` instructions, one input can produce zero, one, or multiple outputs (e.g., `.[]`). This makes jq essentially a streaming generator engine.
    *   **Reference Counting**: Data represented via `jv` struct (in `jv.h`), using reference counting. Allows efficient "shallow copies" (Copy-on-write), but has overhead with high-frequency allocation.

*   **Architecture**:
    *   **`jv` Type System**: Unified JSON value representation (Tagged Union): Null, Bool, Number, String, Array, Object.
    *   **Hybrid Stdlib**: Many standard library functions (`map`, `select`, `recurse`) written in jq language (`builtin.jq`), only low-level primitives (`type`, `keys`) in C. Easy to extend.
    *   **Stack Machine**: VM maintains value stack and frame stack, supports closures and complex control flow.

*   **Syntax Features**:
    *   **Filters**: Everything is a filter. Input -> Filter -> Output stream.
    *   **Pipe (`|`)**: Primary way to compose filters. `a | b` runs `b` with **each** output of `a` as input.
    *   **Context (`.`)**: Explicit reference to current context value.
    *   **Variables (`$var`)**: Bound via `... as $x | ...`, lexically scoped.

*   **Pros**:
    *   **Minimal Streaming Syntax**: Extremely efficient for nested JSON.
    *   **Powerful Composition**: Pipes and backtracking enable Cartesian products, permutations, etc.
    *   **Mature Ecosystem**: Pre-installed almost everywhere, no dependencies.

*   **Cons**:
    *   **Performance Bottleneck**: Compared to columnar engines (xan/polars), jq's per-object processing and interpreter overhead is significant.
    *   **Mental Model**: Backtracking is hard to understand for users from traditional programming (e.g., why `debug` prints multiple times).

## Tera: Template Engine & Rust Ecosystem

`Tera` is Rust's most popular template engine, inspired by Jinja2 and Django templates. While mainly for HTML generation, its design patterns are highly relevant for expression languages.

*   **Core Mechanism**:
    *   **Pest Parser**: Uses PEG (Parsing Expression Grammar) parser (`pest`) to define syntax (`tera.pest`) and generate AST (`ast.rs`). Provides extreme syntax flexibility.
    *   **Renderer & CallStack**: Tree-walker pattern. `CallStack` (`call_stack.rs`) manages `StackFrame` (`stack_frame.rs`) for scope, loops (`ForLoop`), and macro calls.
    *   **JSON-centric**: Data model built entirely on `serde_json::Value`. `Context` is essentially `BTreeMap<String, Value>`. Seamlessly handles any Rust data serializable to JSON.

*   **Architecture**:
    *   **Function/Filter/Test Traits**: Highly extensible. Implement `Function`, `Filter`, `Test` traits (`builtins/mod.rs`) to inject custom logic.
    *   **Inheritance**: Supports `extends` and `block` for template reuse and override, useful for complex document structures.
    *   **Whitespace Control**: `{%-` and `-%}` for precise whitespace control (`parser/whitespace.rs`), crucial for format-sensitive text (code or specific data formats).

*   **Syntax Features**:
    *   **Delimiters**: Variables `{{ ... }}`, tags `{% ... %}`, comments `{# ... #}`.
    *   **Filters**: Pipe style `value | filter(arg=1)`.
    *   **Control Flow**: Rich `if`, `for`, `include`, `macro` support.
    *   **Tests**: `is` operator, e.g., `if variable is defined`.

*   **Pros**:
    *   **Balance of Type Safety and Dynamism**: Rust-based but provides dynamic type experience (JSON Value).
    *   **Extremely Mature Syntax**: Jinja2 style has stood the test of time, low cognitive burden.
    *   **Error Handling**: Detailed error location and context (`errors.rs`).

*   **Cons**:
    *   **Allocation Heavy**: Rendering involves significant `String` allocation and `Value` cloning (Clone-on-write via `Cow`), potentially too costly for high-performance streaming data processing (like tva's goal).
    *   **Text-oriented**: Core goal is text concatenation, not data structure transformation.
