# 供应商信息管理系统 - 安装指南

本文档详细介绍了如何在不同操作系统上安装和配置供应商信息管理系统。

## 1. 系统要求

供应商信息管理系统基于Rust开发，支持以下操作系统：

- Windows 10/11
- macOS 10.15+
- Ubuntu 20.04+/Debian 11+
- CentOS/RHEL 8+

### 1.1 硬件要求

最低配置：
- CPU: 双核处理器
- 内存: 1GB RAM
- 存储: 100MB可用空间

推荐配置：
- CPU: 四核处理器或更高
- 内存: 4GB RAM或更高
- 存储: 1GB可用空间（用于存储大量数据）

### 1.2 软件依赖

- **Rust 1.63.0+**: 编译和运行程序所需的语言环境
- **SQLite 3.35.0+**: 数据存储引擎（大多数情况下已内置）
- **Git**（可选）: 用于获取源代码

## 2. 安装Rust环境

首先，需要在系统上安装Rust工具链。

### 2.1 Windows

1. 访问 [https://rustup.rs](https://rustup.rs) 并下载rustup-init.exe
2. 运行安装程序，按照提示进行安装
3. 安装完成后，打开新的命令提示符或PowerShell窗口
4. 验证安装：`rustc --version`

### 2.2 macOS/Linux

1. 打开终端
2. 运行以下命令：
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```
3. 按照提示完成安装
4. 配置当前shell：
   ```bash
   source $HOME/.cargo/env
   ```
5. 验证安装：`rustc --version`

## 3. 获取源代码

### 3.1 使用Git克隆

```bash
git clone https://github.com/yourusername/wei-assistant-gpu.git
cd wei-assistant-gpu
```

### 3.2 下载源码包

如果不使用Git，可以下载源码包：

1. 访问 https://github.com/yourusername/wei-assistant-gpu/archive/refs/heads/main.zip
2. 下载并解压缩文件
3. 打开终端或命令提示符，切换到解压目录：
   ```bash
   cd path/to/wei-assistant-gpu
   ```

## 4. 编译安装

### 4.1 开发环境构建

```bash
# 开发环境构建（更快，但二进制文件较大）
cargo build
```

生成的二进制文件位于 `target/debug/wei-assistant-gpu` (Linux/macOS) 或 `target\debug\wei-assistant-gpu.exe` (Windows)。

### 4.2 生产环境构建

```bash
# 生产环境优化构建（编译较慢，但生成更小更快的二进制文件）
cargo build --release
```

生成的二进制文件位于 `target/release/wei-assistant-gpu` (Linux/macOS) 或 `target\release\wei-assistant-gpu.exe` (Windows)。

### 4.3 直接运行

也可以使用Cargo直接运行程序：

```bash
cargo run -- --help
```

## 5. 全局安装

要在系统范围内安装程序，可以：

### 5.1 使用Cargo安装

```bash
cargo install --path .
```

这将编译并安装二进制文件到Cargo的bin目录（通常是`~/.cargo/bin/`），确保该目录在系统PATH中。

### 5.2 手动安装

也可以手动复制已编译的二进制文件到系统PATH目录：

#### Windows

```powershell
# 创建目录（如果不存在）
mkdir -p $env:USERPROFILE\bin

# 复制二进制文件
copy target\release\wei-assistant-gpu.exe $env:USERPROFILE\bin\

# 添加到PATH（如果尚未添加）
# 打开"系统属性" -> "环境变量"，将%USERPROFILE%\bin添加到用户PATH变量
```

#### Linux/macOS

```bash
# 复制到/usr/local/bin（可能需要sudo权限）
sudo cp target/release/wei-assistant-gpu /usr/local/bin/

# 或者复制到用户bin目录
mkdir -p $HOME/bin
cp target/release/wei-assistant-gpu $HOME/bin/

# 确保目录在PATH中（如果不在，添加到~/.bashrc或~/.zshrc）
echo 'export PATH="$HOME/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc
```

## 6. 配置

### 6.1 数据库位置

默认情况下，数据库文件（`wei-assistant.db`）将在程序运行的当前目录中创建。

要指定自定义位置，可以设置环境变量：

#### Windows

```powershell
$env:DB_FILE="C:\path\to\data\suppliers.db"
wei-assistant-gpu query
```

#### Linux/macOS

```bash
DB_FILE="/path/to/data/suppliers.db" wei-assistant-gpu query
```

### 6.2 创建系统服务（可选）

对于需要持续运行的服务器环境，可以创建系统服务：

#### Linux (systemd)

创建服务文件 `/etc/systemd/system/wei-assistant-gpu.service`：

```ini
[Unit]
Description=Supplier Information Management System
After=network.target

[Service]
Type=simple
User=username
WorkingDirectory=/path/to/data
ExecStart=/usr/local/bin/wei-assistant-gpu server
Environment="DB_FILE=/path/to/data/wei-assistant.db"
Restart=on-failure

[Install]
WantedBy=multi-user.target
```

启用并启动服务：

```bash
sudo systemctl enable wei-assistant-gpu.service
sudo systemctl start wei-assistant-gpu.service
```

## 7. 验证安装

执行以下命令验证安装是否成功：

```bash
wei-assistant-gpu --help
```

应显示程序的帮助信息。

## 8. 故障排除

### 8.1 常见问题

**问题**: 找不到命令 `wei-assistant-gpu`

**解决方案**: 确保安装目录在系统PATH中，或者使用完整路径运行程序：
```bash
/path/to/wei-assistant-gpu --help
```

**问题**: 无法创建数据库文件

**解决方案**: 检查当前目录的写入权限，或使用`DB_FILE`环境变量指定有权限的位置。

**问题**: SQLite相关错误

**解决方案**: 确保系统安装了SQLite（大多数系统已预装）。如果问题仍存在，尝试重新编译程序。

### 8.2 获取更多帮助

如果遇到其他问题，可以：

1. 查看项目README.md文件获取更多信息
2. 提交GitHub Issues报告问题
3. 联系技术支持团队

---

恭喜！您已成功安装供应商信息管理系统。接下来可以参考[用户指南](user_guide.md)开始使用该系统。 