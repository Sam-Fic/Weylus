# 🚀 Docker编译快速开始

## 最简单的方法（使用代理）

```bash
./docker-build-with-proxy.sh
```

这个脚本已经配置好了你的代理 `http://192.168.2.222:12333`

## 编译完成后

```bash
# 运行Weylus
./output/weylus

# 或指定端口
./output/weylus --port 1701
```

## 修改代理地址

编辑 `docker-build-with-proxy.sh`，修改第6行：

```bash
PROXY="http://你的代理地址:端口"
```

## 其他构建选项

### 不使用代理
```bash
./docker-build.sh
```

### 最大兼容性（适合老系统）
```bash
# 方法1: 使用高级脚本
./docker-build-advanced.sh --ubuntu18

# 方法2: 手动指定
docker build -f Dockerfile.build \
  --build-arg http_proxy=http://192.168.2.222:12333 \
  --build-arg https_proxy=http://192.168.2.222:12333 \
  --build-arg BASE_IMAGE=ubuntu:18.04 \
  -t weylus-builder .

docker run --rm -v "$(pwd)/output:/output" weylus-builder
```

### 使用国内镜像加速

编辑 `Dockerfile.build`，取消第14行的注释：

```dockerfile
# 从这行
# RUN sed -i 's|archive.ubuntu.com|mirrors.tuna.tsinghua.edu.cn|g' /etc/apt/sources.list

# 改为
RUN sed -i 's|archive.ubuntu.com|mirrors.tuna.tsinghua.edu.cn|g' /etc/apt/sources.list
```

配置Rust镜像，取消第40-44行注释：

```dockerfile
RUN mkdir -p /root/.cargo && \
    echo '[source.crates-io]' > /root/.cargo/config.toml && \
    echo 'replace-with = "ustc"' >> /root/.cargo/config.toml && \
    echo '[source.ustc]' >> /root/.cargo/config.toml && \
    echo 'registry = "sparse+https://mirrors.ustc.edu.cn/crates.io-index/"' >> /root/.cargo/config.toml
```

## 检查编译结果

```bash
# 查看文件信息
file output/weylus

# 查看GLIBC依赖
ldd output/weylus | grep GLIBC

# 测试运行
./output/weylus --help
```

## 系统兼容性

### 使用 Ubuntu 20.04 编译（默认）
✅ 支持系统（GLIBC 2.31+）：
- Ubuntu 20.04+
- Debian 11+
- Fedora 32+
- CentOS 8+

### 使用 Ubuntu 18.04 编译（最大兼容性）
✅ 支持系统（GLIBC 2.27+）：
- Ubuntu 18.04+
- Debian 10+
- CentOS 7+
- Fedora 28+

## 常见问题

### Q: 提示GLIBC版本不足
**A**: 使用更老的基础镜像重新编译：
```bash
./docker-build-advanced.sh --ubuntu18
```

### Q: 编译很慢
**A**:
1. 启用国内镜像源（见上文）
2. 后续编译会利用缓存，会快很多

### Q: 网络连接失败
**A**:
1. 检查代理是否可用：`curl -x http://192.168.2.222:12333 https://www.google.com`
2. 尝试使用国内镜像源

### Q: 编译后的程序在目标系统无法运行
**A**: 检查目标系统GLIBC版本：
```bash
ldd --version
```
如果版本低于2.31，使用 `--ubuntu18` 重新编译

## 运行服务

```bash
# 在X11环境中运行
DISPLAY=:0 ./output/weylus

# 后台运行
nohup ./output/weylus > weylus.log 2>&1 &

# 指定端口
./output/weylus --port 8080
```

## XTest功能验证

编译后的Weylus已包含XTest键盘输入支持：

1. 启动Weylus
2. 在平板/手机上连接
3. 测试键盘输入
4. 查看日志确认使用了XTest：
   ```
   debug: Using XTest device for input
   ```

详细说明见 `docs/XTEST_SUPPORT.md`

---

**推荐**: 首次使用 `./docker-build-with-proxy.sh` 快速开始
