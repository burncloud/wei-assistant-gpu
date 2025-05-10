use std::time::{Instant, Duration};
use rusqlite::{Connection, Result as RusqliteResult};
use std::sync::Mutex;
use once_cell::sync::Lazy;
use std::env;
use std::path::Path;

// 获取正确的二进制名称
fn get_bin_name() -> String {
    let manifest = env::var("CARGO_PKG_NAME").unwrap_or_else(|_| "wei-assistant-gpu".to_string());
    manifest
}

// 添加互斥锁确保性能测试不会与其他测试冲突
static TEST_MUTEX: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

#[derive(Debug)]
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

#[derive(Debug)]
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

// 创建测试数据库
fn setup_test_db() -> String {
    // 获取锁，确保互斥访问
    let _guard = match TEST_MUTEX.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(), // 恢复poisoned锁
    };
    
    // 使用固定测试文件路径但添加时间戳确保唯一性
    let temp_dir = env::temp_dir();
    let unique_id = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let db_path = temp_dir.join(format!("test_perf_{}.db", unique_id)).to_str().unwrap().to_string();
    
    // 确保不存在同名文件
    if Path::new(&db_path).exists() {
        std::fs::remove_file(&db_path).unwrap_or_else(|e| {
            println!("无法删除旧测试数据库文件: {}", e);
        });
    }
    
    // 初始化数据库
    let conn = Connection::open(&db_path).unwrap();
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
    conn.execute_batch(create_table_sql).unwrap();
    
    // 关闭连接确保写入
    drop(conn);
    
    // 设置环境变量
    env::set_var("DB_FILE", &db_path);
    
    db_path
}

// 批量插入供应商数据
fn batch_insert_suppliers(db_path: &str, count: usize) -> RusqliteResult<Duration> {
    let conn = Connection::open(db_path)?;
    
    // 启动计时器
    let start = Instant::now();
    
    // 开始事务以提高性能
    conn.execute("BEGIN TRANSACTION", [])?;
    
    let sql = r#"
    INSERT INTO suppliers (
        contact, wechat, phone, quantity, location, price, 
        bandwidth_price, storage_price, min_contract_period, 
        breach_penalties, payment_terms, server_name, 
        server_config, rental_model, networking_category
    ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
    "#;
    
    let mut stmt = conn.prepare(sql)?;
    
    for i in 1..=count {
        let location = match i % 5 {
            0 => "北京",
            1 => "上海", 
            2 => "广州",
            3 => "深圳",
            _ => "杭州",
        };
        
        let rental_model = match i % 3 {
            0 => "包年",
            1 => "包月",
            _ => "按量付费",
        };
        
        stmt.execute(&[
            &format!("供应商{}", i) as &dyn rusqlite::ToSql,
            &format!("wx{}", i) as &dyn rusqlite::ToSql,
            &format!("1380013{:04}", i) as &dyn rusqlite::ToSql,
            &(i % 100) as &dyn rusqlite::ToSql,
            &location as &dyn rusqlite::ToSql,
            &(800.0 + (i as f64 * 10.0) % 1000.0) as &dyn rusqlite::ToSql,
            &(50.0 + (i as f64) % 100.0) as &dyn rusqlite::ToSql,
            &(20.0 + (i as f64) % 50.0) as &dyn rusqlite::ToSql,
            &format!("{}个月", (i % 12) + 1) as &dyn rusqlite::ToSql,
            &"无" as &dyn rusqlite::ToSql,
            &"按月付费" as &dyn rusqlite::ToSql,
            &format!("服务器{}", i) as &dyn rusqlite::ToSql,
            &format!("{}核{}G", (i % 32) + 2, (i % 64) + 4) as &dyn rusqlite::ToSql,
            &rental_model as &dyn rusqlite::ToSql,
            &(if i % 2 == 0 { "BGP" } else { "专线" }) as &dyn rusqlite::ToSql,
        ])?;
    }
    
    // 提交事务
    conn.execute("COMMIT", [])?;
    
    // 计算耗时
    let duration = start.elapsed();
    
    Ok(duration)
}

// 测试查询性能
fn test_query_performance(db_path: &str) -> RusqliteResult<(Duration, usize)> {
    let conn = Connection::open(db_path)?;
    
    // 启动计时器
    let start = Instant::now();
    
    // 执行查询
    let mut stmt = conn.prepare("SELECT * FROM suppliers WHERE location = ?")?;
    let mut rows = stmt.query(&[&"北京" as &dyn rusqlite::ToSql])?;
    
    // 收集结果
    let mut results = Vec::new();
    while let Some(row) = rows.next()? {
        results.push(SupplierRow {
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
        });
    }
    let result_count = results.len();
    
    // 计算耗时
    let duration = start.elapsed();
    
    Ok((duration, result_count))
}

// 测试复杂查询性能
fn test_complex_query_performance(db_path: &str) -> RusqliteResult<(Duration, usize)> {
    let conn = Connection::open(db_path)?;
    
    // 启动计时器
    let start = Instant::now();
    
    // 执行复杂查询（多条件，排序等）
    let mut stmt = conn.prepare("
        SELECT * FROM suppliers 
        WHERE price > ? AND rental_model = ? 
        ORDER BY price DESC 
        LIMIT 100
    ")?;
    
    let price_param = 500.0;
    let model_param = "包年";
    
    let mut rows = stmt.query(&[&price_param as &dyn rusqlite::ToSql, 
                               &model_param as &dyn rusqlite::ToSql])?;
    
    // 收集结果
    let mut results = Vec::new();
    while let Some(row) = rows.next()? {
        results.push(SupplierRow {
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
        });
    }
    let result_count = results.len();
    
    // 计算耗时
    let duration = start.elapsed();
    
    Ok((duration, result_count))
}

// 主要性能测试
#[test]
fn performance_test() {
    // 只有在特殊环境下才运行性能测试，避免在正常测试中运行耗时测试
    if std::env::var("RUN_PERFORMANCE_TESTS").is_err() {
        println!("跳过性能测试。设置 RUN_PERFORMANCE_TESTS=1 环境变量以启用");
        return;
    }
    
    // 设置测试数据库
    let db_path = setup_test_db();
    
    // 测试数据量
    let record_count = 1000;  // 实际测试可以调整为更大的值
    
    // 测试批量插入性能
    match batch_insert_suppliers(&db_path, record_count) {
        Ok(duration) => {
            println!(
                "性能测试: 插入 {} 条记录耗时: {:.2?}，平均每条: {:.2?}",
                record_count,
                duration,
                duration / record_count as u32
            );
        },
        Err(e) => panic!("批量插入测试失败: {}", e),
    }
    
    // 测试基本查询性能
    match test_query_performance(&db_path) {
        Ok((duration, count)) => {
            println!(
                "性能测试: 查询 '北京' 位置的 {} 条记录耗时: {:.2?}",
                count,
                duration
            );
        },
        Err(e) => panic!("查询性能测试失败: {}", e),
    }
    
    // 测试复杂查询性能
    match test_complex_query_performance(&db_path) {
        Ok((duration, count)) => {
            println!(
                "性能测试: 复杂查询（价格>500且包年）找到 {} 条记录耗时: {:.2?}",
                count,
                duration
            );
        },
        Err(e) => panic!("复杂查询性能测试失败: {}", e),
    }
} 