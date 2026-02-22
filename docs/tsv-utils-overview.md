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

在 `tva` 中如果考虑分阶段迁移/重写，建议：
- 第一阶段：从 `number-lines`、`keep-header`、`tsv-pretty` 这类“薄壳工具”入手，打磨公共 IO/字段处理基础设施。
- 第二阶段：逐步覆盖 `tsv-append`、`tsv-split`、`tsv-sample`、`tsv-uniq`、`tsv-select` 等中等复杂度工具。
- 最后阶段：再评估 `tsv-filter`、`tsv-summarize`、`tsv-join` 等“重量级”工具是否需要完整对标，或只抽取其中一部分能力。

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

`number-lines` 已经可以作为这个通用流程的试点：先用它验证“如何迁移一个工具的测试数据和行为定义”，再按同样的模式扩展到 `tsv-uniq`、`tsv-pretty`、`tsv-summarize` 等其他工具。

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

这使得它在许多基准测试中明显快于其他常见工具（包括部分脚本语言实现）。

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

结合当前 `tva` 的定位（Rust 实现的 TSV 助手），tsv-utils 提供了不少可直接借鉴的设计思路：

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

## 6. 总结

tsv-utils 是一个成熟的 TSV 命令行工具合集，特色在于：
- 覆盖 TSV 数据处理的常见操作（过滤、选择、去重、统计、拆分、连接等）
- 针对大文件做了系统性的性能优化
- 文档与 CLI 体验完整统一

在 `tva` 项目中，可以把 tsv-utils 视为一个“功能地图”和“经验库”：一方面按需对标其工具集合规划 `tva` 的子命令，另一方面在实现与文档风格上保持一致或有意识地作出改进。
