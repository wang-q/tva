# tsv-utils 项目概览

本文档基于本地目录 `tsv-utils-master`（eBay/tsv-utils）编写，作为在 `tva` 项目中参考使用的简要说明。

## 1. 项目定位

tsv-utils 是一组针对制表数据（尤其是 TSV：Tab Separated Values）的命令行工具，用于在命令行上对大数据量文本文件进行：
- 行过滤、采样
- 列选择、重排、连接
- 统计汇总
- 去重、切分、格式转换（CSV ↔ TSV）

它面向的典型场景是：
- 文件大到不适合全部载入 R / Pandas 等内存分析工具
- 但又小到不需要上 Hadoop / Spark 这类分布式系统

整体风格是“增强版 Unix 文本流工具”：类似 `cut`、`sort`、`grep`、`awk`，但对 TSV 做了专门优化，并且大量关注性能。

## 2. 工具一览

核心工具（均为独立可执行程序）包括：

- `tsv-filter`：按字段进行数值/字符串/正则过滤，可组合多条件；支持按字段名或字段号。
- `tsv-select`：列选择工具，类似 `cut`，支持按列名选择、重排、排除以及通配符。
- `tsv-uniq`：去重工具，支持按整行或部分字段定义“相等”；可输出等价类信息。
- `tsv-summarize`：对指定字段做汇总统计，可按 key 分组。
- `tsv-sample`：采样工具，支持多种抽样与乱序方式。
- `tsv-join`：按字段做文件间连接（join）。
- `tsv-pretty`：将 TSV 对齐美化，便于在命令行阅读。
- `csv2tsv`：CSV → TSV 转换工具，处理多种 CSV 变体。
- `tsv-split`：按行数、随机或 key 将数据拆分为多个文件。
- `tsv-append`：带头部感知的 TSV 拼接工具，可追踪来源文件。
- `number-lines`：给输入行编号。
- `keep-header`：在保留表头的前提下，对正文部分执行任意 shell 命令。

这些工具共享一套字段语法（field syntax），包括：
- 按字段号：`1,3-5` 等
- 按字段名：`color,count`
- 通配符字段名：`*_time` 等

详细字段语法在上游文档 `docs/tool_reference/common-options-and-behavior.md#field-syntax` 中有完整说明。

### 2.1 代码规模与复杂度（迁移参考）

下面是基于源码体量和逻辑复杂度，对各工具做的粗略分组，只作为后续在 `tva` 中迁移/对标时的优先级参考。

- **代码相对少、逻辑单一（适合作为第一批迁移目标）**
  - `number-lines`：核心就是按行输出带编号，参数和分支较少。
  - `keep-header`：头部感知的包装工具，主要是 IO 流和外部命令封装，控制流简单。
  - `tsv-pretty`：对 TSV 做对齐美化，虽然要计算列宽，但整体流程相对直观。

- **中等复杂度（功能较多但整体仍是线性流程）**
  - `csv2tsv`：处理 CSV 各种边界情况（引号、编码等），解析逻辑比上面工具复杂一截；在 `tva` 中对应的能力由 `from-csv` 子命令基于 Rust `csv` crate 提供，整体实现复杂度有所降低。
  - `tsv-append`：需要处理多文件头部、来源标记等，但数据流仍然是一次扫描。
  - `tsv-split` / `tsv-sample`：有多种拆分/抽样模式，需要一定的参数组合和随机数逻辑。
  - `tsv-uniq`：支持整行/部分字段去重以及等价类输出，内部状态管理比简单去重复杂。
  - `tsv-select`：字段选择、重排、排除、通配符等带来不少选项处理代码。

- **代码量大、逻辑复杂（建议作为后期迁移目标）**
  - `tsv-filter`：选项极多，内部为构建和执行一套“字段谓词”管线，代码行数和分支都明显高于其他工具。
  - `tsv-summarize`：实现汇总统计、分组、多种聚合函数，内部有较复杂的状态与数据结构。
  - `tsv-join`：多文件 join、键匹配、连接模式处理等，控制流和边界情况较多。

在 `tva` 中如果考虑分阶段迁移/重写，可以按目前进展和后续计划理解为：
- 第一阶段（已完成部分）：从 `number-lines`、`keep-header` 这类“薄壳工具”入手，分别在 `tva` 中落地为 `nl`、`keep-header` 子命令，顺带打磨统一的 IO/字段处理基础设施；对于 `tsv-pretty`，当前由 `tva md` 在“表格美观展示”场景上部分接替其作用，暂不计划做 1:1 迁移。
- 第二阶段（进行中）：逐步覆盖 `tsv-append`、`tsv-split`、`tsv-sample`、`tsv-uniq`、`tsv-select` 等中等复杂度工具；其中 `tsv-append` / `tsv-split` / `tsv-sample` / `tsv-select` 已分别在 `tva` 中落地为 `append` / `split` / `sample` / `select` 子命令，并配套 CLI/golden 测试；`tsv-uniq` 已在 `tva` 中落地为 `uniq` 子命令，已迁移一批上游 golden 测试并补充 tva 风格的错误处理测试；`tsv-join` 的基础 inner join 能力已在 `tva` 中以 `join` 子命令形式落地，当前覆盖单 key、header 感知和按列名/列号指定 join key 及追加字段等核心行为。
- 最后阶段（已启动规划）：在现有 `join` 子命令的基础上，逐步评估并补齐 `tsv-join` 的其他 join 模式（如 left outer join/anti-join、`--write-all` 等），并在合适时机评估 `tsv-filter`、`tsv-summarize` 等“重量级”工具是否需要完整对标，或只抽取其中一部分能力。

### 2.2 上游测试资源与迁移策略

`tsv-utils-master` 中每个工具目录（例如 `number-lines/`、`tsv-uniq/`、`tsv-summarize/` 等）通常都包含：

- 源码：`src/tsv_utils/*.d`
- 测试脚本和测试数据：`tests/` 目录，包含输入文件、预期输出、shell 脚本等

这些资源对 `tva` 中所有迁移对象的意义主要有三点：

1. **定义各工具的“行为边界”**
   - 单输入 / 多输入、是否连续编号（如 `number-lines`）、是否保持输入顺序（如 `tsv-uniq`）
   - 各种选项组合下的精确语义（例如 header 行、缺失值、重复 key、不同 join 模式）
   - 小文件、极端值、大文件等不同规模下的行为

2. **提供可复用的测试数据**
   - 许多用例本质上就是“输入文本 + 期望输出文本”的组合，非常适合迁移到 Rust 的 CLI 测试里做 golden test：
     - 把上游的输入文件复制到 `tva` 的 `tests/data/<tool>/`（或类似目录）
     - 运行 `tva <subcommand> ...`，比对 stdout 是否与上游期望输出严格一致
   - 对 `tva` 来说，可以按工具进行拆解：
     - **单功能用例**：只覆盖一个选项，例如单独测试 `--header`、单独测试某个聚合函数
     - **组合用例**：多选项叠加（如过滤 + 排序 + 汇总、多文件 join 模式组合）用于回归测试

3. **帮助保持与上游行为的一致性**
   - 当我们在 `tva` 中重写 `tsv-utils` 的某个工具时，往往会在 API 设计、选项命名上做适度调整，但核心行为（例如去重逻辑、汇总语义、字段选择规则）通常希望保持一致或有明确差异说明。
   - 通过迁移上游测试，可以把这些行为固化到自动化测试中，避免：
     - 在复杂选项组合下出现意外的行为偏差
     - 忘记处理某些边界情况（空输入、单行文件、只有 header 的文件等）
     - 新增功能时无意破坏已有语义

一个适用于所有迁移对象的通用流程可以是：

- **挑选代表性测试集**：对每个计划迁移的工具，从上游测试中挑选一小批“覆盖主要功能”的用例，而不是完整照搬全部脚本。
- **按 tva 目录结构整理测试数据**：为每个子命令建立独立的数据目录（如 `tests/data/nl/`、`tests/data/uniq/`），并在文件名中编码用例意图。
- **为每个子命令建立独立的 CLI 测试文件**：例如 `tests/cli_nl.rs`、`tests/cli_uniq.rs`，与 `tests/cli_tva.rs` 区分开来。
- **用 Rust CLI 测试重写断言**：不直接执行 D 语言或 shell 测试脚本，而是在 Rust 侧使用 `assert_cmd` 等工具：
  - 运行 `tva <subcommand> ...`
  - 将 stdout 与预期输出文件逐行比对
- **记录有意差异**：如果 `tva` 在某些行为上有意偏离上游（例如默认选项、错误信息、编码处理），在测试和文档中明确标注“与 tsv-utils 不同”的部分，避免未来维护者误以为是 bug。

`number-lines` / `keep-header` 已经可以作为这个通用流程的试点：先用它们验证“如何迁移一个工具的测试数据和行为定义”，再按同样的模式扩展到 `tsv-uniq`、`tsv-summarize` 等其他工具（而在“对齐美观展示”方面则由 `tva md` 承担与 `tsv-pretty` 部分重叠的需求）。

### 2.3 工具风格与共性

从整体风格上看，`tsv-utils` 里的各个工具有一套非常统一的“类 Unix 文本流工具”约定，主要体现在：

- **流式、无状态、可管道**：所有工具都优先从 stdin 读、向 stdout 写，不做交互式输入，也尽量避免把整个文件读入内存；这使得它们非常适合用在长管道和大文件场景下。
- **TSV-first 的数据模型**：默认分隔符是 TAB，所有行为围绕“行 + 字段”展开；CSV 只是入口格式（`csv2tsv`），真正的处理逻辑基本都发生在 TSV 层。
- **字段/表头感知**：绝大多数表格工具会显式区分 header 行与数据行，并围绕“列名/列号 + 统一字段语法”来设计选项；像 `tsv-append`、`keep-header` 这类工具更是直接把“如何处理 header”作为核心能力。
- **CLI 选项统一且偏“显式”**：短选项/长选项成对存在（如 `-H`/`--header`、`-f`/`--fields`），默认行为尽量保守，重要行为都要求用户通过选项显式声明，避免“魔法”。
- **错误处理偏严格、fail-fast**：一旦检测到结构性问题（字段数不一致、非法 CSV、header 不匹配等），工具通常会立即在 stderr 打出含文件名/行号的详细错误信息，并以非零退出码终止，而不是“尽量吞掉错误继续跑”。
- **行为由测试定义**：每个工具都配有较完整的测试脚本和 golden output，用来约束边界行为（如 header、多文件、空输入、极端数值等）；相比 README 文字描述，测试往往更完整地体现“工具到底该怎么工作”。

在 `tva` 中，对标 `tsv-utils` 时可以把这些风格理解为“设计约束”而不是“实现细节”，例如：

- 新子命令默认走 stdin/stdout 流式模型，避免一次性载入整表；
- 所有表格类子命令都统一支持 `--header` / `--no-header` / 字段语法，复用同一套 header/字段抽象；
- 对结构性错误保持严格、可预测的错误信息和退出码，避免静默截断或自动纠错；
- 新增/重写能力时优先从“能否用 golden 测试描述行为”出发设计 CLI，而不是先写代码再补测试。

## 3. 使用模式与设计特点

### 3.1 流式处理

- 所有工具都可以从标准输入读取数据并写到标准输出，适合搭建 Unix pipeline。
- 默认分隔符是 TAB，也可以配置为其他字符。
- 输入输出默认使用 UTF-8，且在正则过滤等场景下是 Unicode 友好的。

### 3.2 性能导向

从 README 与 `docs/Performance.md` 可见，tsv-utils 强调：
- 针对大文件进行单次线性扫描
- 减少不必要的排序（例如 `tsv-uniq` 在不排序的情况下做去重）
- 利用 D 语言的性能特性和 LTO/PGO 等编译优化

这使得它在许多基准测试中明显快于其他常见工具（包括部分脚本语言实现），同时覆盖 TSV 数据处理中常见的操作类型（过滤、选择、去重、统计、拆分、连接等），并配套完整文档与统一的 CLI 体验。

### 3.3 统一的 CLI 体验

上游为每个工具提供：
- `--help`：常规帮助
- `--help-verbose`：更接近参考手册的长文档

并配套有：
- Bash 补全脚本（`bash_completion/tsv-utils`）
- 完整的 Tool Reference 文档（`docs/ToolReference.md` 与 `docs/tool_reference/*.md`）

这一点在设计 `tva` 自己的 CLI 体验时很值得借鉴：统一选项命名、错误信息和帮助风格。

## 4. 目录结构简述

顶层关键内容：

- `README.md`：总说明，包含工具列表与详细介绍链接。
- `docs/`：文档目录，包括：
  - 性能与对比：`Performance.md`、`Comparing TSV and CSV formats` 等
  - 构建相关：`BuildCommands.md`、`BuildingWithLTO.md`、`lto-pgo-study.md`
  - 代码说明：`AboutTheCode.md`
  - 工具参考：`ToolReference.md`、`tool_reference/*.md`
- 各工具子目录（例如 `tsv-uniq/`、`tsv-append/`、`tsv-split/`、`tsv-summarize/`、`csv2tsv/` 等）：
  - `src/tsv_utils/*.d`：具体工具实现（D 语言）
  - `README.md`：该工具的简要说明与示例
  - `tests/`：测试脚本与测试数据
  - `dub.json`、`makefile`：构建配置
- `extras/scripts/`：如 `tsv-sort` / `tsv-sort-fast` 等辅助脚本。
- CI 与质量相关文件：`.github/workflows/build-test.yml`、`.codecov.yml` 等。

## 5. 对 tva 项目的可借鉴点

结合当前 `tva` 的定位（Rust 实现的 TSV 助手），可以把 tsv-utils 视为一个“功能地图”和“经验库”：一方面按需对标其工具集合规划 `tva` 的子命令，另一方面在实现与文档风格上保持一致或有意识地作出改进；同时，上游 `common/` 目录提供的字段语法、输入源抽象和数值工具等，也可以视为未来在 Rust 中设计公共库的重要灵感来源。下面是几条可以直接借鉴的设计思路：

1. **子命令规划**
  - 可以参考 tsv-utils 的工具矩阵，逐步扩展 `tva` 的子命令：
    - 过滤（类似 `tsv-filter`）
    - 列选择（类似 `tsv-select`）
    - 去重（当前已有 `uniq` 子命令，对标 `tsv-uniq` 的等价类模式）
    - 汇总统计（类似 `tsv-summarize`）
    - 拼接与拆分（类似 `tsv-append` / `tsv-split`）

2. **字段语法与命名**
   - 统一的字段语法（编号、名称、通配符）可以成为 `tva` 的长期目标。
   - 文档中清晰说明字段选择规则，有利于用户迁移习惯（尤其是从 tsv-utils 过来的用户）。

3. **性能与实现策略**
   - tsv-utils 强调“单次扫描、尽量避免排序”的策略，这一点在 `tva::cmd_tva::uniq` 中已经开始实践。
   - 可以参照 `AboutTheCode.md` 与性能文档，思考在 Rust 中类似的优化：缓存策略、流式读取、避免不必要的分配等。

4. **文档与帮助体系**
   - 顶层 README 给出工具一览 + 链接到细节文档。
   - 每个工具有单独的参考文档与示例。
   - 命令行 `--help` / `--help-verbose` 与文档中的术语保持一致。
   - 这些模式与当前 `tva` 中正在建立的 README / help text style guide 非常契合。

5. **公共基础设施（`common/` 目录）**
   - 输入源抽象（`InputSourceRange` 等）：
     - 为“多文件 + stdin”提供统一的输入视图：按命令行顺序依次打开文件/标准输入，对上层暴露为一个连续的“行流”。
     - 对应到 `tva`，可以在现有 `libs::io::input_sources` 基础上继续演化，统一处理：
       - `-` / `stdin` 等惯例；
       - 文件不存在、权限错误等异常信息的格式；
       - 多文件场景下的遍历顺序与行为（是否跨文件连续编号、是否保留 header 等）。
   - 字段列表解析（`parseFieldList` / 字段语法实现）：
     - 负责解析命令行中的字段列表：数字（如 `1,3-5`）、字段名（如 `color,count`）、通配符字段名（如 `*_time`）等。
     - 对 `tsv-select`、`tsv-uniq`、`tsv-join` 等工具都是“地基式”的能力。
     - `tva` 已在 `libs::fields` 中实现数字字段列表解析，后续可以扩展到“列名 + 通配符”模式，逐步对齐上游语义。
   - 数值工具（`quantile`、`formatNumber` 等）：
     - `quantile`：实现一套与 R 等统计软件兼容的分位数计算方法，用于 `tsv-summarize` 等统计类工具。
     - `formatNumber`：为数值输出提供一致的格式策略，避免出现难以阅读的浮点尾数。
     - 在 `tva` 中，这些可以演化为“统一的数值工具模块”：既定义分位数/统计函数的行为，也统一数字打印风格，便于不同命令之间保持一致。
   - 命令行解析辅助（`getoptInorder` 等）：
     - 上游用来封装 D 语言的 `std.getopt`，保证按命令行给出的顺序处理参数。
     - `tva` 已经使用 Rust 的 `clap` 进行解析，这部分不需要直接迁移代码，但可以借鉴其选项命名和默认值设计习惯，保持各子命令之间的 CLI 风格一致。

6. **统一的 header 处理界面（tva 草案）**
   - 设计目标：
     - 所有“按行/列处理 TSV”的子命令在 header 上行为一致，可预期；
     - 便于实现“按列名操作”的命令（select/join/append/summarize 等）；
     - 多输入时，对 header 的合并与校验有统一规则。
  - CLI 约定（对表格类子命令通用）：
     - `--header` / `-H`：输入有 header 行，第一行是列名；未显式指定时，默认视为“无 header”，所有行都是数据，只允许用列号（1-based），不允许用列名。
   - 多输入与输出规则：
     - `--header`：多输入时以第一个输入的 header 作为“全局 header”，默认只在输出开头打印一次；后续输入的 header 行不再重复输出；
     - 未指定 `--header` 时：多输入不做 header 识别与合并，所有行视为数据行按顺序拼接；
     - 结构性错误（字段数不一致、header 缺失等）统一采用 fail-fast 策略，在 stderr 打出含文件名/行号的错误信息并非零退出。
   - 内部抽象（拟在 `libs` 中提供）：
     - `Header`：封装一行 header 信息，例如 `fields: Vec<String>` 与 `index_by_name: HashMap<String, usize>`，供列名解析使用；
     - `TableInput`：在现有 `InputSource` / `input_sources` 基础上增加可选 `header`，为多输入命令提供统一的“输入源 + header + 数据行”视图；
     - 字段解析：提供统一的“字段列表解析入口”，在显式给出 `--header` 时支持“列号 + 列名/通配符”，在默认的“无 header”模式下仅允许列号，并在用户误用列名时给出明确错误提示。
   - 渐进落地策略：
     - 优先在新设计的 TSV 子命令上采用上述约定（例如未来的 `tsv-select` / `tsv-append`），并通过 golden 测试固化行为；
     - 再视需要将现有命令（如 `check`、`keep-header` 等）逐步迁移到统一的 header 抽象之上。

## tva 子命令实施进度

当前已经在 `tva` 中落地的基础设施和子命令及测试情况简要如下（以本仓库为准）：

### 基础设施模块

- 字段列表解析（对应上游 `parseFieldList`）：
  - 状态：在 `libs::fields` 中实现了统一的字段语法解析：
    - 数字字段语法：`parse_numeric_field_list` / `fields_to_ints` / `fields_to_idx` 支持 `1,3-5` 等区间表达式，已被 `uniq`、`md` 等子命令复用。
    - header 感知语法：`parse_field_list_with_header` / `parse_field_list_with_header_preserve_order` 支持按列名、通配符（如 `*_time`）、列名区间（如 `run-user_time`）以及带反斜杠转义的特殊字段名（空格、`:`、`-`、`*`、`\001` 等），作为 `tva select` 的字段解析地基。
  - 测试：在 `libs::fields` 模块内有单元测试，覆盖空输入、重复字段、空白处理、header 模式下的列名/通配符/列名区间以及字段名转义等场景。
- 输入源抽象（对应上游 `InputSourceRange`）：
  - 状态：已在 `libs::io` 中实现统一输入层：`reader` 处理 `stdin` / `-` / 普通文件 / `.gz` 压缩文件，`InputSource` / `input_sources` 为多输入文件提供一致视图，`has_nonempty_line` 封装“探测是否包含非空行”的逻辑。
  - 使用情况：当前已被 `nl`、`uniq`、`keep-header`、`check` 等子命令复用，作为后续 `tva sort` 等命令的输入基础设施。
- 数值格式化（对应上游 `formatNumber`）：
  - 状态：在 `libs::number` 中实现了 `format_number`，用于统一数值输出格式，避免难以阅读的浮点尾数。
  - 测试：`libs::number` 模块内有单元测试，覆盖典型的整数和小数格式化场景。

### 子命令

- `md`：
  - 状态：已实现，提供区间到 Markdown 表格的转换能力；在“表格美观展示”这一点上与上游 `tsv-pretty` 功能有一定重叠，因此暂不计划单独迁移 `tsv-pretty`。
  - 测试：有专门的 CLI 测试（`tests/cli_tva.rs`），覆盖基本输出形态和数字格式化行为。
- `append`：
  - 状态：已实现，作为 `tsv-append` 的 Rust 版本子命令，用于在 header 感知的前提下拼接多个 TSV 文件，并可通过来源列追踪每行原始文件；支持 `--header/-H`、`--track-source/-t`、`--source-header/-s`、`--file/-f`、`--delimiter/-d` 等核心选项。
  - 实现：复用 `libs::io::reader` 逐个文件流式读取，在 `--header` 模式下以首个输入提供全局表头，并在输出中增加“来源列”（默认列名为 `file`，或由 `--source-header` 指定）；`--line-buffered` 作为兼容性选项被接受但当前实现为 no-op。
  - 测试：`tests/cli_append.rs` 基于上游 `tsv-append/tests/tests.sh` 与 `gold/basic_tests_1.txt` 迁移了多组 golden 用例（包括多文件拼接、header 合并、`--track-source` / `--source-header` / `--file` / `--line-buffered` 等组合），测试数据集中在 `tests/data/append/`，用于保证 `tva append` 的行为与上游一致。
- `uniq`：
  - 状态：已实现，作为 `tsv-uniq` 的 Rust 版本子命令，支持整行去重和按字段去重（包括使用 `-f 0` 将“整行”作为 key）、等价类输出和忽略大小写等行为，在核心语义上对标上游；当前已迁移一部分上游 golden 测试（包括 `input1.tsv`、若干字段组合用例以及 `-f 0` 场景），并在选项组合约束和字段语法错误上采用 `tva uniq: ...` 风格的 fail-fast 错误输出（与上游在“错误文案”上存在有意差异）。
  - 测试：在 `tests/cli_uniq.rs` 中有多组用例，既包含基于基因组示例数据的简单场景，也包含从 `tsv-uniq/tests/gold_basic_tests_1.txt` 迁移的 golden 输出比对用例；此外还通过纯 Rust 断言覆盖了 `-f 0`、`--equiv-start` / `--equiv-header` / `--number-header` 与主选项的组合约束，以及字段列表中包含 `0`、在非 header 模式下使用列名等错误场景；测试数据集中在 `tests/data/uniq/`。
- `nl`：
  - 状态：已实现，作为 `number-lines` 的 Rust 版本，支持多文件、header 处理、自定义分隔符和起始行号。
  - 测试：有独立的 golden 测试文件 `tests/cli_nl.rs`，基于上游 `number-lines` 的 `gold_basic_tests_1.txt` 迁移了完整行为块，并额外增加错误场景和版本/帮助输出等用例。
- `keep-header`：
  - 状态：已实现，负责在保留表头的前提下调用外部命令，对正文部分进行处理。
  - 测试：在 `tests/cli_keep_header.rs` 中有 CLI 测试，覆盖单文件、多文件排序等典型使用方式。
- `check`：
  - 状态：已实现，参考 GNU `datamash check`，作为“前置验证”子命令，用于校验输入表在所有行上是否具有统一的字段数；一旦发现字段数不一致或空行等结构问题，会在 stderr 输出问题行号、该行字段数和原始文本，并以非零退出码 fail-fast。
  - 实现：复用 `libs::io::reader` / `input_sources`，统一处理 `stdin` / `-` / 普通文件 / `.gz` 压缩文件，错误信息风格对齐 datamash，适合在 `sort`、`tsv-select`、`tsv-uniq` 等命令前作为结构健康检查。
  - 测试：`tests/cli_check.rs` 中包含若干用例，覆盖空输入、规则矩阵、字段数不一致以及空行（0 字段）等核心结构检查场景。
 - `transpose`：
   - 状态：已实现，参考 GNU `datamash transpose` 和 `qsv transpose`，作为“整表重排”子命令，用于对严格矩阵执行行列互换；以第一行字段数作为矩阵宽度，所有后续行必须具有相同的字段数，一旦发现不一致会输出问题行号、该行字段数和原始文本，并以非零退出码 fail-fast。
   - 实现：复用 `libs::io::reader` 读入整个输入表，以 `Vec<Vec<String>>` 构建 MxN 矩阵后按照列优先遍历输出 NxM 矩阵；当前版本仅支持 TAB 分隔和严格模式，不提供 `--no-strict`/`--filler` 或列选择等扩展选项。
   - 测试：`tests/cli_transpose.rs` 中包含若干用例，覆盖简单矩阵、空输入、非矩形结构报错以及单行/单列/单字段等退化矩阵场景。
 - `sort`：
   - 状态：已实现，作为“tsv-aware 排序工具”，支持针对 TSV/CSV 的按列排序；当前版本默认以 TAB 分隔，可通过 `-t` 切换分隔符。
   - 实现：复用 `libs::io::input_sources` 读取多输入源，将每一行解析为字段向量，并以内存内排序完成小文件的单 key / 多 key 排序；支持字典序与数值模式（`-n`）、升序/降序（`-r`）以及 GNU `sort` 风格的 `-k` 字段列表语法，字段索引与 `libs::fields` 保持一致。
   - 测试：`tests/cli_sort.rs` 中包含若干用例，覆盖默认词典序、多 key 排序、数值模式及逆序、分隔符切换以及数字/非数字混合场景。
 - `from-csv`：
   - 状态：已实现，作为 CSV 数据的入口子命令，将 CSV 解析为标准 TSV 以便接入 `tva` 其他子命令；在功能上对标上游的 `csv2tsv`，但解析细节交由 Rust `csv` crate 处理。
   - 实现：复用 `libs::io::reader` 读取 `stdin` / `-` / 普通文件 / `.gz` 压缩文件，通过 `csv::ReaderBuilder` 处理引号、分隔符和嵌套字段，将每条记录按 TAB 连接后输出。
   - 测试：`tests/cli_from_csv.rs` 中包含若干用例，覆盖基础转换、带引号和逗号的字段以及自定义分隔符场景。
- `select`：
  - 状态：已实现，作为 `tsv-select` 的 Rust 版本子命令，定位为“TSV-first”的列选择/重排工具，对标上游核心行为（按列号/列名/通配符选择、重排、排除），并兼顾 `cut` / `qsv select` 等常见工具的使用习惯。
    - 支持按列号选择/重排（含区间表达式）以及按列号排除字段；
    - 在显式指定 `--header` / `-H` 时，支持“列号 + 列名 + 通配符”的字段语法，列号与列名可混合使用；
    - header 模式下支持字段名通配符（如 `*_time`）、列名区间（如 `run-user_time`），以及通过反斜杠转义的特殊字段名（空格、冒号、连字符、逗号、星号和数字型列名等）。
    - 当前版本刻意不实现 `--rest` / `-r` 与 `--line-buffered` 等选项，只覆盖核心列选择/排除能力。
  - 实现：复用 `libs::io::input_sources` 作为输入层，默认按 TAB 解析 TSV；在 header 模式下以首个输入的表头作为全局 header，字段解析完全委托给扩展后的 `libs::fields`，在原有数字区间语法基础上增加“列名/通配符”分支，并在错误路径上采用 fail-fast 策略（例如未知字段名、非法字段列表表达式、header 与数据列数不一致等）。
  - 测试：`tests/cli_select.rs` 中包含 20+ 条 CLI 用例，测试数据来自上游 `tsv-select/tests` 中的 `input1.tsv`、`input_header*.tsv`、`input_header_variants.tsv` 等文件，并按既有迁移流程组织为独立测试目录与 CLI 测试文件，覆盖：
    - 按列号/区间的选择与排除、多文件与空文件场景；
    - header 模式下的列名选择、通配符、列名区间和字段名转义；
    - 对部分边界行为（如超大字段号、CRLF 行结束）与错误信息的验证。
- `sample`：
  - 状态：已实现，作为 `tsv-sample` 的 Rust 版本子命令，定位为“采样 / 乱序工具”，对整行进行随机重排或子集抽样，支持多种模式：
    - 默认乱序：不带额外选项时，对所有输入数据行做完全随机重排（所有排列等概率），相当于行级 shuffle。
    - 固定样本量随机采样（`--num/-n N`）：从全部输入中随机选 N 行输出，默认输出顺序也是随机的；配合 `--inorder/-i` 可以保持原始输入顺序。
    - Bernoulli 采样（`--prob/-p P`）：对输入流做逐行判定，每行以概率 P（0.0 < P ≤ 1.0）被保留或丢弃，行顺序保持不变，是完全流式算法。
    - 加权随机采样（`--num/-n N --weight-field/-w F`）：按某一字段的权重做加权采样，内部使用 Efraimidis & Spirakis 流式加权算法，不指定 `--num` 时等价于“按权重打乱所有行”。
    - 有放回采样（`--replace/-r --num/-n N`）：读入全部行后进行 N 次独立随机选择，每次选择一行输出，允许同一行被多次选中。
    - Distinct 采样（`--key-fields/-k F --prob/-p P`）：先在 key 空间上做 Bernoulli 抽样（按 key 值决定是否选中），所有 key 被选中的行都会输出，适合“按用户、会话等主键抽样”。
  - 字段与随机控制：
    - 字段指定：采样时用于 key / weight 的字段通过 `libs::fields` 统一字段语法传入（数字或列名），是否有 header 由 `--header/-H` 控制。
    - 随机种子：默认每次运行结果不同；`--static-seed/-s` 固定种子；`--seed-value/-v` 可指定具体非零 64 位种子，便于可重现实验。
    - 随机值输出：`--print-random` 与 `--gen-random-inorder` 支持将内部使用的随机值作为新字段打印出来（例如调试或后续分析），字段名可通过 `--random-value-header` 配置。
    - 兼容性模式：`--compatibility-mode` 会强制使用“每行一个随机值”的算法族，以保证不同采样参数之间的可组合性（例如同一 static seed 下，`--num 5` 的结果必然包含于 `--num 10` 的结果，多文件 / stdin 混合场景也保持稳定）。
  - 实现：复用 `libs::io::input_sources` 统一处理多文件和 stdin 输入，在 header 语义和字段语法上对齐 `select` / `dedup`；在内部根据是否有权重、是否有 key、是否需要兼容模式选择不同的随机算法（Fisher–Yates 洗牌、reservoir sampling、Efraimidis & Spirakis 加权采样、per-row random key 排序等），并通过 CLI 测试固化多文件、stdin+文件混合、CRLF 输入和各种选项组合下的行为。
  - 测试：`tests/cli_sample.rs` 中包含多组用例，部分测试数据来自上游 `tsv-sample/tests`（迁移至 `tests/data/sample`），覆盖：
    - shuffle、固定样本量和 Bernoulli 采样的基础行为；
    - `--replace`、`--weight-field`、`--key-fields`、`--compatibility-mode`、`--print-random`、`--gen-random-inorder` 等选项及其合法/非法组合；
    - 多文件与 stdin 混合输入、`--header` 下的 header 合并策略以及 Windows 风格行结束符（`.dos_tsv`）等边界场景。
- `split`：
  - 状态：已实现，作为 `tsv-split` 的 Rust 版本子命令，支持按行数拆分（`--lines-per-file/-l`）、纯随机拆分（`--num-files/-n`）以及按 key 随机拆分（`--num-files/-n` 配合 `--key-fields/-k`），并在 `--header-in-out/-H` 模式下将首行 header 写入所有输出文件。
  - 实现：复用 `libs::io::input_sources` 逐行流式读取输入；在行数模式下按固定数据行数轮换输出文件；在随机模式下通过 `RapidRng` 或基于 `rapidhash` 的 key 映射将每行分配到 `N` 个输出桶之一，输出文件名由 `--dir`、`--prefix`、`--suffix` 和 `--digit-width` 共同决定，默认拒绝覆盖已有文件，可通过 `--append/-a` 追加写入。
  - 测试：`tests/cli_split.rs` 中包含多组用例，覆盖按行数拆分、header 复制、多文件随机拆分在 `--static-seed` 下的可重复性，以及按 key 拆分时“同一 key 必然落在同一输出文件”的核心语义。
