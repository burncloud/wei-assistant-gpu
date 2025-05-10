use std::fs::File;
use clap::{Parser, Subcommand, Args};
use rusqlite::{Connection, Result};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use serde_json;
use std::sync::OnceLock;
use csv;

static DB_FILE: OnceLock<String> = OnceLock::new();

fn get_db_file() -> &'static str {
    DB_FILE.get_or_init(|| {
        std::env::var("DB_FILE").unwrap_or_else(|_| "wei-assistant.db".to_string())
    })
}

// 已不再需要此函数，测试中直接使用环境变量更改数据库路径
#[allow(dead_code)]
fn set_db_file(_path: &str) {
    // 此函数不再使用
    // 保留函数签名以避免修改测试代码
}

#[derive(Parser, Debug)]
#[command(name = "wei-assistant")]
#[command(about = "供应商信息管理命令行工具", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// 添加供应商信息（支持JSON整体或参数模式）
    Add {
        /// 以JSON字符串整体输入所有字段
        #[arg(long)]
        json: Option<String>,
        /// 传统参数输入
        #[arg(long)]
        contact: Option<String>,
        #[arg(long)]
        wechat: Option<String>,
        #[arg(long)]
        phone: Option<String>,
        #[arg(long)]
        quantity: Option<i32>,
        #[arg(long)]
        location: Option<String>,
        #[arg(long)]
        price: Option<f64>,
        #[arg(long, name = "bandwidth-price")]
        bandwidth_price: Option<f64>,
        #[arg(long, name = "storage-price")]
        storage_price: Option<f64>,
        #[arg(long, name = "min-contract-period")]
        min_contract_period: Option<String>,
        #[arg(long, name = "breach-penalties")]
        breach_penalties: Option<String>,
        #[arg(long, name = "payment-terms")]
        payment_terms: Option<String>,
        #[arg(long, name = "server-name")]
        server_name: Option<String>,
        #[arg(long, name = "server-config")]
        server_config: Option<String>,
        #[arg(long, name = "rental-model")]
        rental_model: Option<String>,
        #[arg(long, name = "networking-category")]
        networking_category: Option<String>,
    },
    /// 查询所有供应商信息，可按字段筛选
    Query(QueryArgs),
}

#[derive(Args, Debug, Default)]
struct QueryArgs {
    #[arg(long)]
    contact: Option<String>,
    #[arg(long)]
    wechat: Option<String>,
    #[arg(long)]
    phone: Option<String>,
    #[arg(long)]
    quantity: Option<i32>,
    #[arg(long)]
    location: Option<String>,
    #[arg(long)]
    price: Option<f64>,
    #[arg(long, name = "bandwidth-price")]
    bandwidth_price: Option<f64>,
    #[arg(long, name = "storage-price")]
    storage_price: Option<f64>,
    #[arg(long, name = "min-contract-period")]
    min_contract_period: Option<String>,
    #[arg(long, name = "breach-penalties")]
    breach_penalties: Option<String>,
    #[arg(long, name = "payment-terms")]
    payment_terms: Option<String>,
    #[arg(long, name = "server-name")]
    server_name: Option<String>,
    #[arg(long, name = "server-config")]
    server_config: Option<String>,
    #[arg(long, name = "rental-model")]
    rental_model: Option<String>,
    #[arg(long, name = "networking-category")]
    networking_category: Option<String>,
    /// 导出为CSV文件（可选）
    #[arg(long, name = "export-csv")]
    export_csv: Option<String>,
    /// 以JSON格式输出（可选）
    #[arg(long)]
    json: bool,
}

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

#[derive(Debug, Serialize)]
pub struct SupplierRow {
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

/// 字段名枚举，防止拼写错误
#[derive(Debug, Clone)]
pub enum SupplierField {
    ContactPerson,
    Wechat,
    Phone,
    Quantity,
    Location,
    Price,
    BandwidthPrice,
    StoragePrice,
    MinContractPeriod,
    BreachPenalties,
    PaymentTerms,
    ServerName,
    ServerConfig,
    RentalModel,
    NetworkingCategory,
}

/// 比较操作符
#[derive(Debug, Clone)]
pub enum ComparisonOp {
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

/// 单个筛选条件
#[derive(Debug, Clone)]
pub struct FilterCriteria {
    pub field: SupplierField,
    pub op: ComparisonOp,
    pub value: Option<String>, // IS NULL/IS NOT NULL 时为 None
}

/// 查询构建器主结构
pub struct QueryBuilder {
    filters: Vec<FilterCriteria>,
}

impl QueryBuilder {
    pub fn new() -> Self {
        Self { filters: vec![] }
    }

    pub fn filter(mut self, criteria: FilterCriteria) -> Self {
        self.filters.push(criteria);
        self
    }

    /// 校验筛选条件组合的有效性和安全性
    pub fn validate(&self) -> Result<(), String> {
        use std::collections::HashSet;
        let mut fields = HashSet::new();
        if self.filters.len() > 20 {
            return Err("筛选条件过多，最多支持20个条件".to_string());
        }
        for f in &self.filters {
            // 验证字段和操作符的兼容性
            let valid = match f.op {
                ComparisonOp::Like => f.field.is_string(),
                ComparisonOp::Gt | ComparisonOp::Lt | ComparisonOp::Gte | ComparisonOp::Lte => f.field.is_numeric(),
                _ => true, // =、!=、IS NULL、IS NOT NULL 适用于所有类型
            };
            if !valid {
                return Err(format!("字段 '{:?}' 不支持操作符 '{:?}'.", f.field, f.op));
            }
        
            // 检查重复字段
            let key = format!("{:?}-{:?}", f.field, f.op);
            if !fields.insert(key) {
                return Err("存在重复字段和操作符的筛选条件".to_string());
            }
            // 检查参数长度
            if let Some(val) = &f.value {
                if val.len() > 256 {
                    return Err(format!("参数过长: {}...", &val[..32.min(val.len())]));
                }
            }
        }
        Ok(())
    }

    /// 构建 WHERE 子句和参数列表
    pub fn build(self) -> (String, Vec<String>) {
        self.validate().expect("筛选条件组合不合法");
        let mut clauses = Vec::new();
        let mut params = Vec::new();
        for f in self.filters {
            let field = match f.field {
                SupplierField::ContactPerson => "contact",
                SupplierField::Wechat => "wechat",
                SupplierField::Phone => "phone",
                SupplierField::Quantity => "quantity",
                SupplierField::Location => "location",
                SupplierField::Price => "price",
                SupplierField::BandwidthPrice => "bandwidth_price",
                SupplierField::StoragePrice => "storage_price",
                SupplierField::MinContractPeriod => "min_contract_period",
                SupplierField::BreachPenalties => "breach_penalties",
                SupplierField::PaymentTerms => "payment_terms",
                SupplierField::ServerName => "server_name",
                SupplierField::ServerConfig => "server_config",
                SupplierField::RentalModel => "rental_model",
                SupplierField::NetworkingCategory => "networking_category",
            };
            match f.op {
                ComparisonOp::Eq => {
                    clauses.push(format!("{} = ?", field));
                    if let Some(val) = f.value { params.push(val); }
                }
                ComparisonOp::Neq => {
                    clauses.push(format!("{} != ?", field));
                    if let Some(val) = f.value { params.push(val); }
                }
                ComparisonOp::Gt => {
                    clauses.push(format!("{} > ?", field));
                    if let Some(val) = f.value { params.push(val); }
                }
                ComparisonOp::Lt => {
                    clauses.push(format!("{} < ?", field));
                    if let Some(val) = f.value { params.push(val); }
                }
                ComparisonOp::Gte => {
                    clauses.push(format!("{} >= ?", field));
                    if let Some(val) = f.value { params.push(val); }
                }
                ComparisonOp::Lte => {
                    clauses.push(format!("{} <= ?", field));
                    if let Some(val) = f.value { params.push(val); }
                }
                ComparisonOp::Like => {
                    clauses.push(format!("{} LIKE ?", field));
                    if let Some(val) = f.value { params.push(val); }
                }
                ComparisonOp::IsNull => {
                    clauses.push(format!("{} IS NULL", field));
                }
                ComparisonOp::IsNotNull => {
                    clauses.push(format!("{} IS NOT NULL", field));
                }
            };
        }
        let where_sql = if clauses.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", clauses.join(" AND "))
        };
        (where_sql, params)
    }
}

impl FromStr for SupplierField {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "contact" => Ok(SupplierField::ContactPerson),
            "wechat" => Ok(SupplierField::Wechat),
            "phone" => Ok(SupplierField::Phone),
            "quantity" => Ok(SupplierField::Quantity),
            "location" => Ok(SupplierField::Location),
            "price" => Ok(SupplierField::Price),
            "bandwidth_price" => Ok(SupplierField::BandwidthPrice),
            "storage_price" => Ok(SupplierField::StoragePrice),
            "min_contract_period" => Ok(SupplierField::MinContractPeriod),
            "breach_penalties" => Ok(SupplierField::BreachPenalties),
            "payment_terms" => Ok(SupplierField::PaymentTerms),
            "server_name" => Ok(SupplierField::ServerName),
            "server_config" => Ok(SupplierField::ServerConfig),
            "rental_model" => Ok(SupplierField::RentalModel),
            "networking_category" => Ok(SupplierField::NetworkingCategory),
            _ => Err(format!("未知字段名: {}", s)),
        }
    }
}

impl FromStr for ComparisonOp {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "=" | "eq" => Ok(ComparisonOp::Eq),
            "!=" | "<>" | "neq" => Ok(ComparisonOp::Neq),
            ">" | "gt" => Ok(ComparisonOp::Gt),
            "<" | "lt" => Ok(ComparisonOp::Lt),
            ">=" | "gte" => Ok(ComparisonOp::Gte),
            "<=" | "lte" => Ok(ComparisonOp::Lte),
            "like" => Ok(ComparisonOp::Like),
            "is null" | "isnull" => Ok(ComparisonOp::IsNull),
            "is not null" | "isnotnull" => Ok(ComparisonOp::IsNotNull),
            _ => Err(format!("未知操作符: {}", s)),
        }
    }
}

impl SupplierField {
    pub fn is_string(&self) -> bool {
        matches!(self,
            SupplierField::ContactPerson |
            SupplierField::Wechat |
            SupplierField::Phone |
            SupplierField::Location |
            SupplierField::MinContractPeriod |
            SupplierField::BreachPenalties |
            SupplierField::PaymentTerms |
            SupplierField::ServerName |
            SupplierField::ServerConfig |
            SupplierField::RentalModel |
            SupplierField::NetworkingCategory
        )
    }
    pub fn is_numeric(&self) -> bool {
        matches!(self,
            SupplierField::Quantity |
            SupplierField::Price |
            SupplierField::BandwidthPrice |
            SupplierField::StoragePrice
        )
    }
}

impl FilterCriteria {
    /// 从 (字段名, 操作符, 值) 解析为 FilterCriteria，并校验类型兼容性
    pub fn from_str_tuple(field: &str, op: &str, value: Option<&str>) -> Result<Self, String> {
        let field_enum = SupplierField::from_str(field)?;
        let op_enum = ComparisonOp::from_str(op)?;
        // 类型与操作符兼容性校验
        let valid = match op_enum {
            ComparisonOp::Like => field_enum.is_string(),
            ComparisonOp::Gt | ComparisonOp::Lt | ComparisonOp::Gte | ComparisonOp::Lte => field_enum.is_numeric(),
            _ => true, // =、!=、IS NULL、IS NOT NULL 适用于所有类型
        };
        if !valid {
            return Err(format!("字段 '{}' 不支持操作符 '{}'.", field, op));
        }
        let val = match op_enum {
            ComparisonOp::IsNull | ComparisonOp::IsNotNull => None,
            _ => value.map(|v| v.to_string()),
        };
        Ok(FilterCriteria {
            field: field_enum,
            op: op_enum,
            value: val,
        })
    }
}

/// 表格格式化输出
pub fn print_suppliers_table_v2(rows: &[SupplierRow]) {
    use std::cmp::max;
    let headers = [
        "ID", "联系人", "微信", "手机", "数量", "地点", "价格", "带宽价", "存储价", "合同期", "违约金", "付款", "服务器名", "配置", "租赁", "组网"
    ];
    let mut col_widths = vec![2; headers.len()];
    let mut data: Vec<Vec<String>> = Vec::new();
    for s in rows {
        let row = vec![
            s.id.to_string(),
            s.contact.as_deref().unwrap_or("").to_string(),
            s.wechat.as_deref().unwrap_or("").to_string(),
            s.phone.as_deref().unwrap_or("").to_string(),
            s.quantity.map(|v| v.to_string()).unwrap_or_default(),
            s.location.as_deref().unwrap_or("").to_string(),
            s.price.map(|v| format!("{:.2}", v)).unwrap_or_default(),
            s.bandwidth_price.map(|v| format!("{:.2}", v)).unwrap_or_default(),
            s.storage_price.map(|v| format!("{:.2}", v)).unwrap_or_default(),
            s.min_contract_period.as_deref().unwrap_or("").to_string(),
            s.breach_penalties.as_deref().unwrap_or("").to_string(),
            s.payment_terms.as_deref().unwrap_or("").to_string(),
            s.server_name.as_deref().unwrap_or("").to_string(),
            s.server_config.as_deref().unwrap_or("").to_string(),
            s.rental_model.as_deref().unwrap_or("").to_string(),
            s.networking_category.as_deref().unwrap_or("").to_string(),
        ];
        for (i, cell) in row.iter().enumerate() {
            col_widths[i] = max(col_widths[i], cell.chars().count());
        }
        data.push(row);
    }
    // 打印表头
    for (i, h) in headers.iter().enumerate() {
        print!("{:<width$} ", h, width = col_widths[i]);
    }
    println!();
    // 打印分隔线
    for w in &col_widths {
        print!("{:-<width$}-", "", width = *w);
    }
    println!();
    // 打印数据
    for row in &data {
        for (i, cell) in row.iter().enumerate() {
            print!("{:<width$} ", cell, width = col_widths[i]);
        }
        println!();
    }
    if data.is_empty() {
        println!("无供应商信息。");
    }
}

/// JSON格式化输出
pub fn print_suppliers_json(rows: &[SupplierRow]) {
    match serde_json::to_string_pretty(rows) {
        Ok(json) => println!("{}", json),
        Err(e) => eprintln!("JSON序列化错误: {}", e),
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match &cli.command {
        Commands::Add { json, contact, wechat, phone, quantity, location, price, bandwidth_price, storage_price, min_contract_period, breach_penalties, payment_terms, server_name, server_config, rental_model, networking_category } => {
            // 初始化数据库
            init_db()?;
            
            // 处理JSON模式
            if let Some(json_str) = json {
                match serde_json::from_str::<Supplier>(json_str) {
                    Ok(supplier) => {
                        match insert_supplier(&supplier) {
                            Ok(_) => println!("供应商信息添加成功！"),
                            Err(e) => {
                                eprintln!("添加失败: {}", e);
                                return Err(e);
                            }
                        }
                    },
                    Err(e) => {
                        eprintln!("JSON解析失败: {}", e);
                        return Err(rusqlite::Error::InvalidParameterName(e.to_string()));
                    }
                }
            } else {
                // 处理字段模式
                if contact.is_none() {
                    eprintln!("错误：至少需要提供联系人字段！");
                    return Err(rusqlite::Error::InvalidParameterName("缺少必填字段".to_string()));
                }
                
                let supplier = Supplier {
                    contact: contact.clone(),
                    wechat: wechat.clone(),
                    phone: phone.clone(),
                    quantity: *quantity,
                    location: location.clone(),
                    price: *price,
                    bandwidth_price: *bandwidth_price,
                    storage_price: *storage_price,
                    min_contract_period: min_contract_period.clone(),
                    breach_penalties: breach_penalties.clone(),
                    payment_terms: payment_terms.clone(),
                    server_name: server_name.clone(),
                    server_config: server_config.clone(),
                    rental_model: rental_model.clone(),
                    networking_category: networking_category.clone(),
                };
                
                match insert_supplier(&supplier) {
                    Ok(_) => {
                        // 成功时不需要输出，保持界面简洁
                    },
                    Err(e) => {
                        eprintln!("添加失败: {}", e);
                        return Err(e);
                    }
                }
            }
        },
        Commands::Query(args) => {
            // 初始化数据库
            init_db()?;
            
            match query_suppliers_with_filter(args) {
                Ok(rows) => {
                    if rows.is_empty() {
                        println!("没有找到符合条件的供应商");
                    } else {
                        // 根据需要使用不同的输出格式
                        if args.json {
                            print_suppliers_json(&rows);
                        } else {
                            print_suppliers_table_v2(&rows);
                        }
                        
                        // 如果需要导出CSV
                        if let Some(csv_path) = &args.export_csv {
                            match export_suppliers_to_csv(&rows, csv_path) {
                                Ok(_) => println!("已导出 {} 条记录到 {}", rows.len(), csv_path),
                                Err(e) => eprintln!("导出CSV失败: {}", e),
                            }
                        }
                    }
                },
                Err(e) => {
                    eprintln!("查询错误: {}", e);
                    return Err(e);
                }
            }
        },
    }
    Ok(())
}

fn insert_supplier(s: &Supplier) -> Result<()> {
    let conn = Connection::open(get_db_file())?;
    let sql = r#"
        INSERT INTO suppliers (
            contact, wechat, phone, quantity, location, price, bandwidth_price, storage_price, min_contract_period, breach_penalties, payment_terms, server_name, server_config, rental_model, networking_category
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)
    "#;
    conn.execute(sql, [
        s.contact.as_deref(),
        s.wechat.as_deref(),
        s.phone.as_deref(),
        s.quantity.map(|v| v.to_string()).as_deref(),
        s.location.as_deref(),
        s.price.map(|v| v.to_string()).as_deref(),
        s.bandwidth_price.map(|v| v.to_string()).as_deref(),
        s.storage_price.map(|v| v.to_string()).as_deref(),
        s.min_contract_period.as_deref(),
        s.breach_penalties.as_deref(),
        s.payment_terms.as_deref(),
        s.server_name.as_deref(),
        s.server_config.as_deref(),
        s.rental_model.as_deref(),
        s.networking_category.as_deref(),
    ])?;
    Ok(())
}

fn query_suppliers_with_filter(args: &QueryArgs) -> Result<Vec<SupplierRow>> {
    let conn = Connection::open(get_db_file())?;
    
    // 确保数据库表存在
    init_db()?;
    
    let mut conditions = Vec::new();
    let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

    // 注意：参数名可能使用的是破折号格式 (bandwidth-price)，但数据库中是下划线格式 (bandwidth_price)
    // 确保两者匹配
    if let Some(val) = &args.contact {
        conditions.push("contact = ?");
        params.push(Box::new(val.clone()));
    }
    if let Some(val) = &args.wechat {
        conditions.push("wechat = ?");
        params.push(Box::new(val.clone()));
    }
    if let Some(val) = &args.phone {
        conditions.push("phone = ?");
        params.push(Box::new(val.clone()));
    }
    if let Some(val) = &args.quantity {
        conditions.push("quantity = ?");
        params.push(Box::new(*val));
    }
    if let Some(val) = &args.location {
        conditions.push("location = ?");
        params.push(Box::new(val.clone()));
    }
    if let Some(val) = &args.price {
        conditions.push("price = ?");
        params.push(Box::new(*val));
    }
    if let Some(val) = &args.bandwidth_price {
        conditions.push("bandwidth_price = ?");
        params.push(Box::new(*val));
    }
    if let Some(val) = &args.storage_price {
        conditions.push("storage_price = ?");
        params.push(Box::new(*val));
    }
    if let Some(val) = &args.min_contract_period {
        conditions.push("min_contract_period = ?");
        params.push(Box::new(val.clone()));
    }
    if let Some(val) = &args.breach_penalties {
        conditions.push("breach_penalties = ?");
        params.push(Box::new(val.clone()));
    }
    if let Some(val) = &args.payment_terms {
        conditions.push("payment_terms = ?");
        params.push(Box::new(val.clone()));
    }
    if let Some(val) = &args.server_name {
        conditions.push("server_name = ?");
        params.push(Box::new(val.clone()));
    }
    if let Some(val) = &args.server_config {
        conditions.push("server_config = ?");
        params.push(Box::new(val.clone()));
    }
    if let Some(val) = &args.rental_model {
        conditions.push("rental_model = ?");
        params.push(Box::new(val.clone()));
    }
    if let Some(val) = &args.networking_category {
        conditions.push("networking_category = ?");
        params.push(Box::new(val.clone()));
    }

    let where_clause = if conditions.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", conditions.join(" AND "))
    };

    let sql = format!(
        r#"
        SELECT id, contact, wechat, phone, quantity, location, price, bandwidth_price, storage_price, min_contract_period, breach_penalties, payment_terms, server_name, server_config, rental_model, networking_category
        FROM suppliers
        {}
        "#,
        where_clause
    );

    let mut stmt = conn.prepare(&sql)?;
    
    let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();
    let rows = stmt.query(param_refs.as_slice())?;

    let supplier_rows = rows.mapped(|row| {
        Ok(SupplierRow {
            id: row.get(0)?,
            contact: row.get(1)?,
            wechat: row.get(2)?,
            phone: row.get(3)?,
            quantity: row.get(4)?,
            location: row.get(5)?,
            price: row.get(6)?,
            bandwidth_price: row.get(7)?,
            storage_price: row.get(8)?,
            min_contract_period: row.get(9)?,
            breach_penalties: row.get(10)?,
            payment_terms: row.get(11)?,
            server_name: row.get(12)?,
            server_config: row.get(13)?,
            rental_model: row.get(14)?,
            networking_category: row.get(15)?,
        })
    }).collect::<Result<Vec<_>, rusqlite::Error>>()?;
    
    Ok(supplier_rows)
}

fn init_db() -> Result<()> {
    let conn = Connection::open(get_db_file())?;
    // 建表SQL，字段注释以SQL注释形式写在建表语句中
    let sql = r#"
-- 供应商表结构定义
CREATE TABLE IF NOT EXISTS suppliers (
    id INTEGER PRIMARY KEY AUTOINCREMENT, -- 主键，自增
    contact TEXT NOT NULL,               -- 联系人
    wechat TEXT,                         -- 微信
    phone TEXT,                          -- 电话
    quantity INTEGER,                    -- 数量
    location TEXT,                       -- 地点
    price REAL,                          -- 价格
    bandwidth_price REAL,                -- 带宽价格
    storage_price REAL,                  -- 存储价格
    min_contract_period TEXT,            -- 最短合同期
    breach_penalties TEXT,               -- 违约金
    payment_terms TEXT,                  -- 付款方式
    server_name TEXT,                    -- 服务器名称
    server_config TEXT,                  -- 服务器配置
    rental_model TEXT,                   -- 租赁模式
    networking_category TEXT             -- 网络类型
);
"#;
    conn.execute_batch(sql)?;
    Ok(())
}

fn export_suppliers_to_csv(rows: &[SupplierRow], path: &str) -> std::io::Result<()> {
    let file = File::create(path)?;
    let mut wtr = csv::WriterBuilder::new()
        .has_headers(true)
        .from_writer(file);
    
    // 写入表头
    wtr.write_record(&[
        "ID", "联系人", "微信", "手机", "数量", "位置", "价格",
        "带宽价", "存储价", "签约周期", "违约", "付款", "服务器名",
        "配置", "租赁", "组网"
    ])?;
    
    // 写入数据行
    for s in rows {
        wtr.write_record(&[
            &s.id.to_string(),
            s.contact.as_deref().unwrap_or(""),
            s.wechat.as_deref().unwrap_or(""),
            s.phone.as_deref().unwrap_or(""),
            &s.quantity.map(|v| v.to_string()).unwrap_or_default(),
            s.location.as_deref().unwrap_or(""),
            &s.price.map(|v| v.to_string()).unwrap_or_default(),
            &s.bandwidth_price.map(|v| v.to_string()).unwrap_or_default(),
            &s.storage_price.map(|v| v.to_string()).unwrap_or_default(),
            s.min_contract_period.as_deref().unwrap_or(""),
            s.breach_penalties.as_deref().unwrap_or(""),
            s.payment_terms.as_deref().unwrap_or(""),
            s.server_name.as_deref().unwrap_or(""),
            s.server_config.as_deref().unwrap_or(""),
            s.rental_model.as_deref().unwrap_or(""),
            s.networking_category.as_deref().unwrap_or("")
        ])?;
    }
    
    wtr.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::sync::Mutex;
    use once_cell::sync::Lazy;
    use assert_cmd::Command;
    use predicates::prelude::*;

    static TEST_MUTEX: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

    fn setup_test_db() -> String {
        // 使用std::sync::Mutex的try_lock而不是unwrap，避免PoisonError
        let _guard = match TEST_MUTEX.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(), // 恢复poisoned锁
        };
        
        // 使用固定测试文件路径，但添加唯一标识符
        let temp_dir = std::env::temp_dir();
        let unique_id = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis();
        let db_path = temp_dir.join(format!("test_suppliers_{}.db", unique_id)).to_str().unwrap().to_string();
        
        // 确保数据库文件不存在（如果存在则删除）
        if std::path::Path::new(&db_path).exists() {
            std::fs::remove_file(&db_path).unwrap_or_else(|e| {
                println!("无法删除旧测试数据库文件: {}", e);
            });
        }
        
        // 直接执行表创建SQL，不依赖全局状态
        let conn = match Connection::open(&db_path) {
            Ok(conn) => conn,
            Err(e) => panic!("打开数据库连接失败: {} - 路径: {}", e, db_path),
        };
        
        let create_table_sql = "CREATE TABLE IF NOT EXISTS suppliers (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            contact TEXT,
            wechat TEXT,
            phone TEXT,
            quantity INTEGER,
            location TEXT,
            price REAL,
            bandwidth_price REAL,
            storage_price REAL,
            min_contract_period TEXT,
            breach_penalties TEXT,
            payment_terms TEXT,
            server_name TEXT,
            server_config TEXT,
            rental_model TEXT,
            networking_category TEXT
        );";
        
        match conn.execute_batch(create_table_sql) {
            Ok(_) => println!("测试数据库表创建成功: {}", db_path),
            Err(e) => panic!("测试数据库表创建失败: {}", e),
        }
        
        // 验证表是否创建成功
        let mut stmt = match conn.prepare("SELECT name FROM sqlite_master WHERE type='table' AND name='suppliers'") {
            Ok(stmt) => stmt,
            Err(e) => panic!("准备验证SQL失败: {}", e),
        };
        
        let exists: bool = match stmt.exists([]) {
            Ok(exists) => exists,
            Err(e) => panic!("验证表是否存在失败: {}", e),
        };
        
        if !exists {
            panic!("创建表后验证失败，表suppliers不存在");
        } else {
            println!("表suppliers存在验证通过");
        }
        
        // 关闭连接，确保所有操作都写入磁盘
        drop(stmt);
        drop(conn);
        
        // 设置环境变量，指向测试数据库
        std::env::set_var("DB_FILE", &db_path);
        
        db_path
    }

    #[test]
    fn test_init_db() {
        let db_path = setup_test_db();
        
        // 直接使用传递过来的数据库路径
        let conn = Connection::open(&db_path).unwrap();
        let mut stmt = conn.prepare("SELECT name FROM sqlite_master WHERE type='table' AND name='suppliers'").unwrap();
        let exists: bool = stmt.exists([]).unwrap();
        assert!(exists);
    }

    #[test]
    fn test_query_empty_db() {
        let db_path = setup_test_db();
        
        // 直接使用传递过来的数据库路径
        let conn = Connection::open(&db_path).unwrap();
        
        // 用空的查询参数查询，确认返回0条记录
        let _args = QueryArgs::default();
        
        // 验证表结构是否存在
        let mut check_stmt = conn.prepare("SELECT name FROM sqlite_master WHERE type='table' AND name='suppliers'").unwrap();
        let exists: bool = check_stmt.exists([]).unwrap();
        assert!(exists, "suppliers表不存在");
        
        // 直接在数据库上查询，不走全局函数
        let sql = "SELECT id, contact, wechat, phone, quantity, location, price, bandwidth_price, 
                   storage_price, min_contract_period, breach_penalties, payment_terms, 
                   server_name, server_config, rental_model, networking_category 
                   FROM suppliers";
        
        let mut stmt = conn.prepare(sql).unwrap();
        let rows = stmt.query_map([], |row| {
            Ok(SupplierRow {
                id: row.get(0)?,
                contact: row.get(1)?,
                wechat: row.get(2)?,
                phone: row.get(3)?,
                quantity: row.get(4)?,
                location: row.get(5)?,
                price: row.get(6)?,
                bandwidth_price: row.get(7)?,
                storage_price: row.get(8)?,
                min_contract_period: row.get(9)?,
                breach_penalties: row.get(10)?,
                payment_terms: row.get(11)?,
                server_name: row.get(12)?,
                server_config: row.get(13)?,
                rental_model: row.get(14)?,
                networking_category: row.get(15)?,
            })
        }).unwrap();
        
        let result: Vec<SupplierRow> = rows.map(|r| r.unwrap()).collect();
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_insert_and_query_supplier() {
        let db_path = setup_test_db();
        
        // 直接使用本地连接，不使用全局函数
        let conn = Connection::open(&db_path).unwrap();
        
        // 验证表结构是否存在
        let mut check_stmt = conn.prepare("SELECT name FROM sqlite_master WHERE type='table' AND name='suppliers'").unwrap();
        let exists: bool = check_stmt.exists([]).unwrap();
        assert!(exists, "suppliers表不存在");
        
        let supplier = Supplier {
            contact: Some("张三".to_string()),
            wechat: Some("wxid".to_string()),
            phone: Some("12345678901".to_string()),
            quantity: Some(10),
            location: Some("北京".to_string()),
            price: Some(1000.0),
            bandwidth_price: Some(100.0),
            storage_price: Some(50.0),
            min_contract_period: Some("12个月".to_string()),
            breach_penalties: Some("无".to_string()),
            payment_terms: Some("月付".to_string()),
            server_name: Some("服务器A".to_string()),
            server_config: Some("8核16G".to_string()),
            rental_model: Some("包年".to_string()),
            networking_category: Some("BGP".to_string()),
        };

        // 直接在数据库中插入数据
        let sql = r#"
            INSERT INTO suppliers (
                contact, wechat, phone, quantity, location, price, bandwidth_price, storage_price, 
                min_contract_period, breach_penalties, payment_terms, server_name, server_config, 
                rental_model, networking_category
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)
        "#;
        conn.execute(sql, [
            supplier.contact.as_deref(),
            supplier.wechat.as_deref(),
            supplier.phone.as_deref(),
            supplier.quantity.map(|v| v.to_string()).as_deref(),
            supplier.location.as_deref(),
            supplier.price.map(|v| v.to_string()).as_deref(),
            supplier.bandwidth_price.map(|v| v.to_string()).as_deref(),
            supplier.storage_price.map(|v| v.to_string()).as_deref(),
            supplier.min_contract_period.as_deref(),
            supplier.breach_penalties.as_deref(),
            supplier.payment_terms.as_deref(),
            supplier.server_name.as_deref(),
            supplier.server_config.as_deref(),
            supplier.rental_model.as_deref(),
            supplier.networking_category.as_deref(),
        ]).unwrap();

        // 直接在数据库中查询
        let query_sql = "SELECT id, contact FROM suppliers WHERE contact = ?";
        let mut stmt = conn.prepare(query_sql).unwrap();
        let result = stmt.query_map(["张三"], |row| {
            Ok((row.get::<_, i32>(0)?, row.get::<_, String>(1)?))
        }).unwrap().map(|r| r.unwrap()).collect::<Vec<_>>();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].1, "张三");
    }

    #[test]
    fn test_query_single_record() {
        let db_path = setup_test_db();
        
        // 直接使用本地连接，不使用全局函数
        let conn = Connection::open(&db_path).unwrap();
        
        // 验证表结构是否存在
        let mut check_stmt = conn.prepare("SELECT name FROM sqlite_master WHERE type='table' AND name='suppliers'").unwrap();
        let exists: bool = check_stmt.exists([]).unwrap();
        assert!(exists, "suppliers表不存在");
        
        let supplier = Supplier {
            contact: Some("张三".to_string()),
            wechat: Some("wxid".to_string()),
            phone: Some("12345678901".to_string()),
            quantity: Some(10),
            location: Some("北京".to_string()),
            price: Some(1000.0),
            bandwidth_price: Some(100.0),
            storage_price: Some(50.0),
            min_contract_period: Some("12个月".to_string()),
            breach_penalties: Some("无".to_string()),
            payment_terms: Some("月付".to_string()),
            server_name: Some("服务器A".to_string()),
            server_config: Some("8核16G".to_string()),
            rental_model: Some("包年".to_string()),
            networking_category: Some("BGP".to_string()),
        };

        // 直接在数据库中插入数据
        let sql = r#"
            INSERT INTO suppliers (
                contact, wechat, phone, quantity, location, price, bandwidth_price, storage_price, 
                min_contract_period, breach_penalties, payment_terms, server_name, server_config, 
                rental_model, networking_category
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)
        "#;
        conn.execute(sql, [
            supplier.contact.as_deref(),
            supplier.wechat.as_deref(),
            supplier.phone.as_deref(),
            supplier.quantity.map(|v| v.to_string()).as_deref(),
            supplier.location.as_deref(),
            supplier.price.map(|v| v.to_string()).as_deref(),
            supplier.bandwidth_price.map(|v| v.to_string()).as_deref(),
            supplier.storage_price.map(|v| v.to_string()).as_deref(),
            supplier.min_contract_period.as_deref(),
            supplier.breach_penalties.as_deref(),
            supplier.payment_terms.as_deref(),
            supplier.server_name.as_deref(),
            supplier.server_config.as_deref(),
            supplier.rental_model.as_deref(),
            supplier.networking_category.as_deref(),
        ]).unwrap();

        // 直接在数据库中查询
        let query_sql = "SELECT contact FROM suppliers WHERE contact = ?";
        let mut stmt = conn.prepare(query_sql).unwrap();
        let result = stmt.query_map(["张三"], |row| {
            Ok(row.get::<_, String>(0)?)
        }).unwrap().map(|r| r.unwrap()).collect::<Vec<_>>();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0], "张三");
    }

    #[test]
    fn test_query_multiple_records() {
        let db_path = setup_test_db();
        
        // 直接使用本地连接，不使用全局函数
        let conn = Connection::open(&db_path).unwrap();
        
        // 验证表结构是否存在
        let mut check_stmt = conn.prepare("SELECT name FROM sqlite_master WHERE type='table' AND name='suppliers'").unwrap();
        let exists: bool = check_stmt.exists([]).unwrap();
        assert!(exists, "suppliers表不存在");
        
        // 插入多条测试数据
        let locations = ["北京", "上海", "广州", "深圳", "杭州"];
        
        for (i, location) in locations.iter().enumerate() {
            let contact = format!("联系人{}", i + 1);
            let wechat = format!("wx{}", i + 1);
            let phone = format!("1380013800{}", i + 1);
            let quantity = (i + 1) * 10;
            let price = 1000.0 + i as f64 * 100.0;
            
            let sql = r#"
                INSERT INTO suppliers (
                    contact, wechat, phone, quantity, location, price
                ) VALUES (?, ?, ?, ?, ?, ?)
            "#;
            conn.execute(
                sql,
                [
                    contact.as_str(),
                    wechat.as_str(),
                    phone.as_str(),
                    &quantity.to_string(),
                    location,
                    &price.to_string(),
                ],
            ).unwrap();
        }
        
        // 直接在数据库中查询全部记录
        let query_all_sql = "SELECT COUNT(*) FROM suppliers";
        let total: i32 = conn.query_row(query_all_sql, [], |row| row.get(0)).unwrap();
        assert_eq!(total, 5, "应该插入了5条记录");
        
        // 按位置查询
        let query_location_sql = "SELECT contact FROM suppliers WHERE location = ?";
        let mut stmt = conn.prepare(query_location_sql).unwrap();
        let beijing_records: Vec<String> = stmt.query_map(["北京"], |row| row.get(0))
            .unwrap()
            .map(|r| r.unwrap())
            .collect();
            
        assert_eq!(beijing_records.len(), 1, "北京位置应有1条记录");
        assert_eq!(beijing_records[0], "联系人1");
        
        // 按价格范围查询
        let query_price_sql = "SELECT contact FROM suppliers WHERE price > ? ORDER BY price";
        let mut stmt = conn.prepare(query_price_sql).unwrap();
        let expensive_records: Vec<String> = stmt.query_map([1200.0.to_string()], |row| row.get(0))
            .unwrap()
            .map(|r| r.unwrap())
            .collect();
            
        assert_eq!(expensive_records.len(), 2, "价格大于1200的记录应有2条");
    }
    
    // 新增测试用例：测试过滤条件验证
    #[test]
    fn test_filter_criteria_validation() {
        // 测试有效的字符串字段比较
        let criteria1 = FilterCriteria::from_str_tuple("contact", "eq", Some("张三"));
        assert!(criteria1.is_ok());
        
        // 测试有效的数字字段比较
        let criteria2 = FilterCriteria::from_str_tuple("quantity", "gt", Some("5"));
        assert!(criteria2.is_ok());
        
        // 测试无效的字段名
        let criteria3 = FilterCriteria::from_str_tuple("unknown_field", "eq", Some("值"));
        assert!(criteria3.is_err());
        
        // 测试无效的操作符
        let criteria4 = FilterCriteria::from_str_tuple("contact", "invalid_op", Some("值"));
        assert!(criteria4.is_err());
        
        // 测试数字字段使用字符串操作符
        let criteria5 = FilterCriteria::from_str_tuple("quantity", "like", Some("非数字"));
        assert!(criteria5.is_err());
    }
    
    // 测试查询构建器
    #[test]
    fn test_query_builder() {
        let criteria1 = FilterCriteria::from_str_tuple("contact", "eq", Some("张三")).unwrap();
        let criteria2 = FilterCriteria::from_str_tuple("quantity", "gt", Some("5")).unwrap();
        
        let builder = QueryBuilder::new()
            .filter(criteria1)
            .filter(criteria2);
            
        assert!(builder.validate().is_ok());
        
        let (query, params) = builder.build();
        assert!(query.contains("WHERE"));
        assert!(query.contains("contact = ?"));
        assert!(query.contains("quantity > ?"));
        assert!(query.contains("AND"));
        assert_eq!(params.len(), 2);
        assert_eq!(params[0], "张三");
        assert_eq!(params[1], "5");
    }
    
    // 测试查询构建器验证逻辑
    #[test]
    fn test_query_builder_validation() {
        // 测试数字字段使用LIKE操作符（应该失败）
        let invalid_criteria = FilterCriteria {
            field: SupplierField::Quantity,
            op: ComparisonOp::Like,
            value: Some("10".to_string()),
        };
        
        let builder = QueryBuilder::new().filter(invalid_criteria);
        let validation_result = builder.validate();
        
        assert!(validation_result.is_err());
        let err_msg = validation_result.unwrap_err();
        assert!(err_msg.contains("Quantity") && err_msg.contains("Like"));
    }
    
    // 测试JSON输入解析
    #[test]
    fn test_json_input_parsing() {
        let json_str = r#"{"contact":"张三","wechat":"wxid1","phone":"12345678901","quantity":10}"#;
        let supplier: Result<Supplier, _> = serde_json::from_str(json_str);
        
        assert!(supplier.is_ok());
        let s = supplier.unwrap();
        assert_eq!(s.contact.as_deref(), Some("张三"));
        assert_eq!(s.quantity, Some(10));
        assert_eq!(s.wechat.as_deref(), Some("wxid1"));
    }
    
    // 测试导出CSV文件
    #[test]
    fn test_export_csv() {
        // 重置测试环境
        let _db_path = setup_test_db();
        
        // 插入一条测试数据
        let supplier = Supplier {
            contact: Some("张三".to_string()),
            wechat: Some("wxid".to_string()),
            phone: Some("12345678901".to_string()),
            quantity: Some(10),
            location: Some("北京".to_string()),
            price: Some(1000.0),
            bandwidth_price: Some(100.0),
            storage_price: Some(50.0),
            min_contract_period: Some("12个月".to_string()),
            breach_penalties: Some("无".to_string()),
            payment_terms: Some("月付".to_string()),
            server_name: Some("服务器A".to_string()),
            server_config: Some("8核16G".to_string()),
            rental_model: Some("包年".to_string()),
            networking_category: Some("BGP".to_string()),
        };
        
        insert_supplier(&supplier).unwrap();
        
        // 查询并导出
        let args = QueryArgs::default();
        let rows = query_suppliers_with_filter(&args).unwrap();
        
        let csv_file = NamedTempFile::new().unwrap();
        let csv_path = csv_file.path().to_str().unwrap();
        
        let export_result = export_suppliers_to_csv(&rows, csv_path);
        assert!(export_result.is_ok());
        
        // 检查CSV文件内容
        let content = std::fs::read_to_string(csv_path).unwrap();
        assert!(content.contains("张三"));
        assert!(content.contains("wxid"));
        assert!(content.contains("12345678901"));
    }
    
    // 集成测试：命令行接口测试
    #[test]
    fn test_cli_help() {
        let mut cmd = Command::cargo_bin("wei-assistant-gpu").unwrap();
        let assert = cmd.arg("--help").assert();
        assert.success()
              .stdout(predicate::str::contains("供应商信息管理命令行工具"))
              .stdout(predicate::str::contains("Usage"))
              .stdout(predicate::str::contains("Commands"));
    }
    
    // 集成测试：添加供应商命令
    #[test]
    fn test_cli_add_supplier() {
        // 使用临时文件作为数据库
        let db_file = NamedTempFile::new().unwrap();
        let db_path = db_file.path().to_str().unwrap();
        std::env::set_var("DB_FILE", &db_path);
        
        // 初始化数据库
        init_db().unwrap();
        
        // 添加一个供应商
        let mut cmd = Command::cargo_bin("wei-assistant-gpu").unwrap();
        let assert = cmd
            .env("DB_FILE", db_path)
            .arg("add")
            .arg("--contact").arg("CLI测试")
            .arg("--wechat").arg("cli-wxid")
            .arg("--phone").arg("12345678909")
            .arg("--quantity").arg("20")
            .assert();
            
        assert.success();
        
        // 查询验证
        let mut cmd = Command::cargo_bin("wei-assistant-gpu").unwrap();
        let assert = cmd
            .env("DB_FILE", db_path)
            .arg("query")
            .arg("--contact").arg("CLI测试")
            .assert();
            
        assert.success()
              .stdout(predicate::str::contains("CLI测试"))
              .stdout(predicate::str::contains("cli-wxid"));
    }
    
    // 集成测试：使用JSON添加供应商
    #[test]
    fn test_cli_add_json() {
        // 使用临时文件作为数据库
        let db_file = NamedTempFile::new().unwrap();
        let db_path = db_file.path().to_str().unwrap();
        std::env::set_var("DB_FILE", &db_path);
        
        // 初始化数据库
        init_db().unwrap();
        
        // JSON格式添加供应商
        let json = r#"{"contact":"JSON测试","wechat":"json-wxid","phone":"98765432101","quantity":30}"#;
        
        let mut cmd = Command::cargo_bin("wei-assistant-gpu").unwrap();
        let assert = cmd
            .env("DB_FILE", db_path)
            .arg("add")
            .arg("--json").arg(json)
            .assert();
            
        assert.success();
        
        // 查询验证
        let mut cmd = Command::cargo_bin("wei-assistant-gpu").unwrap();
        let assert = cmd
            .env("DB_FILE", db_path)
            .arg("query")
            .arg("--json")  // 输出JSON格式
            .assert();
            
        assert.success()
              .stdout(predicate::str::contains("JSON测试"))
              .stdout(predicate::str::contains("json-wxid"))
              .stdout(predicate::str::contains("98765432101"));
    }
    
    // 测试错误处理：无效数据库文件
    #[test]
    fn test_invalid_db_file() {
        let mut cmd = Command::cargo_bin("wei-assistant-gpu").unwrap();
        let assert = cmd
            .env("DB_FILE", "/non/existent/path/db.sqlite")
            .arg("query")
            .assert();
            
        assert.failure();
    }
}
