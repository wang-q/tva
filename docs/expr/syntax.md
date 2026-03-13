# 表达式语法

TVA 表达式是一种简洁、强大的数据处理语言，用于在 `tva filter`、`tva mutate` 等命令中进行数据转换和计算。

## 目录

- [快速开始](#快速开始)
- [字面量](#字面量)
- [列引用](#列引用)
- [变量绑定](#变量绑定)
- [操作符](#操作符)
- [函数调用](#函数调用)
- [Lambda 表达式](#lambda-表达式)
- [类型系统](#类型系统)
- [完整示例](#完整示例)

---

## 快速开始

```bash
# 过滤：价格大于100的记录
tva filter -E "@price > 100" data.tsv

# 计算：添加含税价格列
tva mutate -E "@price * 1.1 as @price_with_tax" data.tsv

# 字符串处理：格式化姓名
tva mutate -E "@first ++ ' ' ++ @last as @full_name" data.tsv
```

---

## 字面量

| 类型 | 语法 | 示例 |
|------|------|------|
| 整数 | 数字序列 | `42`, `-10` |
| 浮点数 | 小数点 | `3.14`, `-0.5` |
| 字符串 | 单/双引号 | `"hello"`, `'world'` |
| 布尔 | `true` / `false` | `true`, `false` |
| 空值 | `null` | `null` |
| 列表 | 方括号 | `[1, 2, 3]`, `["a", "b"]` |

### 示例

```rust
42 + @age           // 整数运算
3.14 * @radius      // 浮点数运算
"Hello, " ++ @name  // 字符串拼接
[1, 2, 3]           // 列表字面量
```

---

## 列引用

使用 `@` 前缀引用列，避免与 Shell 变量冲突：

| 语法 | 含义 | 示例 |
|------|------|------|
| `@1`, `@2` | 1-based 列索引 | `@1` 表示第1列 |
| `@col_name` | 列名引用 | `@price` 表示 price 列 |

### 示例

```rust
@1 + @2                 // 第1列加第2列
@price * @qty           // 使用列名
@name | trim() | upper() // 列值链式处理
```

---

## 变量绑定

使用 `as` 关键字将表达式结果绑定到变量，在后续管道中复用：

```rust
// 基本语法
@price * @qty as @total | @total * 1.1

// 多变量绑定
@price as @p | @qty as @q | @p * @q
```

### 规则

- 变量在当前行内有效
- 可遮蔽同名列引用
- 列引用优先检查变量，再回退到列名查找

---

## 操作符

### 优先级表（从高到低）

| 优先级 | 操作符 | 说明 |
|--------|--------|------|
| 1 | `()` | 分组 |
| 2 | `-` (一元) | 负号 |
| 3 | `**` | 乘方 |
| 4 | `*`, `/`, `%` | 乘除模 |
| 5 | `+`, `-` | 加减 |
| 6 | `++` | 字符串拼接 |
| 7 | `==`, `!=`, `<`, `<=`, `>`, `>=` | 数值比较 |
| 8 | `eq`, `ne`, `lt`, `le`, `gt`, `ge` | 字符串比较 |
| 9 | `and` | 逻辑与 |
| 10 | `or` | 逻辑或 |
| 11 | `\|` | 管道 |

### 示例

```rust
// 数值比较
@age >= 18 and @age < 65

// 字符串比较
@name eq "Alice" and @status ne "deleted"

// 算术运算
(@price + @tax) * @qty

// 逻辑运算
@age > 18 or @status eq "vip"
```

---

## 函数调用

### 前缀调用

```rust
trim(@name)
substr(@desc, 0, 50)
```

### 管道调用

```rust
// 单参数函数（_ 可省略）
@name | trim() | upper()

// 多参数函数（使用 _ 占位符）
@desc | substr(_, 0, 50) | upper()
```

### 方法调用

方法调用是函数调用的语法糖：

```rust
@name.trim().upper()
// 等价于 @name | trim() | upper()
```

---

## Lambda 表达式

Lambda 用于创建匿名函数，主要用于 `map`、`filter` 等高阶函数。

### 语法

```rust
// 单参数
x => x + 1

// 多参数
(x, y) => x + y

// 无参数
() => 42
```

### 示例

```rust
// 对列表每个元素加3
map([1, 2, 3], x => x + 3)           // [4, 5, 6]

// 过滤大于2的元素
filter([1, 2, 3, 4], x => x > 2)     // [3, 4]

// 使用列值
map(split(@tags, ","), t => trim(t))
```

---

## 类型系统

TVA 采用动态类型系统：

| 类型 | 说明 | 示例 |
|------|------|------|
| `Int` | 64位有符号整数 | `42` |
| `Float` | 64位浮点数 | `3.14` |
| `String` | UTF-8 字符串 | `"hello"` |
| `Bool` | 布尔值 | `true` |
| `Null` | 空值 | `null` |
| `List` | 异构列表 | `[1, "a", true]` |

### 类型转换

```rust
int("42")       // 42
float("3.14")   // 3.14
string(42)      // "42"
```

---

## 完整示例

```bash
# 过滤：价格大于100且库存大于0
tva filter -E "@price > 100 and @stock > 0" data.tsv

# 计算：总价（含税）
tva mutate -E "@price * (1 + @tax_rate) as @total" data.tsv

# 字符串处理：格式化姓名
tva mutate -E "@first | trim() | upper() ++ ' ' ++ @last | trim() | upper() as @full_name" data.tsv

# 条件：根据年龄分组
tva mutate -E "if(@age < 18, 'minor', if(@age < 60, 'adult', 'senior')) as @group" data.tsv

# 列表处理：分割并处理标签
tva mutate -E "split(@tags, ',') | map(_, t => trim(t)) | join(_, '; ') as @clean_tags" data.tsv
```

---

## 相关文档

- [函数参考](functions.md) - 完整的函数列表
- [操作符详解](operators.md) - 操作符详细说明
