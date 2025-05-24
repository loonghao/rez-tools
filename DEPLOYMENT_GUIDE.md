# rez-tools 部署和分发指南

本指南详细说明了 rez-tools Rust 重构版本的完整部署和分发策略。

## 🎯 支持的部署场景

### 场景一：无 rez 环境的自动化安装

当用户系统中没有安装 rez 时，rez-tools 提供自动化安装：

```bash
# 检查 rez 环境
rt check-rez

# 自动安装 rez（按优先级尝试）
rt install-rez
```

**安装方法优先级：**
1. **uv + 虚拟环境**：使用 `uv` 创建隔离环境并安装 rez
2. **系统 pip**：使用系统 Python 的 pip 安装 rez
3. **Python Build Standalone**：下载独立 Python 环境并安装 rez

### 场景二：现有 rez 环境的集成

自动检测和集成现有 rez 安装：

```bash
# 自动检测 REZ_PATH 环境变量
export REZ_PATH=/path/to/rez

# 自动发现 rez 包路径
rt check-rez
```

**集成特性：**
- 自动读取 `REZ_PATH` 环境变量
- 解析 rez 配置文件获取包路径
- 与现有 rez 工作流无缝集成

### 场景三：跨平台支持

支持所有主要平台，符合 VFX Platform 标准：

| 平台 | 架构 | 状态 |
|------|------|------|
| Windows | x86_64 | ✅ 支持 |
| Linux | x86_64 | ✅ 支持 |
| macOS | x86_64 | ✅ 支持 |
| macOS | ARM64 | ✅ 支持 |

## 📦 安装方法

### 方法 1：一键安装脚本

**Unix/Linux/macOS:**
```bash
curl -fsSL https://raw.githubusercontent.com/loonghao/rez-tools/rust-rewrite/install.sh | bash
```

**Windows (PowerShell):**
```powershell
iwr -useb https://raw.githubusercontent.com/loonghao/rez-tools/rust-rewrite/install.ps1 | iex
```

### 方法 2：通过 pip 安装

```bash
pip install rez-tools
```

这将自动下载适合当前平台的预编译二进制文件。

### 方法 3：手动下载

从 [GitHub Releases](https://github.com/loonghao/rez-tools/releases) 下载对应平台的二进制文件。

### 方法 4：从源码构建

```bash
git clone https://github.com/loonghao/rez-tools.git
cd rez-tools
git checkout rust-rewrite
cargo build --release
```

## 🔧 配置管理

### 智能配置解析

rez-tools 支持多种配置格式，按优先级自动选择：

1. **Python 配置执行**（推荐）
   - 通过 Python 解释器执行配置文件
   - 完美支持 `os.path.dirname(__file__)` 等动态表达式
   - 100% 兼容现有配置

2. **TOML 配置**（现代化）
   - 简洁的 TOML 格式
   - 更好的可读性和维护性
   - 支持配置转换工具

3. **简化解析器**（降级）
   - 处理基本 Python 语法
   - 作为 Python 解释器不可用时的后备方案

### 配置文件示例

**Python 格式 (reztoolsconfig.py):**
```python
import os

tool_paths = [
    os.path.normpath(os.path.expanduser("~/packages")),
    os.path.dirname(__file__),
    "/shared/tools"
]

extension = ".rt"
```

**TOML 格式 (reztoolsconfig.toml):**
```toml
extension = ".rt"

tool_paths = [
    "~/packages",
    "/shared/tools"
]
```

### 配置转换

```bash
# 将 Python 配置转换为 TOML
rt-convert-config reztoolsconfig.py reztoolsconfig.toml
```

## 🚀 CI/CD 流水线

### GitHub Actions 工作流

项目包含完整的 CI/CD 流水线：

**持续集成 (.github/workflows/ci.yml):**
- 多平台测试（Windows、Linux、macOS）
- 代码质量检查（clippy、fmt）
- 集成测试

**发布流水线 (.github/workflows/release.yml):**
- 多平台二进制构建
- 自动创建 GitHub Release
- PyPI 包发布
- Homebrew 公式更新

### 发布流程

1. **创建标签**：
   ```bash
   git tag v0.1.0
   git push origin v0.1.0
   ```

2. **自动构建**：GitHub Actions 自动构建所有平台的二进制文件

3. **自动发布**：
   - GitHub Release 创建
   - PyPI 包上传
   - Homebrew 公式更新

## 🐍 Python 包装器

### PyPI 分发策略

参考 `uv` 的分发模式，提供 Python 包装器：

```python
from rez_tools import RezTools

# 创建实例（自动下载二进制文件）
rt = RezTools()

# 列出插件
plugins = rt.list_plugins()

# 检查 rez 环境
rez_info = rt.check_rez()

# 运行命令
result = rt.run(["maya", "--help"])
```

### 包结构

```
python/
├── pyproject.toml          # 项目配置
├── rez_tools/
│   ├── __init__.py         # 包初始化
│   ├── core.py             # 核心功能
│   ├── cli.py              # 命令行接口
│   └── py.typed            # 类型标记
└── README.md               # 包文档
```

## 📊 性能对比

| 指标 | Python 版本 | Rust 版本 | 改进 |
|------|-------------|-----------|------|
| 启动时间 | ~200ms | ~20ms | **10x 更快** |
| 内存使用 | ~50MB | ~10MB | **5x 更少** |
| 二进制大小 | Python + 依赖 | ~2MB | **显著减少** |
| 跨平台部署 | 需要 Python 运行时 | 单一可执行文件 | **零依赖** |

## 🔄 迁移策略

### 渐进式迁移

**第一阶段：并行部署**
```bash
# 安装 Rust 版本到不同路径
curl -fsSL https://raw.githubusercontent.com/loonghao/rez-tools/rust-rewrite/install.sh | bash
# 二进制安装到 ~/.local/bin/rt

# 保持现有 Python 版本
which rt-python  # 现有版本
which rt          # 新 Rust 版本
```

**第二阶段：功能验证**
```bash
# 使用相同配置测试两个版本
export REZ_TOOL_CONFIG=/path/to/reztoolsconfig.py

rt-python list    # Python 版本
rt list           # Rust 版本

# 对比输出确保一致性
```

**第三阶段：完全替换**
```bash
# 备份原版本
mv /usr/local/bin/rt /usr/local/bin/rt-python-backup

# 部署新版本
cp ~/.local/bin/rt /usr/local/bin/rt

# 可选：现代化配置
rt-convert-config reztoolsconfig.py reztoolsconfig.toml
export REZ_TOOL_CONFIG=/path/to/reztoolsconfig.toml
```

## 🛠️ 故障排除

### 常见问题

**1. 二进制文件下载失败**
```bash
# 手动下载并安装
wget https://github.com/loonghao/rez-tools/releases/latest/download/rt-linux-x86_64.tar.gz
tar -xzf rt-linux-x86_64.tar.gz
chmod +x rt
mv rt ~/.local/bin/
```

**2. Python 配置解析失败**
```bash
# 检查 Python 解释器
python --version
python3 --version

# 手动测试配置文件
python -c "exec(open('reztoolsconfig.py').read()); print(tool_paths)"
```

**3. rez 安装失败**
```bash
# 手动安装 rez
pip install rez

# 或使用 uv
uv venv rez-env
source rez-env/bin/activate  # Linux/macOS
# rez-env\Scripts\activate   # Windows
pip install rez
```

### 调试模式

```bash
# 启用详细日志
rt -v list
rt --verbose check-rez

# 检查配置解析
rt check-rez
```

## 📚 相关资源

- [rez 官方文档](https://github.com/nerdvegas/rez)
- [VFX Platform 标准](https://vfxplatform.com/)
- [Python Build Standalone](https://gregoryszorc.com/docs/python-build-standalone/main/)
- [uv 包管理器](https://github.com/astral-sh/uv)

## 🤝 贡献指南

1. Fork 项目
2. 创建功能分支
3. 提交更改
4. 创建 Pull Request

详细信息请参考 [CONTRIBUTING.md](CONTRIBUTING.md)。
