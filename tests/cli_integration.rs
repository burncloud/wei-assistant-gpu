use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::NamedTempFile;
use std::io;
use rusqlite::Connection;
use std::env;

// 获取正确的二进制名称
fn get_bin_name() -> String {
    let manifest = env::var("CARGO_PKG_NAME").unwrap_or_else(|_| "wei-assistant-gpu".to_string());
    manifest
}

// 辅助函数：创建测试数据库文件
fn create_test_db() -> io::Result<(NamedTempFile, String)> {
    let db_file = NamedTempFile::new()?;
    let db_path = db_file.path().to_str().unwrap().to_string();
    
    // 初始化数据库表结构
    let conn = Connection::open(&db_path).unwrap();
    let create_table_sql = r#"
    CREATE TABLE IF NOT EXISTS suppliers (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        contact TEXT,                       -- 联系人
        wechat TEXT,                        -- 微信
        phone TEXT,                         -- 电话
        quantity INTEGER,                   -- 数量
        location TEXT,                      -- 地点
        price REAL,                         -- 价格
        bandwidth_price REAL,               -- 带宽价格
        storage_price REAL,                 -- 存储价格
        min_contract_period TEXT,           -- 最短合同期
        breach_penalties TEXT,              -- 违约金
        payment_terms TEXT,                 -- 付款方式
        server_name TEXT,                   -- 服务器名称
        server_config TEXT,                 -- 服务器配置
        rental_model TEXT,                  -- 租赁模式
        networking_category TEXT            -- 网络类型
    );
    "#;
    conn.execute_batch(create_table_sql).unwrap();
    
    Ok((db_file, db_path))
}

#[test]
fn test_help_command() {
    let mut cmd = Command::cargo_bin(&get_bin_name()).unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("供应商信息管理命令行工具"))
        .stdout(predicate::str::contains("USAGE"))
        .stdout(predicate::str::contains("COMMANDS"));
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
    let (_db_file, db_path) = create_test_db().unwrap();
    
    // 添加供应商
    let mut cmd = Command::cargo_bin(&get_bin_name()).unwrap();
    cmd.env("DB_FILE", &db_path)
        .arg("add")
        .arg("--contact").arg("测试供应商")
        .arg("--wechat").arg("test-wxid")
        .arg("--phone").arg("13800138000")
        .arg("--quantity").arg("100")
        .arg("--location").arg("测试城市")
        .arg("--price").arg("888.88")
        .arg("--bandwidth-price").arg("50.5")
        .arg("--storage-price").arg("20.2")
        .arg("--min-contract-period").arg("3个月")
        .arg("--breach-penalties").arg("违约金1000元")
        .arg("--payment-terms").arg("预付款")
        .arg("--server-name").arg("测试服务器")
        .arg("--server-config").arg("16核32G")
        .arg("--rental-model").arg("按月付费")
        .arg("--networking-category").arg("专线")
        .assert()
        .success();
    
    // 查询所有供应商
    let mut cmd = Command::cargo_bin(&get_bin_name()).unwrap();
    cmd.env("DB_FILE", &db_path)
        .arg("query")
        .assert()
        .success()
        .stdout(predicate::str::contains("测试供应商"))
        .stdout(predicate::str::contains("test-wxid"))
        .stdout(predicate::str::contains("13800138000"))
        .stdout(predicate::str::contains("100"))
        .stdout(predicate::str::contains("888.88"))
        .stdout(predicate::str::contains("测试城市"));
    
    // 使用过滤条件查询
    let mut cmd = Command::cargo_bin(&get_bin_name()).unwrap();
    cmd.env("DB_FILE", &db_path)
        .arg("query")
        .arg("--location").arg("测试城市")
        .assert()
        .success()
        .stdout(predicate::str::contains("测试供应商"));
    
    // 使用过滤条件查询 - 不存在的记录
    let mut cmd = Command::cargo_bin(&get_bin_name()).unwrap();
    cmd.env("DB_FILE", &db_path)
        .arg("query")
        .arg("--location").arg("不存在的城市")
        .assert()
        .success()
        .stdout(predicate::str::contains("没有找到符合条件的供应商"));
    
    // 使用JSON格式输出
    let mut cmd = Command::cargo_bin(&get_bin_name()).unwrap();
    cmd.env("DB_FILE", &db_path)
        .arg("query")
        .arg("--json")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"contact\":"))
        .stdout(predicate::str::contains("\"测试供应商\""))
        .stdout(predicate::str::contains("\"quantity\":"))
        .stdout(predicate::str::contains("100"));
    
    // 导出CSV
    let csv_file = NamedTempFile::new().unwrap();
    let csv_path = csv_file.path().to_str().unwrap();
    
    let mut cmd = Command::cargo_bin(&get_bin_name()).unwrap();
    cmd.env("DB_FILE", &db_path)
        .arg("query")
        .arg("--export-csv").arg(csv_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("已导出"));
    
    // 验证CSV文件内容
    let content = std::fs::read_to_string(csv_path).unwrap();
    assert!(content.contains("测试供应商"));
    assert!(content.contains("test-wxid"));
    assert!(content.contains("13800138000"));
    assert!(content.contains("100"));
}

#[test]
fn test_json_add_workflow() {
    // 创建测试数据库
    let (_db_file, db_path) = create_test_db().unwrap();
    
    // 准备JSON数据
    let json_data = r#"{
        "contact": "JSON供应商",
        "wechat": "json-wxid",
        "phone": "13900139000",
        "quantity": 200,
        "location": "JSON城市",
        "price": 999.99,
        "bandwidth_price": 60.6,
        "storage_price": 30.3,
        "min_contract_period": "6个月",
        "breach_penalties": "违约金2000元",
        "payment_terms": "月付",
        "server_name": "JSON服务器",
        "server_config": "32核64G",
        "rental_model": "包月",
        "networking_category": "BGP"
    }"#;
    
    // 使用JSON添加供应商
    let mut cmd = Command::cargo_bin(&get_bin_name()).unwrap();
    cmd.env("DB_FILE", &db_path)
        .arg("add")
        .arg("--json").arg(json_data)
        .assert()
        .success();
    
    // 查询验证
    let mut cmd = Command::cargo_bin(&get_bin_name()).unwrap();
    cmd.env("DB_FILE", &db_path)
        .arg("query")
        .arg("--contact").arg("JSON供应商")
        .assert()
        .success()
        .stdout(predicate::str::contains("JSON供应商"))
        .stdout(predicate::str::contains("json-wxid"))
        .stdout(predicate::str::contains("13900139000"))
        .stdout(predicate::str::contains("200"))
        .stdout(predicate::str::contains("JSON城市"))
        .stdout(predicate::str::contains("999.99"));
}

#[test]
fn test_multiple_suppliers() {
    // 创建测试数据库
    let (_db_file, db_path) = create_test_db().unwrap();
    
    // 添加多个供应商
    for i in 1..=5 {
        let contact = format!("供应商{}", i);
        let wechat = format!("wxid{}", i);
        let phone = format!("1380013800{}", i);
        let location = if i % 2 == 0 { "北京" } else { "上海" };
        let price = 800.0 + (i as f64 * 100.0);
        
        let mut cmd = Command::cargo_bin(&get_bin_name()).unwrap();
        cmd.env("DB_FILE", &db_path)
            .arg("add")
            .arg("--contact").arg(&contact)
            .arg("--wechat").arg(&wechat)
            .arg("--phone").arg(&phone)
            .arg("--quantity").arg(&i.to_string())
            .arg("--location").arg(location)
            .arg("--price").arg(&price.to_string())
            .assert()
            .success();
    }
    
    // 查询所有供应商
    let mut cmd = Command::cargo_bin(&get_bin_name()).unwrap();
    cmd.env("DB_FILE", &db_path)
        .arg("query")
        .assert()
        .success()
        .stdout(predicate::str::contains("供应商1"))
        .stdout(predicate::str::contains("供应商2"))
        .stdout(predicate::str::contains("供应商3"))
        .stdout(predicate::str::contains("供应商4"))
        .stdout(predicate::str::contains("供应商5"));
    
    // 按位置筛选
    let mut cmd = Command::cargo_bin(&get_bin_name()).unwrap();
    cmd.env("DB_FILE", &db_path)
        .arg("query")
        .arg("--location").arg("北京")
        .assert()
        .success()
        .stdout(predicate::str::contains("供应商2"))
        .stdout(predicate::str::contains("供应商4"))
        .stdout(predicate::str::contains("供应商1").not())
        .stdout(predicate::str::contains("供应商3").not())
        .stdout(predicate::str::contains("供应商5").not());
    
    // 按价格筛选
    let mut cmd = Command::cargo_bin(&get_bin_name()).unwrap();
    cmd.env("DB_FILE", &db_path)
        .arg("query")
        .arg("--price").arg("1100.0")
        .assert()
        .success()
        .stdout(predicate::str::contains("供应商3"))
        .stdout(predicate::str::contains("1100"))
        .stdout(predicate::str::contains("供应商1").not())
        .stdout(predicate::str::contains("供应商2").not())
        .stdout(predicate::str::contains("供应商4").not())
        .stdout(predicate::str::contains("供应商5").not());
}

#[test]
fn test_error_handling() {
    // 测试不存在的命令
    let mut cmd = Command::cargo_bin(&get_bin_name()).unwrap();
    cmd.arg("not-exist-command")
        .assert()
        .failure();
    
    // 测试添加供应商时缺少必要参数
    let (_db_file, db_path) = create_test_db().unwrap();
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