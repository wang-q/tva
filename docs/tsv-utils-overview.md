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
  - `csv2tsv`：处理 CSV 各种边界情况（引号、编码等），解析逻辑比上面工具复杂一截。
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
- 第二阶段（待选）：逐步覆盖 `tsv-append`、`tsv-split`、`tsv-sample`、`tsv-uniq`、`tsv-select` 等中等复杂度工具。
- 最后阶段（视需要）：再评估 `tsv-filter`、`tsv-summarize`、`tsv-join` 等“重量级”工具是否需要完整对标，或只抽取其中一部分能力。

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
     - 去重（当前已有 `dedup`，可向 `tsv-uniq` 的等价类模式靠拢）
     - 汇总统计（类似 `tsv-summarize`）
     - 拼接与拆分（类似 `tsv-append` / `tsv-split`）

2. **字段语法与命名**
   - 统一的字段语法（编号、名称、通配符）可以成为 `tva` 的长期目标。
   - 文档中清晰说明字段选择规则，有利于用户迁移习惯（尤其是从 tsv-utils 过来的用户）。

3. **性能与实现策略**
   - tsv-utils 强调“单次扫描、尽量避免排序”的策略，这一点在 `tva::cmd_tva::dedup` 中已经开始实践。
   - 可以参照 `AboutTheCode.md` 与性能文档，思考在 Rust 中类似的优化：缓存策略、流式读取、避免不必要的分配等。

4. **文档与帮助体系**
   - 顶层 README 给出工具一览 + 链接到细节文档。
   - 每个工具有单独的参考文档与示例。
   - 命令行 `--help` / `--help-verbose` 与文档中的术语保持一致。
   - 这些模式与当前 `tva` 中正在建立的 README / help text style guide 非常契合。

5. **公共基础设施（`common/` 目录）**
     - **输入源抽象（`InputSourceRange` 等）**
       - 为“多文件 + stdin”提供统一的输入视图：按命令行顺序依次打开文件/标准输入，对上层暴露为一个连续的“行流”。
       - 对应到 `tva`，可以在现有 `libs::reader` 基础上增加一层“输入源迭代器”，统一处理：
         - `-` / `stdin` 等惯例
         - 文件不存在、权限错误等异常信息的格式
         - 多文件场景下的遍历顺序与行为（是否跨文件连续编号、是否保留 header 等）。
     - **字段列表解析（`parseFieldList` / 字段语法实现）**
       - 负责解析命令行中的字段列表：数字（如 `1,3-5`）、字段名（如 `color,count`）、通配符字段名（如 `*_time`）等。
       - 对 `tsv-select`、`tsv-uniq`、`tsv-join` 等工具都是“地基式”的能力。
       - `tva` 可以参考其语义，在 Rust 中实现统一的字段列表模块（支持相同的语法与错误处理），作为未来各子命令的公共依赖。
     - **数值工具（`quantile`、`formatNumber` 等）**
       - `quantile`：实现一套与 R 等统计软件兼容的分位数计算方法，用于 `tsv-summarize` 等统计类工具。
       - `formatNumber`：为数值输出提供一致的格式策略，避免出现难以阅读的浮点尾数。
       - 在 `tva` 中，这些可以演化为“统一的数值工具模块”：既定义分位数/统计函数的行为，也统一数字打印风格，便于不同命令之间保持一致。
     - **命令行解析辅助（`getoptInorder` 等）**
       - 上游用来封装 D 语言的 `std.getopt`，保证按命令行给出的顺序处理参数。
       - `tva` 已经使用 Rust 的 `clap` 进行解析，这部分不需要直接迁移代码，但可以借鉴其选项命名和默认值设计习惯，保持各子命令之间的 CLI 风格一致。
  - 从迁移优先级上看，可以粗略分为三批：
    - **第一批（基础设施）**：字段列表解析、输入源抽象——这两者对后续大多数子命令都有直接帮助，适合在更多工具落地前先设计好 Rust 版实现。
    - **第二批（统计相关）**：分位数与数值格式化工具——在 `tva` 开始实现 `tsv-summarize` 或类似统计命令时，可以按需对标迁移。
    - **第三批（按需参考）**：命令行解析封装、其他 IO 辅助——Rust 生态已有成熟方案，可主要参考设计思路而非迁移具体代码。

## tva 子命令实施进度

当前已经在 `tva` 中落地的基础设施和子命令及测试情况简要如下（以本仓库为准）：

### 基础设施模块

- 字段列表解析（对应上游 `parseFieldList`）：
  - 状态：已在 `libs::fields` 中实现数字区间语法（如 `1,3-5`），通过 `parse_numeric_field_list` / `fields_to_ints` / `fields_to_idx` 为 `dedup`、`md` 等子命令提供统一的数字字段列表解析。
  - 测试：在 `libs::fields` 模块内有单元测试，覆盖空输入、重复字段、空白处理等基本场景。
- 输入源抽象（对应上游 `InputSourceRange`）：
  - 状态：已在 `libs::io` 中实现统一输入层：`reader` 处理 `stdin` / `-` / 普通文件 / `.gz` 压缩文件，`InputSource` / `input_sources` 为多输入文件提供一致视图，`has_nonempty_line` 封装“探测是否包含非空行”的逻辑。
  - 使用情况：当前已被 `nl`、`dedup`、`keep-header`、`check` 等子命令复用，作为后续 `tva sort` 等命令的输入基础设施。
- 数值格式化（对应上游 `formatNumber`）：
  - 状态：在 `libs::number` 中实现了 `format_number`，用于统一数值输出格式，避免难以阅读的浮点尾数。
  - 测试：`libs::number` 模块内有单元测试，覆盖典型的整数和小数格式化场景。

### 子命令

- `md`：
  - 状态：已实现，提供区间到 Markdown 表格的转换能力；在“表格美观展示”这一点上与上游 `tsv-pretty` 功能有一定重叠，因此暂不计划单独迁移 `tsv-pretty`。
  - 测试：有专门的 CLI 测试（`tests/cli_tva.rs`），覆盖基本输出形态和数字格式化行为。
- `dedup`：
  - 状态：已实现，支持整行去重和按字段去重，基于 `libs::fields` 的字段列表解析。
  - 测试：在 `tests/cli_tva.rs` 中有多组用例，覆盖文件输入、stdin 输入以及“stdin+文件”混合场景。
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
