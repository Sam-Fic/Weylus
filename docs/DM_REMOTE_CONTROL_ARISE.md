# Weylus 远程控制 LightDM 登录界面说明

本文档针对当前这台机器已经验证过的环境：

- 显卡 DRM 驱动：`arise`
- 显示管理器：`lightdm`
- 图形服务器：`Xorg :0`
- 桌面环境：UKUI

目标是通过 Weylus 远程查看和控制 `dm` 登录界面，也就是用户登录前的图形界面。

## 结论

在这台机器上：

- `KMS` 路径不可用，不要使用 `--kms-support`
- `LightDM + Xorg :0` 路径可以尝试
- `VAAPI` 已编进二进制，但 `arise_drv_video.so` 初始化失败，当前实际仍会回退到 `libx264`

因此，当前推荐方案是：

- 使用 `Xorg :0`
- 使用 `LightDM` 的 `XAUTHORITY`
- 使用 `sudo` 启动 Weylus
- 不开启 `--kms-support`
- 不依赖 `--try-vaapi`

## 已确认的系统状态

当前机器上已经确认：

- `lightdm.service` 处于运行状态
- X server 进程是：

```text
/usr/lib/xorg/Xorg -core :0 -seat seat0 -auth /var/run/lightdm/root/:0 -nolisten tcp vt7 -novtswitch
```

这说明：

- 登录界面运行在 `Xorg :0`
- 对应认证文件是 [`/var/run/lightdm/root/:0`](/var/run/lightdm/root/:0)

这也是 Weylus 连接登录界面时必须使用的认证文件。

## 为什么不能用 KMS

在这台 `arise` 机器上，Weylus 自己的 KMS 实验后端和原版 `kmsvnc` 都已经验证过：

- `GET_FB2` / `drmModeGetFB2` 失败，返回 `Invalid argument`
- 即使退回 `GET_FB + dumb-map` 或 `PRIME mmap`，读出的 framebuffer 内容仍然是纯白帧

因此，继续使用：

```bash
--kms-support
```

在当前环境下没有意义。

## 推荐启动方式

### 1. 普通调试版

```bash
cd /home/fit/icepie/Weylus
sudo DISPLAY=:0 XAUTHORITY=/var/run/lightdm/root/:0 \
  ./target/debug/weylus --no-gui --web-port 1701
```

### 2. Release 版

```bash
cd /home/fit/icepie/Weylus
sudo DISPLAY=:0 XAUTHORITY=/var/run/lightdm/root/:0 \
  ./target/release/weylus --no-gui --web-port 1701
```

### 3. 已编入 VAAPI 的调试版

如果当前二进制是通过下面方式编出来的：

```bash
cargo build --features vaapi
```

那么启动时还必须带上编译产物里的 `libva` 路径，否则会因为系统 `libva` 过旧出现：

```text
undefined symbol: vaSyncBuffer
```

启动命令如下：

```bash
cd /home/fit/icepie/Weylus
sudo DISPLAY=:0 \
  XAUTHORITY=/var/run/lightdm/root/:0 \
  LD_LIBRARY_PATH="$PWD/deps/dist_linux_vaapi/lib" \
  ./target/debug/weylus --no-gui --web-port 1701 --try-vaapi
```

注意：

- 当前 `arise` 的 VAAPI 驱动初始化失败
- 即使加了 `--try-vaapi`，实际也会自动回退到 `libx264`

所以当前推荐仍然是不加 `--try-vaapi`

## 浏览器访问

服务启动后，在远端浏览器访问：

```text
http://<机器IP>:1701/
```

例如：

```text
http://192.168.2.75:1701/
```

## 浏览器侧建议

为了减少登录界面阶段的问题，建议：

- 先不要勾选 `Enable uinput`
- 优先使用鼠标键盘基本控制
- 如果页面里有 capturable 列表，优先选 `Desktop`

原因：

- `/dev/uinput` 当前不可访问时，Weylus 会自动回退到 `XTest`
- 登录界面阶段不需要压感笔或多点触控
- 先验证能否看到并控制登录界面，再考虑更复杂输入

## 常见问题

### 1. 页面能打开，但没有画面

先确认 Weylus 是通过 `sudo` 启动，并且带了：

```bash
DISPLAY=:0
XAUTHORITY=/var/run/lightdm/root/:0
```

如果没有这两个环境变量，Weylus 可能连到了错误的 X 会话，或者根本没有通过认证。

### 2. 页面能打开，但没有 capturable 列表

通常表示：

- `XAUTHORITY` 不对
- 启动用户没有权限访问 LightDM 的 X 会话

优先使用：

```bash
sudo DISPLAY=:0 XAUTHORITY=/var/run/lightdm/root/:0 ...
```

### 3. 日志里出现 `/dev/uinput` 打不开

这是当前机器上的已知现象，不影响最基本的远程鼠标键盘控制。

Weylus 会尝试：

- 先创建 `uinput`
- 失败后回退到 `XTest`

如果日志里看到类似：

```text
Successfully created XTest device as fallback
```

说明输入回退已经生效。

### 4. 日志里仍然出现 `libx264`

这是当前预期行为。

因为在这台机器上：

```text
libva: Trying to open /usr/lib/dri/arise_drv_video.so
libva: va_openDriver() returns -1
Failed to initialise VAAPI connection
```

所以当前会自动回退到软件编码。

### 5. 不要再加 `--kms-support`

这台机器上 KMS 路径已经验证失败：

- Weylus KMS 实验后端失败
- 原版 `kmsvnc` 也失败

登录界面远程控制应当使用：

- `Xorg :0`
- `LightDM XAUTHORITY`

而不是 `KMS`

## 推荐的最小可用命令

建议先从这一条开始：

```bash
cd /home/fit/icepie/Weylus
sudo DISPLAY=:0 XAUTHORITY=/var/run/lightdm/root/:0 \
  ./target/debug/weylus --no-gui --web-port 1701
```

只有当它已经能稳定出画面时，再考虑是否切到：

- `target/release/weylus`
- `--try-vaapi`

## 后续优化方向

如果登录界面阶段画面不稳定，优先排查的不是 KMS，而是 X11 路径：

- 当前 X11 捕获存在 `XShm` 相关 `BadAccess` / `BadShmSeg` 风险
- 更稳妥的方向是给 X11 采集补 `XGetImage` 回退

这个优化和 `dm` 远程控制目标是直接相关的，比继续折腾 KMS 更有价值。
