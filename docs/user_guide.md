# 供应商信息管理系统 - 用户指南

本文档提供了供应商信息管理系统的完整使用指南，帮助您快速上手和使用所有功能。

## 1. 系统概述

供应商信息管理系统是一个命令行工具，专门用于管理GPU及服务器供应商的信息。您可以记录供应商的联系方式、服务器配置、价格、位置等关键信息，并通过灵活的查询功能快速筛选符合特定条件的供应商。

## 2. 基本命令

系统支持以下基本命令：

- `add`: 添加新的供应商信息
- `query`: 查询已添加的供应商信息

### 查看帮助信息

```bash
# 查看总体帮助
wei-assistant-gpu --help

# 查看添加命令帮助
wei-assistant-gpu add --help

# 查看查询命令帮助
wei-assistant-gpu query --help
```

## 3. 添加供应商信息

系统支持两种方式添加供应商信息：参数方式和JSON方式。

### 3.1 参数方式添加

通过命令行参数逐一指定各个字段：

```bash
wei-assistant-gpu add --contact 张三 --wechat zhangsanwx --phone 13800000000 --quantity 10 --location 北京 --price 1000 --bandwidth_price 100 --storage_price 50 --min_contract_period 1年 --breach_penalties 无 --payment_terms 月付 --server_name 服务器A --server_config "8核16G" --rental_model 包年 --networking_category 专线
```

**参数说明：**

- `--contact`: 供应商联系人姓名
- `--wechat`: 微信号
- `--phone`: 电话号码
- `--quantity`: 可提供的GPU/服务器数量
- `--location`: 服务器所在地点
- `--price`: 基本价格
- `--bandwidth_price`: 带宽价格
- `--storage_price`: 存储价格
- `--min_contract_period`: 最短合同期限
- `--breach_penalties`: 违约金条款
- `--payment_terms`: 付款方式
- `--server_name`: 服务器名称
- `--server_config`: 服务器配置
- `--rental_model`: 租赁模式
- `--networking_category`: 网络类型

### 3.2 JSON方式添加

如果您有大量数据或从其他系统导出的数据，可以通过JSON格式一次性添加：

```bash
wei-assistant-gpu add --json '{"contact":"李四","wechat":"lisiwx","phone":"13900000000","quantity":5,"location":"上海","price":1200,"bandwidth_price":80,"storage_price":40,"min_contract_period":"2年","breach_penalties":"违约金2000元","payment_terms":"季付","server_name":"服务器B","server_config":"16核32G","rental_model":"包月","networking_category":"BGP"}'
```

JSON格式必须包含与参数方式相同的字段名。

## 4. 查询供应商信息

### 4.1 查询所有供应商

```bash
wei-assistant-gpu query
```

### 4.2 条件筛选查询

可以指定一个或多个条件进行筛选：

```bash
# 按位置筛选
wei-assistant-gpu query --location 北京

# 按价格筛选
wei-assistant-gpu query --price 1000

# 多条件组合筛选
wei-assistant-gpu query --location 北京 --rental_model 包年
```

系统会返回满足所有指定条件的记录。

### 4.3 输出格式选项

#### 4.3.1 表格输出（默认）

默认情况下，查询结果会以表格格式显示，便于在终端阅读。

#### 4.3.2 JSON输出

添加`--json`参数可以以JSON格式输出结果：

```bash
wei-assistant-gpu query --location 北京 --json
```

这种格式适合程序间数据交换或后续处理。

#### 4.3.3 导出为CSV

可以将查询结果导出为CSV文件：

```bash
# 导出所有供应商信息
wei-assistant-gpu query --export-csv all_suppliers.csv

# 导出筛选后的结果
wei-assistant-gpu query --location 北京 --export-csv beijing_suppliers.csv
```

## 5. 高级用法

### 5.1 环境变量配置

系统支持通过环境变量自定义某些行为：

- `DB_FILE`: 指定数据库文件路径（默认：`wei-assistant.db`）

示例：
```bash
DB_FILE=/path/to/custom.db wei-assistant-gpu query
```

### 5.2 数据管理最佳实践

1. **定期备份**: 定期复制`wei-assistant.db`文件以备份数据
2. **数据导出**: 使用`--export-csv`定期导出所有数据
3. **数据验证**: 添加新供应商后，立即查询验证信息是否正确

## 6. 错误处理

常见错误及解决方法：

1. **数据库访问错误**: 检查数据库文件权限和路径
2. **JSON格式错误**: 确保JSON字符串格式正确
3. **查询无结果**: 检查筛选条件是否过于严格
4. **参数错误**: 参考帮助信息确保参数名称正确

## 7. 常见问题解答

**Q: 如何修改已添加的供应商信息?**  
A: 当前版本不支持直接修改，您需要查询出该记录，记下ID，然后通过数据库工具直接修改。

**Q: 是否支持批量导入数据?**  
A: 您可以编写脚本循环调用`add`命令，或使用SQLite工具直接导入数据。

**Q: 如何完全重置数据库?**  
A: 删除`wei-assistant.db`文件，系统会在下次运行时自动创建新数据库。

## 8. 技术支持

如遇到问题，请通过以下渠道获取支持：

- 提交GitHub Issue
- 发送邮件至support@example.com

---

感谢您使用供应商信息管理系统！ 