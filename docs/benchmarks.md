# 性能基准测试计划

我们旨在重现 [tsv-utils](https://github.com/eBay/tsv-utils/blob/master/docs/comparative-benchmarks-2017.md) 使用的严格基准测试策略。

## 1. 基准工具
*   tsv-utils (D): 主要性能对标目标。
*   qsv (Rust): xsv 的活跃分支，功能超级强大。
*   GNU datamash (C): 统计操作的标准。
*   GNU awk / mawk (C): 行过滤和基本处理的基准。
*   csvtk (Go): 另一个现代跨平台工具包。

## 2. 关键指标与数据集
我们将使用大数据集（GB 级）来压力测试流处理能力和内存使用。

*   **数据集来源**:
    *   **HEPMASS**: [UCI Machine Learning Repository](https://archive.ics.uci.edu/ml/datasets/HEPMASS)。约 4.8GB，700万行，29列数值数据。
    *   **FIA Tree Data**: [USDA Forest Service](https://apps.fs.usda.gov/fia/datamart/CSV/datamart_csv.html)。使用 `TREE_GRM_ESTN.csv` 的前 1400 万行 (约 2.7GB)，包含混合文本和数值。

*   **数值行过滤**: `tva filter --gt` vs `awk`。
    *   数据集: HEPMASS (4.8GB)。
*   **正则行过滤**: `tva filter --regex` vs `grep`/`awk`。
    *   数据集: FIA Tree Data (2.7GB)。
*   列选择: `tva select` vs `cut`。
    *   目标: 测量字段解析开销。
*   连接: `tva join` vs `join` (Unix) vs `qsv join`。
    *   场景: 大文件与子集文件基于公共键连接。
*   统计: `tva stats` vs `datamash`。
    *   操作: GroupBy + Sum/Mean/Min/Max。
*   CSV 转 TSV: `tva from csv` vs `qsv fmt`。
    *   目标: 测量解析速度和正确性（引号处理）。

## 3. 详细测试场景

为了确保公平和全面的对比，我们将执行以下具体场景（参考 tsv-utils 2017）：

*   **数值行过滤 (Numeric Filter)**:
    *   逻辑: 多列数值比较 (例如 `col4 > 0.000025 && col16 > 0.3`)。
    *   基准: `tva filter` vs `awk` (mawk/gawk)。
    *   目的: 测试数值解析和比较的效率。
*   **正则行过滤 (Regex Filter)**:
    *   逻辑: 针对特定文本列的正则匹配 (例如 `[RD].*(ION[0-2])`)。
    *   基准: `tva filter --regex` vs `grep` / `awk` / `ripgrep` (如果适用)。
    *   注意: 区分全行匹配与特定字段匹配。
*   **列选择 (Column Selection)**:
    *   逻辑: 提取分散的列 (例如 1, 8, 19)。
    *   基准: `tva select` vs `cut`。
    *   注意: 测试不同文件大小。GNU `cut` 在小文件上通常非常快，但在大文件上可能不如流式优化工具。
    *   **短行测试 (Short Lines)**: 针对海量短行数据（如 8600万行，1.7GB）进行测试，主要考察每行处理的固定开销。
*   **文件连接 (Join)**:
    *   **数据准备**: 将大文件拆分为两个文件（例如：左文件含列 1-15，右文件含列 1, 16-29），并**随机打乱**行顺序，但保留公共键（列 1）。
    *   逻辑: 基于公共键将两个乱序文件重新连接。
    *   基准: `tva join` vs `join` (Unix - 需先 sort) vs `qsv join`。
    *   目的: 测试哈希表构建和查找的内存与速度平衡。
*   **统计摘要 (Summary Statistics)**:
    *   逻辑: 计算多个列的 Count, Sum, Min, Max, Mean, Stdev。
    *   基准: `tva stats` vs `datamash`。
*   **CSV 转 TSV (CSV to TSV)**:
    *   逻辑: 处理包含转义字符和嵌入换行符的复杂 CSV。
    *   基准: `tva from csv` vs `qsv fmt`。
    *   目的: 这是一个高计算密集型任务，测试 CSV 解析器的性能。

## 4. 执行环境与记录

*   **硬件记录**: 必须记录 CPU 型号、核心数、RAM 大小以及**磁盘类型** (NVMe SSD 对 I/O 密集型测试影响巨大)。
*   **软件版本**:
    *   Rust 编译器版本 (`rustc --version`)。
    *   所有对比工具的版本 (`qsv --version`, `awk --version` 等)。
*   **文件大小差异**:
    *   **大文件 (GB级)**: 主要测试吞吐量和内存稳定性。
    *   **小文件 (MB级)**: 测试启动开销 (Startup Overhead) 和缓冲策略。
*   **预热 (Warmup)**: 使用 `hyperfine --warmup` 确保文件系统缓存处于一致状态（通常是热缓存状态）。

## 5. 执行工作流示例

我们将使用内联 Bash 脚本与 `hyperfine` 结合，实现完全自动化的基准测试。

```bash
# 1. 数据准备 (Data Preparation)
# ------------------------------
# 下载并解压 HEPMASS (如果不存在)
if [ ! -f "hepmass.tsv" ]; then
    echo "Downloading HEPMASS dataset..."
    curl -O https://archive.ics.uci.edu/ml/machine-learning-databases/00347/all_train.csv.gz
    gzip -d all_train.csv.gz
    # 转换为 TSV
    tva from csv all_train.csv > hepmass.tsv
fi

# 准备 Join 测试数据 (拆分并乱序)
if [ ! -f "hepmass_left.tsv" ]; then
    echo "Preparing Join datasets..."
    # 添加行号作为唯一键
    tva number-lines -H --header-name "row_id" hepmass.tsv > hepmass_numbered.tsv
    # 拆分并打乱
    tva select -f 1-16 hepmass_numbered.tsv | tsv-sample -H > hepmass_left.tsv
    tva select -f 1,17-30 hepmass_numbered.tsv | tsv-sample -H > hepmass_right.tsv
    rm hepmass_numbered.tsv
fi

# 2. 运行基准测试 (Run Benchmark)
# ------------------------------
echo "Running Benchmarks..."
hyperfine \
    --warmup 3 \
    --min-runs 10 \
    --export-csv benchmark_results.csv \
    --parameter-scan threads 1 4 \
    "tva filter --gt 1:0.5 hepmass.tsv" \
    "awk -F '\t' '\$1 > 0.5' hepmass.tsv" \
    "tva select -f 1,8,19 hepmass.tsv" \
    "cut -f 1,8,19 hepmass.tsv"

# 3. 结果处理与可视化 (Process & Visualize)
# ------------------------------
# 使用 tva 将 hyperfine 的 CSV 结果转换为 TSV
tva from csv benchmark_results.csv > benchmark_results.tsv

# 使用 Python 绘图 (内联脚本)
python3 -c "
import pandas as pd
import seaborn as sns
import matplotlib.pyplot as plt

# 读取由 tva 转换的 TSV 数据
df = pd.read_csv('benchmark_results.tsv', sep='\t')

# 简单的条形图
plt.figure(figsize=(10, 6))
sns.barplot(x='mean', y='command', data=df)
plt.title('Benchmark Results: Execution Time (s)')
plt.xlabel('Time (seconds)')
plt.tight_layout()
plt.savefig('benchmark_plot.png')
print('Plot saved to benchmark_plot.png')
"
```

## 6. 优化目标

*   **内存使用**: 确保流式命令（filter, select, from-csv）保持 O(1) 内存使用。
*   **零拷贝**: 尽可能使用零拷贝解析技术（类似 `csv` crate 的 `ByteRecord`）。
*   **I/O 效率**: 确保所有读写操作都经过 `BufReader`/`BufWriter` 缓冲。
*   **构建优化**:
    *   **LTO (Link Time Optimization)**: `Cargo.toml` 中已启用 `lto = true`，这对减少二进制大小和提高运行时性能至关重要。
    *   **PGO (Profile Guided Optimization)**: 未来探索方向。使用真实工作负载数据来指导编译器优化，进一步压榨性能。
