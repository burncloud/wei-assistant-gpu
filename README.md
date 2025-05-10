# 供应商信息管理系统

这是一个基于Rust实现的命令行工具，用于管理供应商信息，包括服务器配置、价格、位置等详细信息。该工具支持通过命令行参数或JSON格式添加供应商信息，并提供灵活的查询功能。

## 文档

详细文档请参考以下链接：

- [安装指南](docs/setup_guide.md) - 如何在各种系统上安装和配置
- [用户指南](docs/user_guide.md) - 完整的使用说明和示例
- [开发者指南](docs/development_guide.md) - 技术实现和扩展指南

## 功能特点

- 支持命令行参数和JSON两种方式添加供应商信息
- 强大的查询功能，支持多字段组合筛选
- 支持表格和JSON两种输出格式
- 支持导出查询结果为CSV文件
- 使用SQLite数据库本地存储数据
- 自动初始化数据库结构

## 安装方法

### 环境要求
- Rust 开发环境
- Cargo 包管理器

### 从源码安装

1. 克隆仓库到本地：
   ```bash
   git clone https://github.com/yourusername/wei-assistant-gpu.git
   cd wei-assistant-gpu
   ```

2. 编译项目：
   ```bash
   cargo build --release
   ```

3. 运行程序：
   ```bash
   ./target/release/wei-assistant-gpu --help
   ```

## 使用指南

### 基本命令

程序支持两个主要命令：`add` 和 `query`。

```bash
# 查看帮助信息
wei-assistant-gpu --help

# 查看添加命令帮助
wei-assistant-gpu add --help

# 查看查询命令帮助
wei-assistant-gpu query --help
```

### 添加供应商信息

#### 通过参数方式添加：

```bash
wei-assistant-gpu add --contact 张三 --wechat zhangsanwx --phone 13800000000 --quantity 10 --location 北京 --price 1000 --bandwidth_price 100 --storage_price 50 --min_contract_period 1年 --breach_penalties 无 --payment_terms 月付 --server_name 服务器A --server_config "8核16G" --rental_model 包年 --networking_category 专线
```

#### 通过JSON方式添加：

```bash
wei-assistant-gpu add --json '{"contact":"李四","wechat":"lisiwx","phone":"13900000000","quantity":5,"location":"上海","price":1200,"bandwidth_price":80,"storage_price":40,"min_contract_period":"2年","breach_penalties":"违约金2000元","payment_terms":"季付","server_name":"服务器B","server_config":"16核32G","rental_model":"包月","networking_category":"BGP"}'
```

### 查询供应商信息

#### 查询所有供应商：

```bash
wei-assistant-gpu query
```

#### 按条件筛选查询：

```bash
# 单条件查询
wei-assistant-gpu query --location 北京

# 多条件组合查询
wei-assistant-gpu query --location 北京 --price 1000
```

#### 输出为JSON格式：

```bash
wei-assistant-gpu query --json
```

#### 导出为CSV文件：

```bash
wei-assistant-gpu query --export-csv suppliers.csv
```

#### 筛选并导出为CSV：

```bash
wei-assistant-gpu query --location 北京 --export-csv beijing_suppliers.csv
```

## 支持的字段

供应商信息包含以下字段：

| 字段名 | 说明 | 类型 | 示例 |
|-------|------|------|------|
| contact | 联系人 | 文本 | 张三 |
| wechat | 微信号 | 文本 | zhangsan123 |
| phone | 电话号码 | 文本 | 13800000000 |
| quantity | 数量 | 整数 | 10 |
| location | 位置 | 文本 | 北京 |
| price | 价格 | 浮点数 | 1000.0 |
| bandwidth_price | 带宽价格 | 浮点数 | 100.0 |
| storage_price | 存储价格 | 浮点数 | 50.0 |
| min_contract_period | 最短合同期 | 文本 | 1年 |
| breach_penalties | 违约金 | 文本 | 无 |
| payment_terms | 付款方式 | 文本 | 月付 |
| server_name | 服务器名称 | 文本 | 服务器A |
| server_config | 服务器配置 | 文本 | 8核16G |
| rental_model | 租赁模式 | 文本 | 包年 |
| networking_category | 网络类型 | 文本 | 专线 |

## 数据库结构

程序使用SQLite作为数据库存储，数据库文件默认为`wei-assistant.db`，表结构如下：

```sql
CREATE TABLE IF NOT EXISTS suppliers (
    id INTEGER PRIMARY KEY AUTOINCREMENT, -- 主键，自增
    contact TEXT NOT NULL,                -- 联系人
    wechat TEXT,                          -- 微信
    phone TEXT,                           -- 电话
    quantity INTEGER,                     -- 数量
    location TEXT,                        -- 地点
    price REAL,                           -- 价格
    bandwidth_price REAL,                 -- 带宽价格
    storage_price REAL,                   -- 存储价格
    min_contract_period TEXT,             -- 最短合同期
    breach_penalties TEXT,                -- 违约金
    payment_terms TEXT,                   -- 付款方式
    server_name TEXT,                     -- 服务器名称
    server_config TEXT,                   -- 服务器配置
    rental_model TEXT,                    -- 租赁模式
    networking_category TEXT              -- 网络类型
);
```

## 环境变量配置

- `DB_FILE`: 可以通过此环境变量指定数据库文件路径，默认为当前目录下的`wei-assistant.db`

例如：
```bash
DB_FILE=/path/to/custom.db wei-assistant-gpu query
```

## 错误处理

程序在运行时会处理各种常见错误：

- 数据库连接失败
- JSON解析错误
- 参数格式不正确
- 数据库查询出错

当错误发生时，程序会显示清晰的错误信息，帮助用户快速定位问题。

## 开发者信息

- 这个项目使用Rust语言开发
- 使用了以下主要依赖库：
  - rusqlite: SQLite数据库连接
  - clap: 命令行参数解析
  - serde: 数据序列化/反序列化
  - serde_json: JSON处理

## 许可证

MIT License