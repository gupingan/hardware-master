<p align="center">
  <img src="https://gitee.com/xiaogugyx/drawing-bed/raw/master/hardware-master-logo.png" alt="Hardware Master" width="66%">
</p>


<p align="center">
  基于 Rust 开发的轻量级 Windows 硬件信息检测工具
</p>
<p align="center">
    <a href="https://www.rust-lang.org"><img src="https://img.shields.io/badge/Rust-2021-orange.svg" alt="Language"></a>
    <a href="https://github.com/gupingan/hardware-master/graphs/contributors"><img src="https://img.shields.io/github/contributors/gupingan/hardware-master.svg" alt="Contributors"></a>
    <a href="https://github.com/gupingan/hardware-master/network/members"><img src="https://img.shields.io/github/forks/gupingan/hardware-master.svg?style=flat" alt="Forks"></a>
    <a href="https://github.com/gupingan/hardware-master/stargazers"><img src="https://img.shields.io/github/stars/gupingan/hardware-master.svg?style=flat" alt="Stargazers"></a>
    <a href="https://github.com/gupingan/hardware-master/issues"><img src="https://img.shields.io/github/issues/gupingan/hardware-master.svg" alt="Issues"></a>
    <a href="https://github.com/gupingan/hardware-master/blob/master/LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="MIT License"></a>
</p>


## 简介

硬大师是一款使用 Rust 编写的 Windows 硬件信息检测工具，通过 WMI 接口获取系统硬件信息，使用 egui 构建原生 GUI 界面。项目设计简洁，代码结构清晰，适合学习 Rust 系统编程和 Windows API 调用。

## 特性

- 🚀 **极速检测** - 基于 Rust 高性能引擎，秒级完成硬件扫描
- 🎯 **全面覆盖** - 支持 CPU、GPU、内存、主板、磁盘、网卡、声卡、电池、显示器等硬件检测
- 💻 **原生界面** - 使用 egui 构建，轻量流畅，无外部依赖
- 📋 **一键复制** - 支持将硬件信息一键复制到剪贴板
- 🎨 **中文支持** - 完美支持中文显示

## 快速开始

### 前置要求

- Windows 10/11
- Rust 1.70+ (2021 Edition)

### 构建

```bash
git clone https://github.com/gupingan6/hardware-master.git
cd hardware-master
cargo build --release
```

### 运行

```bash
cargo run
```

构建完成后，可执行文件位于 `target/release/hardware-master.exe`

## 技术栈

| 类别 | 技术 |
|------|------|
| **语言** | Rust 2021 Edition |
| **GUI 框架** | [egui](https://github.com/emilk/egui) |
| **硬件检测** | Windows WMI API |
| **系统信息** | [sysinfo](https://github.com/GuillaumeGomez/sysinfo) |
| **错误处理** | [thiserror](https://github.com/dtolnay/thiserror) |

## 项目结构

```
hardware-master/
├── src/
│   ├── detector/       # 硬件检测模块
│   ├── ui/            # 用户界面
│   ├── utils/         # 工具函数（WMI、注册表等）
│   ├── iddb/          # PCI/USB 设备 ID 数据库
│   └── assets/        # 资源文件
├── doc/               # 项目文档
└── Cargo.toml
```

## 许可证

本项目采用 [MIT License](LICENSE) 开源协议。

---

Made with ❤️ by [gupingan6](https://github.com/gupingan6)
