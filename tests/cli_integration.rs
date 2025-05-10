use assert_cmd::Command;
use predicates::prelude::*;
use std::io;
use rusqlite::Connection;
use std::env;
use std::path::Path;
use std::sync::Mutex;
use once_cell::sync::Lazy;
use rand;

// 测试互斥锁，确保不同测试不会同时操作同一个数据库文件
static TEST_MUTEX: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

// 获取正确的二进制名称
fn get_bin_name() -> String {
    let manifest = env::var("CARGO_PKG_NAME").unwrap_or_else(|_| "wei-assistant-gpu".to_string());
    manifest
}

// 辅助函数：创建测试数据库文件
fn create_test_db() -> io::Result<String> {
    // 获取锁确保测试互斥
    let _guard = match TEST_MUTEX.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(), // 恢复poisoned锁
    };
    
    // 使用固定测试文件路径但加上唯一标识
    let temp_dir = env::temp_dir();
    let unique_id = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let db_path = temp_dir.join(format!("test_cli_{}.db", unique_id)).to_str().unwrap().to_string();
    
    // 确保不存在同名文件
    if Path::new(&db_path).exists() {
        std::fs::remove_file(&db_path)?;
    }
    
    // 初始化数据库表结构
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
    
    Ok(db_path)
}

#[test]
fn test_cli_help() {
    let mut cmd = Command::cargo_bin(&get_bin_name()).unwrap();
    let assert = cmd.arg("--help").assert();
    assert.success()
          .stdout(predicate::str::contains("供应商信息管理"))
          .stdout(predicate::str::contains("Usage"))
          .stdout(predicate::str::contains("Commands"));
}

#[test]
fn test_add_command_help() {
    let mut cmd = Command::cargo_bin(&get_bin_name()).unwrap();
    cmd.arg("add").arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("添加供应商信息"))
        .stdout(predicate::str::contains("--json"))
        .stdout(predicate::str::contains("--contact"));
}

#[test]
fn test_query_command_help() {
    let mut cmd = Command::cargo_bin(&get_bin_name()).unwrap();
    cmd.arg("query").arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("查询所有供应商信息"))
        .stdout(predicate::str::contains("--json"))
        .stdout(predicate::str::contains("--export-csv"));
}

#[test]
fn test_add_and_query_complete_workflow() {
    // 创建测试数据库
    let db_path = create_test_db().unwrap();
    
    // 初始化数据库表结构
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
    drop(conn);
    
    // 添加供应商
    let mut cmd = Command::cargo_bin(&get_bin_name()).unwrap();
    cmd.env("DB_FILE", &db_path)
        .arg("add")
        .arg("--contact").arg("张三")
        .arg("--wechat").arg("wx123")
        .arg("--phone").arg("13800138000")
        .arg("--location").arg("北京")
        .assert()
        .success();
    
    // 查询供应商
    let mut cmd = Command::cargo_bin(&get_bin_name()).unwrap();
    cmd.env("DB_FILE", &db_path)
        .arg("query")
        .arg("--contact").arg("张三")
        .assert()
        .success()
        .stdout(predicate::str::contains("张三"))
        .stdout(predicate::str::contains("wx123"))
        .stdout(predicate::str::contains("13800138000"));
}

#[test]
fn test_json_add_workflow() {
    // 创建测试数据库
    let db_path = create_test_db().unwrap();
    
    // 初始化数据库表结构
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
    drop(conn);
    
    // 准备JSON数据
    let json = r#"{
        "contact": "JSON测试",
        "wechat": "wxjson",
        "phone": "13900139000",
        "quantity": 50,
        "location": "上海"
    }"#;
    
    // 添加供应商
    let mut cmd = Command::cargo_bin(&get_bin_name()).unwrap();
    cmd.env("DB_FILE", &db_path)
        .arg("add")
        .arg("--json").arg(json)
        .assert()
        .success();
    
    // 查询供应商
    let mut cmd = Command::cargo_bin(&get_bin_name()).unwrap();
    cmd.env("DB_FILE", &db_path)
        .arg("query")
        .arg("--json")
        .assert()
        .success()
        .stdout(predicate::str::contains("JSON测试"))
        .stdout(predicate::str::contains("wxjson"))
        .stdout(predicate::str::contains("13900139000"));
}

#[test]
fn test_multiple_suppliers() {
    // 创建测试数据库
    let db_path = create_test_db().unwrap();
    
    // 初始化数据库表结构
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
    drop(conn);
    
    // 添加多个供应商
    for i in 1..=3 {
        let contact = format!("测试{}", i);
        let wechat = format!("wx{}", i);
        let phone = format!("1380013800{}", i);
        
        let mut cmd = Command::cargo_bin(&get_bin_name()).unwrap();
        cmd.env("DB_FILE", &db_path)
            .arg("add")
            .arg("--contact").arg(&contact)
            .arg("--wechat").arg(&wechat)
            .arg("--phone").arg(&phone)
            .assert()
            .success();
    }
    
    // 查询所有供应商
    let mut cmd = Command::cargo_bin(&get_bin_name()).unwrap();
    cmd.env("DB_FILE", &db_path)
        .arg("query")
        .assert()
        .success()
        .stdout(predicate::str::contains("测试1"))
        .stdout(predicate::str::contains("测试2"))
        .stdout(predicate::str::contains("测试3"));
}

#[test]
fn test_invalid_args() {
    // 测试添加供应商时缺少必要参数
    let db_path = create_test_db().unwrap();
    
    // 初始化数据库表结构
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
    drop(conn);
    
    let mut cmd = Command::cargo_bin(&get_bin_name()).unwrap();
    cmd.env("DB_FILE", &db_path)
        .arg("add")
        .assert()
        .failure();
}

#[test]
fn test_error_handling() {
    // 测试不存在的命令
    let mut cmd = Command::cargo_bin(&get_bin_name()).unwrap();
    cmd.arg("not-exist-command")
        .assert()
        .failure();
    
    // 测试添加供应商时缺少必要参数
    let db_path = create_test_db().unwrap();
    let mut cmd = Command::cargo_bin(&get_bin_name()).unwrap();
    cmd.env("DB_FILE", &db_path)
        .arg("add") // 没有提供任何参数
        .assert()
        .failure();
    
    // 测试JSON格式错误
    let mut cmd = Command::cargo_bin(&get_bin_name()).unwrap();
    cmd.env("DB_FILE", &db_path)
        .arg("add")
        .arg("--json").arg("{invalid json}")
        .assert()
        .failure();
}

#[test]
fn test_invalid_db_file() {
    // 创建一个不可能存在的路径
    let unique_id = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let invalid_path = format!("/non/existent/path/db_{}_{}.sqlite", unique_id, rand::random::<u32>());
    
    let mut cmd = Command::cargo_bin(&get_bin_name()).unwrap();
    let assert = cmd
        .env("DB_FILE", &invalid_path)
        .arg("query")
        .assert();
        
    assert.failure();
} 