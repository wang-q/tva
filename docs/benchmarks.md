# 性能基准测试计划

我们旨在重现 [tsv-utils](https://github.com/eBay/tsv-utils/blob/master/docs/comparative-benchmarks-2017.md)
使用的严格基准测试策略。

## 1. 基准工具

* [tsv-utils](https://github.com/eBay/tsv-utils) (D): 主要性能对标目标。
* [qsv](https://github.com/jqnatividad/qsv) (Rust): xsv 的活跃分支，功能超级强大。
* [GNU datamash](https://www.gnu.org/software/datamash/) (C): 统计操作的标准。
* [GNU awk](https://www.gnu.org/software/gawk/) / [mawk](https://invisible-island.net/mawk/) (C):
  行过滤和基本处理的基准。
* [csvtk](https://github.com/shenwei356/csvtk) (Go): 另一个现代跨平台工具包。

## 2. 测试数据集与策略

我们将使用不同规模的数据集来全面评估性能。

### 数据集来源

* **HEPMASS** (
  4.8GB): [UCI Machine Learning Repository](https://archive.ics.uci.edu/ml/datasets/HEPMASS)。
    * 内容: 约 700万行，29列数值数据。
    * 用途: 用于**数值行过滤**、**列选择**、**统计摘要**和**文件连接**测试。
* **FIA Tree Data** (
  2.7GB): [USDA Forest Service](https://apps.fs.usda.gov/fia/datamart/CSV/datamart_csv.html)。
    * 内容: `TREE_GRM_ESTN.csv` 的前 1400 万行，包含混合文本和数值。
    * 用途: 用于**正则行过滤**和**CSV 转 TSV**测试。

### 测试策略

* **吞吐量与稳定性 (大文件)**:
    * 使用完整的 GB 级数据集 (HEPMASS, FIA Tree Data)。
    * 目标: 压力测试流处理能力、内存稳定性以及 I/O 吞吐量。
* **启动开销 (小文件)**:
    * 使用 **HEPMASS_100k** (~70MB, HEPMASS 的前 10万行)。
    * 目标: 测试工具的启动开销 (Startup Overhead) 和缓冲策略。对于极短的运行时间，Rust/C 的启动时间差异会更明显。

## 3. 详细测试场景

为了确保公平和全面的对比，我们将执行以下具体场景（参考 tsv-utils 2017/2018）：

* **数值行过滤 (Numeric Filter)**:
    * 逻辑: 多列数值比较 (例如 `col4 > 0.000025 && col16 > 0.3`)。
    * 基准: `tva filter` vs `awk` (mawk/gawk) vs `tsv-filter` (D) vs `qsv search` (Rust)。
    * 目的: 测试数值解析和比较的效率。
* **正则行过滤 (Regex Filter)**:
    * 逻辑: 针对特定文本列的正则匹配 (例如 `[RD].*(ION[0-2])`)。
    * 基准: `tva filter --regex` vs `grep` / `awk` / `ripgrep` (如果适用) vs `tsv-filter` vs
      `qsv search`。
    * 注意: 区分全行匹配与特定字段匹配。
* **列选择 (Column Selection)**:
    * 逻辑: 提取分散的列 (例如 1, 8, 19)。
    * 基准: `tva select` vs `cut` vs `tsv-select` vs `qsv select` vs `csvtk cut`。
    * 注意: 测试不同文件大小。GNU `cut` 在小文件上通常非常快，但在大文件上可能不如流式优化工具。
    * **短行测试 (Short Lines)**: 针对海量短行数据（如 8600万行，1.7GB）进行测试，主要考察每行处理的固定开销。
* **文件连接 (Join)**:
    * **数据准备**: 将大文件拆分为两个文件（例如：左文件含列 1-15，右文件含列 1, 16-29），并**随机打乱**
      行顺序，但保留公共键（列 1）。
    * 逻辑: 基于公共键将两个乱序文件重新连接。
    * 基准: `tva join` vs `join` (Unix - 需先 sort) vs `qsv join` vs `tsv-join` vs `csvtk join`。
    * 目的: 测试哈希表构建和查找的内存与速度平衡。
* **统计摘要 (Summary Statistics)**:
    * 逻辑: 计算多个列的 Count, Sum, Min, Max, Mean, Stdev。
    * 基准: `tva stats` vs `datamash` vs `tsv-summarize` vs `qsv stats` vs `csvtk summary`。
* **CSV 转 TSV (CSV to TSV)**:
    * 逻辑: 处理包含转义字符和嵌入换行符的复杂 CSV。
    * 基准: `tva from csv` vs `qsv fmt` vs `csvtk csv2tab` vs `csv2tsv` (tsv-utils)。
    * 目的: 这是一个高计算密集型任务，测试 CSV 解析器的性能。
* **加权随机采样 (Weighted Sampling)**:
    * 逻辑: 基于权重列进行加权随机采样 (Weighted Reservoir Sampling)。
    * 基准: `tva sample --weight` vs `tsv-sample` vs `qsv sample` (如果支持)。
    * 目的: 测试复杂算法与 I/O 的结合效率。
* **去重 (Deduplication)**:
    * 逻辑: 基于特定列进行哈希去重。
    * 基准: `tva uniq` vs `tsv-uniq` vs `awk` vs `sort | uniq`。
    * 目的: 测试哈希表性能和内存管理。
* **排序 (Sorting)**:
    * 逻辑: 基于数值列进行排序。
    * 基准: `tva sort` vs `sort` (GNU) vs `tsv-sort`。
    * 目的: 测试外部排序算法和内存使用。
* **切片 (Slicing)**:
    * 逻辑: 提取文件中间的大段行 (如第 100万 到 200万 行)。
    * 基准: `tva slice` vs `sed` vs `tail | head`。
    * 目的: 测试快速跳过行的能力。
* **反转 (Reverse)**:
    * 逻辑: 反转整个文件的行序。
    * 基准: `tva reverse` vs `tac`。
* **追加 (Append)**:
    * 逻辑: 连接多个大文件。
    * 基准: `tva append` vs `cat`。
* **导出 CSV (Export to CSV)**:
    * 逻辑: 将 TSV 转换为标准 CSV (处理转义)。
    * 基准: `tva to csv` vs `qsv fmt`。

## 4. 执行环境与记录

* **硬件记录**: 必须记录 CPU 型号、核心数、RAM 大小以及**磁盘类型** (NVMe SSD 对 I/O
  密集型测试影响巨大)。
* **软件版本**:
    * Rust 编译器版本 (`rustc --version`)。
    * 所有对比工具的版本 (`qsv --version`, `awk --version` 等)。
* **预热 (Warmup)**: 使用 `hyperfine --warmup` 确保文件系统缓存处于一致状态（通常是热缓存状态）。

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
    tva nl -H --header-string "row_id" hepmass.tsv > hepmass_numbered.tsv
    # 拆分并打乱
    tva select -f 1-16 hepmass_numbered.tsv | tva sample -H > hepmass_left.tsv
    tva select -f 1,17-30 hepmass_numbered.tsv | tva sample -H > hepmass_right.tsv
    rm hepmass_numbered.tsv
fi

# 2. 运行基准测试 (Run Benchmark)
# ------------------------------
echo "Running Benchmarks..."

# Scenario 1: Numeric Filter
hyperfine \
    --warmup 3 \
    --min-runs 5 \
    --export-csv benchmark_filter.csv \
    -n "tva filter" "tva filter -H --gt 1:0.5 hepmass.tsv > /dev/null" \
    -n "tsv-filter" "tsv-filter -H --gt 1:0.5 hepmass.tsv > /dev/null" \
    -n "awk" "awk -F '\t' '\$1 > 0.5' hepmass.tsv > /dev/null"

# Scenario 2: Column Selection
hyperfine \
    --warmup 3 \
    --min-runs 5 \
    --export-csv benchmark_select.csv \
    -n "tva select" "tva select -f 1,8,19 hepmass.tsv > /dev/null" \
    -n "tsv-select" "tsv-select -f 1,8,19 hepmass.tsv > /dev/null" \
    -n "cut" "cut -f 1,8,19 hepmass.tsv > /dev/null"

# Scenario 3: Join
hyperfine \
    --warmup 3 \
    --min-runs 5 \
    --export-csv benchmark_join.csv \
    -n "tva join" "tva join -H -f hepmass_right.tsv -k 1 hepmass_left.tsv > /dev/null" \
    -n "tsv-join" "tsv-join -H -f hepmass_right.tsv -k 1 hepmass_left.tsv > /dev/null" \
    -n "xan join" "xan join -d '\t' --semi row_id hepmass_left.tsv row_id hepmass_right.tsv > /dev/null"

    # qsv join is too slow
    # "qsv join row_id hepmass_left.tsv row_id hepmass_right.tsv > /dev/null"

# Scenario 4: Summary Statistics
hyperfine \
    --warmup 3 \
    --min-runs 5 \
    --export-csv benchmark_stats.csv \
    -n "tva stats" "tva stats -H --count --sum 3,5,20 --min 3,5,20 --max 3,5,20 --mean 3,5,20 --stdev 3,5,20 hepmass.tsv > /dev/null" \
    -n "tsv-summarize" "tsv-summarize -H --count --sum 3,5,20 --min 3,5,20 --max 3,5,20 --mean 3,5,20 --stdev 3,5,20 hepmass.tsv > /dev/null"

# Scenario 5: Weighted Sampling (k=1000)
# Assumes column 5 is a suitable weight (positive float)
hyperfine \
    --warmup 3 \
    --min-runs 5 \
    --export-csv benchmark_sample.csv \
    -n "tva sample" "tva sample -H --weight-field 5 -n 1000 hepmass.tsv > /dev/null" \
    -n "tsv-sample" "tsv-sample -H --weight-field 5 -n 1000 hepmass.tsv > /dev/null"

# Scenario 6: Uniq (Hash-based Deduplication)
hyperfine \
    --warmup 3 \
    --min-runs 5 \
    --export-csv benchmark_uniq.csv \
    -n "tva uniq" "tva uniq -H -f 1 hepmass.tsv > /dev/null" \
    -n "tsv-uniq" "tsv-uniq -H -f 1 hepmass.tsv > /dev/null"

# Scenario 8: Slice (Middle of file)
hyperfine \
    --warmup 3 \
    --min-runs 5 \
    --export-csv benchmark_slice.csv \
    -n "tva slice" "tva slice -r 1000000-2000000 hepmass.tsv > /dev/null" \
    -n "sed" "sed -n '1000000,2000000p' hepmass.tsv > /dev/null"

```

## 7. expr 对比 专用命令

使用 `docs/data/diamonds.tsv`

### filter

```bash
hyperfine \
    --warmup 3 \
    --min-runs 50 \
    --export-markdown tva_filter.tmp.md \
    -n "tsv-filter" "tsv-filter -H --gt carat:1 --str-eq cut:Premium --lt price:3000 docs/data/diamonds.tsv > /dev/null" \
    -n "xan filter" "xan filter 'carat > 1 and cut eq \"Premium\" and price < 3000' docs/data/diamonds.tsv > /dev/null" \
    -n "tva expr -m skip-null" "tva expr -H -m skip-null -E 'if(@carat > 1 and @cut eq q(Premium) and @price < 3000, @0, null)' docs/data/diamonds.tsv > /dev/null" \
    -n "tva expr -m filter" "tva expr -H -m filter -E '@carat > 1 and @cut eq q(Premium) and @price < 3000' docs/data/diamonds.tsv > /dev/null" \
    -n "tva filter" "tva filter -H --gt carat:1 --str-eq cut:Premium --lt price:3000 docs/data/diamonds.tsv > /dev/null"
```

| Command                 |  Mean [ms] | Min [ms] | Max [ms] |    Relative |
|:------------------------|-----------:|---------:|---------:|------------:|
| `tsv-filter`            | 21.0 ± 1.2 |     18.8 |     24.0 |        1.00 |
| `xan filter`            | 63.3 ± 2.2 |     59.9 |     73.8 | 3.01 ± 0.20 |
| `tva expr -m skip-null` | 54.5 ± 3.0 |     50.7 |     68.6 | 2.59 ± 0.21 |
| `tva expr -m filter`    | 42.3 ± 2.2 |     39.5 |     53.9 | 2.01 ± 0.16 |
| `tva filter`            | 21.0 ± 1.6 |     18.8 |     31.2 | 1.00 ± 0.10 |

### select

```bash
hyperfine \
    --warmup 3 \
    --min-runs 50 \
    --export-markdown tva_select.tmp.md \
    -n "tsv-select" "tsv-select -H -f carat,cut,price docs/data/diamonds.tsv > /dev/null" \
    -n "xan select" "xan select 'carat,cut,price' docs/data/diamonds.tsv > /dev/null" \
    -n "xan select -e" "xan select -e '[carat, cut, price]' docs/data/diamonds.tsv > /dev/null" \
    -n "tva expr -m eval" "tva expr -H -m eval -E '[@carat, @cut, @price]' docs/data/diamonds.tsv > /dev/null" \
    -n "tva select" "tva select -H -f carat,cut,price docs/data/diamonds.tsv > /dev/null"
```

| Command            |  Mean [ms] | Min [ms] | Max [ms] |    Relative |
|:-------------------|-----------:|---------:|---------:|------------:|
| `tsv-select`       | 21.0 ± 1.2 |     18.6 |     24.6 | 1.03 ± 0.09 |
| `xan select`       | 58.8 ± 2.7 |     54.4 |     72.5 | 2.87 ± 0.23 |
| `xan select -e`    | 69.2 ± 1.8 |     65.8 |     73.2 | 3.38 ± 0.24 |
| `tva expr -m eval` | 57.3 ± 2.7 |     53.8 |     68.3 | 2.80 ± 0.22 |
| `tva select`       | 20.5 ± 1.3 |     17.6 |     24.5 |        1.00 |

### reverse

```bash
hyperfine \
    --warmup 3 \
    --min-runs 50 \
    --export-markdown tva_reverse.tmp.md \
    -n "tva reverse" "tva reverse docs/data/diamonds.tsv > /dev/null" \
    -n "tva reverse -H" "tva reverse -H docs/data/diamonds.tsv > /dev/null" \
    -n "tva reverse --no-mmap" "tva reverse --no-mmap docs/data/diamonds.tsv > /dev/null" \
    -n "tac" "tac docs/data/diamonds.tsv > /dev/null" \
    -n "keep-header -- tac" "tva keep-header docs/data/diamonds.tsv -- tac > /dev/null"
```

| Command | Mean [ms] | Min [ms] | Max [ms] | Relative |
|:---|---:|---:|---:|---:|
| `tva reverse` | 92.0 ± 3.2 | 86.0 | 103.1 | 5.28 ± 0.39 |
| `tva reverse -H` | 94.6 ± 5.2 | 88.6 | 116.8 | 5.43 ± 0.46 |
| `tva reverse --no-mmap` | 17.4 ± 1.1 | 14.6 | 21.6 | 1.00 |
| `tac` | 50.2 ± 3.0 | 47.1 | 66.9 | 2.88 ± 0.26 |
| `keep-header -- tac` | 56.7 ± 3.2 | 52.9 | 69.3 | 3.25 ± 0.28 |

`tva reverse` 的基准测试显示了一个反直觉的结果：

**分析**:
- mmap 模式比 `--no-mmap` 慢 **5.3 倍**
- 甚至低于 `tac`（2.88x）

**原因**:
1. **页缓存预读失效**: Linux 内核的预读机制优化顺序读取，反向扫描破坏预读策略
2. **TLB 抖动**: 随机访问模式导致页表遍历开销增加
3. **缺页中断**: 小文件（5MB）完全适合内存，`read_to_end` 一次性读入后连续访问更缓存友好

**代码层面**:
```rust
// mmap 模式: 反向迭代触发随机访问
for i in memrchr_iter(b'\n', slice) {  // 反向查找换行符
    writer.write_all(&slice[i + 1..following_line_start])?;
}

// --no-mmap 模式: Vec<u8> 连续存储，CPU 缓存友好
let mut buf = Vec::new();
f.read_to_end(&mut buf)?;  // 一次性读入
```

**启示**: 对于小文件（<100MB）或反向/随机访问模式，`--no-mmap` 显著优于 mmap。

### uniq


```bash
hyperfine \
    --warmup 3 \
    --min-runs 50 \
    --export-markdown tva_uniq.tmp.md \
    -n "tsv-uniq -f carat" "tsv-uniq -H -f carat docs/data/diamonds.tsv > /dev/null" \
    -n "tsv-uniq -f 1" "tsv-uniq -H -f 1 docs/data/diamonds.tsv > /dev/null" \
    -n "tva uniq -f carat" "tva uniq -H -f carat docs/data/diamonds.tsv > /dev/null" \
    -n "tva uniq -f 1" "tva uniq -H -f 1 docs/data/diamonds.tsv > /dev/null" \
    -n "cut sort uniq" "cut -f 1 docs/data/diamonds.tsv | sort | uniq > /dev/null" \
    -n "tsv-uniq" "tsv-uniq docs/data/diamonds.tsv > /dev/null" \
    -n "tva uniq" "tva uniq docs/data/diamonds.tsv > /dev/null" \
    -n "sort uniq" "sort docs/data/diamonds.tsv | uniq > /dev/null"
``

| Command | Mean [ms] | Min [ms] | Max [ms] | Relative |
|:---|---:|---:|---:|---:|
| `tsv-uniq -f carat` | 35.5 ± 11.3 | 23.9 | 64.8 | 1.00 |
| `tsv-uniq -f 1` | 37.3 ± 11.5 | 26.7 | 86.5 | 1.05 ± 0.46 |
| `tva uniq -f carat` | 41.3 ± 13.2 | 23.4 | 91.9 | 1.16 ± 0.52 |
| `tva uniq -f 1` | 44.7 ± 10.5 | 26.4 | 74.1 | 1.26 ± 0.50 |
| `cut sort uniq` | 175.8 ± 42.4 | 138.4 | 311.1 | 4.96 ± 1.97 |
| `tsv-uniq` | 64.4 ± 17.8 | 41.4 | 103.0 | 1.81 ± 0.76 |
| `tva uniq` | 44.2 ± 6.7 | 30.9 | 63.3 | 1.25 ± 0.44 |
| `sort uniq` | 59.2 ± 11.5 | 47.8 | 96.4 | 1.67 ± 0.62 |

### append

```bash
hyperfine \
    --warmup 3 \
    --min-runs 50 \
    --export-markdown tva_append.tmp.md \
    -n "tsv-append" "tsv-append docs/data/diamonds.tsv docs/data/diamonds.tsv > /dev/null" \
    -n "tva append" "tva append docs/data/diamonds.tsv docs/data/diamonds.tsv > /dev/null" \
    -n "cat" "cat docs/data/diamonds.tsv docs/data/diamonds.tsv > /dev/null"
```

| Command | Mean [ms] | Min [ms] | Max [ms] | Relative |
|:---|---:|---:|---:|---:|
| `tsv-append` | 34.3 ± 3.0 | 30.4 | 47.9 | 1.12 ± 0.10 |
| `tva append` | 33.8 ± 1.7 | 31.0 | 38.0 | 1.11 ± 0.06 |
| `cat` | 30.5 ± 0.9 | 28.4 | 33.3 | 1.00 |


### sort

```bash
hyperfine \
    --warmup 3 \
    --min-runs 50 \
    --export-markdown tva_sort.tmp.md \
    -n "tva sort -k 2" "tva sort -H -k 2 docs/data/diamonds.tsv > /dev/null" \
    -n "sort -k 2" "sort -k 2 docs/data/diamonds.tsv > /dev/null" \
    -n "keep-header -- sort" "keep-header docs/data/diamonds.tsv -- sort -k 2 > /dev/null" \
    -n "tva keep-header -- sort" "tva keep-header docs/data/diamonds.tsv -- sort -k 2 > /dev/null" \
    -n "tva sort" "tva sort docs/data/diamonds.tsv > /dev/null" \
    -n "sort" "sort docs/data/diamonds.tsv > /dev/null"
```

| Command | Mean [ms] | Min [ms] | Max [ms] | Relative |
|:---|---:|---:|---:|---:|
| `tva sort` | 37.6 ± 3.5 | 30.8 | 48.9 | 1.00 |
| `sort` | 39.5 ± 3.3 | 33.7 | 50.2 | 1.05 ± 0.13 |
| `keep-header -- sort` | 42.8 ± 3.6 | 38.6 | 61.0 | 1.14 ± 0.14 |
| `tva keep-header -- sort` | 74.0 ± 3.3 | 68.8 | 85.7 | 1.97 ± 0.20 |
