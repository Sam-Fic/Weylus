# Weylus Docker编译指南

解决GLIBC版本兼容性问题，使用Docker编译出能在更多系统上运行的二进制文件。

## 快速开始

### 最简单的方式

```bash
./docker-build.sh
```

编译完成后，二进制文件在 `output/weylus`

## 推荐配置

### 方案1：最大兼容性 (Ubuntu 18.04 - GLIBC 2.27)

适用于大多数Linux发行版（2018年后）

```bash
./docker-build-advanced.sh --ubuntu18
```

**支持的系统**：
- Ubuntu 18.04+ / Debian 10+
- CentOS 7+ / RHEL 8+
- Fedora 28+ / openSUSE Leap 15.1+

### 方案2：平衡兼容性 (Ubuntu 20.04 - GLIBC 2.31) [默认]

适用于较新系统，构建速度快

```bash
./docker-build.sh
# 或
./docker-build-advanced.sh
```

**支持的系统**：
- Ubuntu 20.04+ / Debian 11+
- Fedora 32+ / openSUSE Leap 15.3+

### 方案3：最新特性 (Ubuntu 22.04 - GLIBC 2.35)

适用于最新系统，支持最新库特性

```bash
./docker-build-advanced.sh --ubuntu22
```

**支持的系统**：
- Ubuntu 22.04+ / Debian 12+
- Fedora 36+

## 完整选项

```bash
./docker-build-advanced.sh [选项]

选项:
  --ubuntu18      使用Ubuntu 18.04 (最大兼容性)
  --ubuntu22      使用Ubuntu 22.04 (最新特性)
  --multistage    使用多阶段构建 (更好的缓存)
  --no-cache      不使用Docker缓存 (强制重建)
  -o DIR          指定输出目录 (默认: output)
  -h, --help      显示帮助
```

## 验证编译结果

```bash
# 查看文件信息
file output/weylus

# 检查GLIBC依赖版本
ldd output/weylus | grep GLIBC

# 测试运行
./output/weylus --help
```

## 网络问题解决

### 国内用户加速

编辑 `Dockerfile.build` 或 `Dockerfile.multistage`，在 `RUN apt-get update` 前添加：

```dockerfile
# 使用清华源
RUN sed -i 's|archive.ubuntu.com|mirrors.tuna.tsinghua.edu.cn|g' /etc/apt/sources.list
```

配置Rust国内镜像，在 `RUN curl` 安装Rust后添加：

```dockerfile
RUN mkdir -p /root/.cargo && \
    echo '[source.crates-io]' > /root/.cargo/config.toml && \
    echo 'replace-with = "ustc"' >> /root/.cargo/config.toml && \
    echo '[source.ustc]' >> /root/.cargo/config.toml && \
    echo 'registry = "sparse+https://mirrors.ustc.edu.cn/crates.io-index/"' >> /root/.cargo/config.toml
```

### 使用代理

```bash
# 构建时设置代理
docker build -f Dockerfile.build \
  --build-arg http_proxy=http://proxy:port \
  --build-arg https_proxy=http://proxy:port \
  -t weylus-builder .
```

## 性能优化

### 使用缓存加速重复构建

```bash
# 创建cargo缓存卷
docker volume create weylus-cargo-cache

# 修改docker run命令
docker run --rm \
  -v "$(pwd)/output:/output" \
  -v weylus-cargo-cache:/root/.cargo \
  weylus-builder
```

### 使用多阶段构建

多阶段构建可以更好地利用Docker层缓存：

```bash
./docker-build-advanced.sh --multistage
```

## 常见问题

### Q: 编译失败，提示无法下载依赖

**A**: 网络问题，尝试：
1. 使用国内镜像源（见上文）
2. 设置代理
3. 使用 `--no-cache` 强制重建

### Q: 编译的二进制在我的系统上仍然无法运行

**A**:
1. 检查你的系统GLIBC版本：`ldd --version`
2. 使用更老的基础镜像：`./docker-build-advanced.sh --ubuntu18`
3. 检查其他依赖：`ldd output/weylus`

### Q: 编译很慢

**A**:
1. 使用多阶段构建：`--multistage`
2. 创建缓存卷（见性能优化）
3. 后续编译会快很多（利用缓存）

### Q: 想在其他架构编译（如ARM64）

**A**: 使用Docker buildx：

```bash
# 一次性设置
docker buildx create --use

# 为ARM64编译
docker buildx build \
  --platform linux/arm64 \
  -f Dockerfile.build \
  -t weylus-builder-arm64 \
  --load .

# 运行编译
docker run --rm -v "$(pwd)/output:/output" weylus-builder-arm64
```

## 目录结构

```
Weylus/
├── Dockerfile.build           # 简单构建配置
├── Dockerfile.multistage      # 多阶段构建配置
├── docker-build.sh            # 简单构建脚本
├── docker-build-advanced.sh   # 高级构建脚本
├── docs/
│   ├── DOCKER_BUILD.md        # 详细Docker文档
│   └── DOCKER_QUICK_START.md  # 本文件
└── output/                    # 编译输出目录
    └── weylus                 # 编译好的二进制
```

## 系统兼容性对照表

| 基础镜像 | GLIBC | 支持的发行版（最低版本） |
|---------|-------|------------------------|
| Ubuntu 18.04 | 2.27 | Ubuntu 18.04, Debian 10, CentOS 7 |
| Ubuntu 20.04 | 2.31 | Ubuntu 20.04, Debian 11, Fedora 32 |
| Ubuntu 22.04 | 2.35 | Ubuntu 22.04, Debian 12, Fedora 36 |

## 下一步

编译完成后：

1. **运行测试**
```bash
./output/weylus --help
```

2. **启动服务**
```bash
DISPLAY=:0 ./output/weylus
```

3. **连接设备**
打开浏览器访问 `http://your-ip:1701`

4. **测试XTest功能**
- 在平板上连接
- 测试键盘输入
- 查看日志确认使用了XTest

## 更多信息

- XTest功能说明: `docs/XTEST_SUPPORT.md`
- 完整Docker文档: `docs/DOCKER_BUILD.md`
- 测试报告: `docs/XTEST_TEST_REPORT.md`

---

**推荐配置**: 使用 `./docker-build-advanced.sh --ubuntu18` 获得最大兼容性
