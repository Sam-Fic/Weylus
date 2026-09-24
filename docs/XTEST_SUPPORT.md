# XTest Keyboard Input Support

Weylus现在支持使用X11 XTest扩展作为键盘输入方法。这对于Xorg环境特别有用。

## 功能特性

- **自动fallback**：当uinput不可用时，自动使用XTest作为备选
- **完整的键盘支持**：支持所有标准键、功能键、修饰键和小键盘
- **修饰键处理**：正确处理Shift、Ctrl、Alt和Meta(Super)键
- **与X11无缝集成**：直接与X服务器通信，无需额外的权限设置

## 工作原理

### 输入设备选择顺序 (Linux):

1. **uinput_support = true**:
   - 首先尝试创建uinput设备
   - 如果失败，自动fallback到XTest
   - 如果XTest也失败，报告错误

2. **uinput_support = false**:
   - 首先尝试创建XTest设备
   - 如果失败，fallback到AutoPilot(有限支持)

## 优势

### XTest相比uinput的优势:
- 不需要访问`/dev/uinput`（不需要root权限或特殊组权限）
- 直接与X服务器通信，对X11应用更友好
- 可以绕过某些应用的输入过滤
- 在容器或受限环境中更容易使用

### XTest相比AutoPilot的优势:
- 支持更多键码（140+ vs 40+）
- 更好的修饰键处理
- 专为X11设计，更可靠

## 系统要求

- X11显示服务器（不支持Wayland）
- XTEST扩展（大多数X11安装默认包含）
- 设置了DISPLAY环境变量

## 使用方法

XTest支持是自动启用的，无需额外配置。只需：

1. 在Linux上运行Weylus
2. 如果uinput不可用或未启用，XTest会自动作为备选

## 调试

检查XTest是否可用：
```bash
xdpyinfo | grep XTEST
```

应该看到类似输出：
```
XTEST
```

## 当前限制

- 仅实现键盘输入（鼠标和触摸仍使用uinput）
- 仅支持X11（不支持Wayland）
- 需要活动的X显示

## 技术细节

实现位于:
- `src/input/xtest_device.rs` - XTest设备实现
- `src/input/x11_keys.rs` - X11 KeySym定义
- `src/websocket.rs:229-286` - 设备选择逻辑
