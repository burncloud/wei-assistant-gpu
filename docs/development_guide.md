# 供应商信息管理系统 - 开发者指南

本文档提供了供应商信息管理系统的技术实现细节、代码结构和扩展指南，供开发者参考。

## 1. 技术架构

供应商信息管理系统是一个基于Rust语言开发的命令行应用程序，采用以下技术栈：

- **Rust**: 主编程语言
- **SQLite**: 本地数据存储
- **Clap**: 命令行参数解析
- **Serde**: 数据序列化/反序列化
- **Rusqlite**: SQLite数据库连接

### 1.1 项目结构

```
wei-assistant-gpu/
├── src/
│   └── main.rs           # 源代码（单文件结构）
├── tests/                # 测试代码
│   ├── cli_integration.rs # 命令行集成测试
│   └── performance_test.rs # 性能测试
├── docs/                 # 文档
│   ├── user_guide.md     # 用户指南
│   └── development_guide.md # 开发者指南
├── Cargo.toml            # 项目配置和依赖
├── Cargo.lock            # 依赖锁定文件
└── README.md             # 项目说明
```

### 1.2 核心模块

系统主要分为以下几个核心模块：

1. **命令行参数解析**: 使用Clap库实现的命令行界面
2. **数据库初始化**: 自动检测并创建SQLite数据库
3. **数据插入**: 支持JSON和参数两种方式添加数据
4. **数据查询**: 支持多条件筛选和复杂查询
5. **输出格式化**: 表格、JSON和CSV输出支持

## 2. 数据模型

### 2.1 Supplier 结构体

```rust
#[derive(Debug, Deserialize)]
struct Supplier {
    contact: Option<String>,
    wechat: Option<String>,
    phone: Option<String>,
    quantity: Option<i32>,
    location: Option<String>,
    price: Option<f64>,
    bandwidth_price: Option<f64>,
    storage_price: Option<f64>,
    min_contract_period: Option<String>,
    breach_penalties: Option<String>,
    payment_terms: Option<String>,
    server_name: Option<String>,
    server_config: Option<String>,
    rental_model: Option<String>,
    networking_category: Option<String>,
}
```

此结构体用于表示从命令行或JSON输入的供应商数据。

### 2.2 SupplierRow 结构体

```rust
#[derive(Debug, Serialize)]
struct SupplierRow {
    id: i32,
    contact: Option<String>,
    wechat: Option<String>,
    phone: Option<String>,
    quantity: Option<i32>,
    location: Option<String>,
    price: Option<f64>,
    bandwidth_price: Option<f64>,
    storage_price: Option<f64>,
    min_contract_period: Option<String>,
    breach_penalties: Option<String>,
    payment_terms: Option<String>,
    server_name: Option<String>,
    server_config: Option<String>,
    rental_model: Option<String>,
    networking_category: Option<String>,
}
```

此结构体用于表示从数据库中查询出的供应商数据，包含ID字段。

### 2.3 数据库表结构

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

## 3. 查询系统设计

系统实现了灵活的查询构建器模式，支持动态构建SQL查询。

### 3.1 查询相关枚举和结构体

```rust
// 字段名枚举
enum SupplierField {
    ContactPerson,
    Wechat,
    Phone,
    // ... 其他字段
}

// 比较操作符
enum ComparisonOp {
    Eq,         // =
    Neq,        // !=
    Gt,         // >
    Lt,         // <
    Gte,        // >=
    Lte,        // <=
    Like,       // LIKE
    IsNull,     // IS NULL
    IsNotNull,  // IS NOT NULL
}

// 单个筛选条件
struct FilterCriteria {
    field: SupplierField,
    op: ComparisonOp,
    value: Option<String>, // IS NULL/IS NOT NULL 时为 None
}

// 查询构建器
struct QueryBuilder {
    filters: Vec<FilterCriteria>,
}
```

### 3.2 查询构建逻辑

1. 从命令行参数解析过滤条件
2. 将过滤条件转换为FilterCriteria对象
3. 使用QueryBuilder动态构建SQL查询字符串
4. 执行SQL查询并处理结果

## 4. 错误处理策略

系统采用以下错误处理策略：

1. **返回类型**: 使用Rust的`Result<T, E>`类型进行错误传播
2. **错误显示**: 用户友好的错误信息
3. **提前检查**: 在执行关键操作前验证参数有效性
4. **安全处理**: 使用参数化查询防止SQL注入

示例：
```rust
match insert_supplier(&supplier) {
    Ok(_) => println!("供应商信息添加成功！"),
    Err(e) => eprintln!("添加失败: {}", e),
}
```

## 5. 测试策略

### 5.1 单元测试

单元测试验证各个组件的独立功能：

```rust
#[test]
fn test_init_db() {
    let (_dbfile, db_path) = setup_test_db();
    let conn = Connection::open(&db_path).unwrap();
    let mut stmt = conn.prepare("SELECT name FROM sqlite_master WHERE type='table' AND name='suppliers'").unwrap();
    let exists: bool = stmt.exists([]).unwrap();
    assert!(exists);
}
```

### 5.2 集成测试

集成测试验证端到端功能：

```rust
#[test]
fn test_add_and_query_complete_workflow() {
    // 创建测试数据库
    let (_db_file, db_path) = create_test_db().unwrap();
    
    // 添加供应商
    let mut cmd = Command::cargo_bin("wei-assistant-gpu").unwrap();
    cmd.env("DB_FILE", &db_path)
        .arg("add")
        .arg("--contact").arg("测试供应商")
        // ... 其他字段
        .assert()
        .success();
    
    // 查询验证
    let mut cmd = Command::cargo_bin("wei-assistant-gpu").unwrap();
    cmd.env("DB_FILE", &db_path)
        .arg("query")
        .assert()
        .success()
        .stdout(predicate::str::contains("测试供应商"));
}
```

### 5.3 性能测试

性能测试位于`tests/performance_test.rs`，测试系统在大量数据下的性能表现。

## 6. 扩展指南

### 6.1 添加新字段

要添加新的供应商字段，需要修改以下几个部分：

1. 更新`Supplier`和`SupplierRow`结构体
2. 更新数据库表结构和初始化SQL
3. 更新命令行参数定义
4. 更新`insert_supplier`函数
5. 更新`SupplierField`枚举和解析
6. 添加相应的测试用例

### 6.2 添加新命令

要添加新命令（如`update`或`delete`），需要：

1. 在`Commands`枚举中添加新命令
2. 为新命令创建参数结构体（如果需要）
3. 在`main`函数的匹配块中处理新命令
4. 实现相应的业务逻辑函数
5. 添加测试用例

示例：
```rust
enum Commands {
    // 现有命令
    Add { /* ... */ },
    Query(QueryArgs),
    // 新增命令
    Delete {
        #[arg(long)]
        id: i32,
    },
}
```

### 6.3 输出格式扩展

要添加新的输出格式（如XML），需要：

1. 在`QueryArgs`结构体中添加新的格式标志
2. 在查询命令处理中检测该标志
3. 实现格式化逻辑
4. 更新帮助文档

## 7. 性能优化建议

1. **批量插入**: 使用事务进行批量数据插入
2. **索引优化**: 根据查询模式添加数据库索引
3. **连接池**: 对于高并发场景，考虑实现数据库连接池
4. **查询优化**: 优化复杂查询，只选择必要的字段

## 8. 调试技巧

1. **环境变量**: 设置`RUST_LOG=debug`获取详细日志
2. **SQLite查看工具**: 使用DB Browser for SQLite等工具检查数据库
3. **测试数据库**: 使用`DB_FILE`环境变量指向测试数据库

---

希望本指南能帮助您理解系统架构并进行扩展开发。如有问题，请联系开发团队。 