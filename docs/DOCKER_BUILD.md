# Docker构建指南

## 为什么使用Docker构建

使用Docker构建可以：
- ✅ 控制GLIBC版本，提高兼容性
- ✅ 在较老系统上运行（Ubuntu 20.04+, Debian 11+）
- ✅ 避免污染主机环境
- ✅ 保证构建环境一致性

## 快速开始

### 方法1：使用构建脚本（推荐）

```bash
# 确保Docker已安装并运行
./docker-build.sh
```

编译完成后，二进制文件在 `output/weylus`

### 方法2：手动构建

```bash
# 1. 构建镜像
docker build -f Dockerfile.build -t weylus-builder .

# 2. 创建输出目录
mkdir -p output

# 3. 运行编译
docker run --rm -v "$(pwd)/output:/output" weylus-builder

# 4. 运行程序
./output/weylus
```

## 兼容性级别

### 当前配置：Ubuntu 20.04 (GLIBC 2.31)

**适用系统**：
- ✅ Ubuntu 20.04+
- ✅ Debian 11+
- ✅ Fedora 32+
- ✅ openSUSE Leap 15.3+
- ✅ RHEL/CentOS 8+

### 更高兼容性：Ubuntu 18.04 (GLIBC 2.27)

如果需要在更老的系统上运行，修改 `Dockerfile.build` 第一行：

```dockerfile
FROM ubuntu:18.04
```

**适用系统**：
- ✅ Ubuntu 18.04+
- ✅ Debian 10+
- ✅ Fedora 28+
- ✅ CentOS 7+ (需要EPEL)

### 最高兼容性：Alpine Linux (musl libc)

使用musl libc代替glibc，获得最大兼容性（但编译更复杂）。

## 检查依赖版本

编译后检查GLIBC依赖：

```bash
# 查看所需GLIBC版本
ldd output/weylus | grep GLIBC

# 示例输出 (Ubuntu 20.04编译):
# libc.so.6 => /lib/x86_64-linux-gnu/libc.so.6 (0x00007f...)
#   GLIBC_2.31
#   GLIBC_2.29
#   GLIBC_2.2.5
```

## 常见问题

### Q1: 编译很慢怎么办？

使用缓存加速：

```bash
# 创建专用volume缓存cargo
docker volume create weylus-cargo-cache

# 修改docker run命令添加缓存
docker run --rm \
  -v "$(pwd)/output:/output" \
  -v weylus-cargo-cache:/root/.cargo \
  weylus-builder
```

### Q2: 网络问题无法下载依赖

设置代理：

```bash
# 构建时传递代理
docker build -f Dockerfile.build \
  --build-arg http_proxy=http://your-proxy:port \
  --build-arg https_proxy=http://your-proxy:port \
  -t weylus-builder .
```

### Q3: 想使用国内镜像源

在 `Dockerfile.build` 的 `RUN apt-get update` 前添加：

```dockerfile
# 使用清华源
RUN sed -i 's|archive.ubuntu.com|mirrors.tuna.tsinghua.edu.cn|g' /etc/apt/sources.list

# 使用阿里云源
RUN sed -i 's|archive.ubuntu.com|mirrors.aliyun.com|g' /etc/apt/sources.list
```

对于Rust crates，在Dockerfile中添加：

```dockerfile
RUN mkdir -p /root/.cargo && \
    echo '[source.crates-io]' > /root/.cargo/config.toml && \
    echo 'replace-with = "ustc"' >> /root/.cargo/config.toml && \
    echo '[source.ustc]' >> /root/.cargo/config.toml && \
    echo 'registry = "sparse+https://mirrors.ustc.edu.cn/crates.io-index/"' >> /root/.cargo/config.toml
```

### Q4: 需要静态链接的二进制

修改Dockerfile，在编译命令前添加：

```dockerfile
ENV RUSTFLAGS="-C target-feature=+crt-static"
RUN cargo build --release --target x86_64-unknown-linux-musl
```

## 交叉编译

### 为ARM64编译（树莓派等）

```dockerfile
FROM --platform=linux/arm64 ubuntu:20.04
# ... 其余配置相同
```

然后：

```bash
docker buildx build --platform linux/arm64 -f Dockerfile.build -t weylus-builder-arm64 .
```

## 验证编译结果

```bash
# 检查文件类型
file output/weylus

# 检查动态链接库
ldd output/weylus

# 检查XTest支持
output/weylus --help  # 应正常显示帮助

# 在X11环境中测试
DISPLAY=:0 output/weylus
```

## 清理

```bash
# 删除Docker镜像
docker rmi weylus-builder

# 删除输出文件
rm -rf output/

# 删除缓存volume
docker volume rm weylus-cargo-cache
```

## 生产部署

编译后的二进制文件可以：

1. **直接分发**：将 `output/weylus` 复制到目标系统
2. **打包deb**：使用 `cargo deb` 创建Debian包
3. **创建AppImage**：使用linuxdeploy创建通用包
4. **创建运行时镜像**：创建一个小型镜像只包含运行时依赖

### 示例：创建运行时Docker镜像

```dockerfile
# Dockerfile.runtime
FROM ubuntu:20.04

RUN apt-get update && apt-get install -y \
    libx11-6 \
    libxtst6 \
    libglib2.0-0 \
    libgstreamer1.0-0 \
    libdbus-1-3 \
    && rm -rf /var/lib/apt/lists/*

COPY output/weylus /usr/local/bin/weylus

EXPOSE 1701

CMD ["weylus"]
```

构建运行时镜像：

```bash
docker build -f Dockerfile.runtime -t weylus:latest .
```

运行：

```bash
docker run -d \
  --name weylus \
  -p 1701:1701 \
  -e DISPLAY=$DISPLAY \
  -v /tmp/.X11-unix:/tmp/.X11-unix \
  weylus:latest
```

---

**提示**：首次构建可能需要10-30分钟（取决于网络速度），后续增量构建会快很多。
