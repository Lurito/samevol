# Same Volume Checker

[<img alt="github" src="https://img.shields.io/badge/github-Lurito/samevol-ee80a9?style=for-the-badge&labelColor=555555&logo=github" height="20">](https://github.com/Lurito/samevol)
[<img alt="crates.io" src="https://img.shields.io/crates/v/samevol.svg?style=for-the-badge&color=f09d13&logo=rust" height="20">](https://crates.io/crates/samevol)
[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-samevol-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs" height="20">](https://docs.rs/samevol)
[<img alt="docs.rs" src="https://img.shields.io/crates/l/samevol/0.1.1?color=d22128&style=for-the-badge&logo=apache" height="20">](./LICENSE)

> 使用中文的开发者可以跳转至下方的[中文文档](#同一存储卷检查器)。

A lightweight Windows utility for determining if two paths reside on the same storage volume.

## Features

- 🚀 Fast volume device path comparison using Windows API
- 📌 Automatically handles path normalization and mount point resolution
- 🔄 Built-in volume mapping cache with manual refresh
- 🛡️ Safe error handling for invalid paths
- 💽 Supports physical drives and VHD(X) mounts

## Installation

Add to your `Cargo.toml`:
```powershell
cargo add samevol
```

## Usage

Basic example:
```rust
use samevol::is_same_vol;

fn main() {
    let path1 = r"C:\Windows\System32";
    let path2 = r"D:\Data\test.txt";
    
    println!("Same volume? {}", is_same_vol(path1, path2)); // false
}
```

Resolves the device path of volume for a given path:
```rust
use samevol::resolve_device_path;

fn main() {
    let path = r"C:\Windows\System32\drivers\etc\hosts";
    let device_path = resolve_device_path(path).expect("Failed to resolve volume");
    println!("Device path: {}", device_path);
}
```

Force refresh volume mappings:
```rust
use samevol::reinitialize_volume_map;

fn main() {
    // After system storage configuration changes
    let count = reinitialize_volume_map().expect("Failed to refresh mappings");
    println!("Reloaded {} volume mappings", count);
}
```

## Contributing

Issues and PRs are welcome!

> Note: Due to the limitations of the maintainer's English proficiency, the existing code comments are primarily in Chinese, the maintainer's language.

## Copyright and License

Apache 2.0 © 2025 爱佐 (Ayrzo)

This repository is licensed under the Apache 2.0 License. See the [LICENSE](./LICENSE) file for details.

AI usage notice: Portions of the codebase, comments, and documentation were written with assistance from [DeepSeek AI](https://chat.deepseek.com/).

---

# 同一存储卷检查器

[<img alt="github" src="https://img.shields.io/badge/github-Lurito/samevol-ee80a9?style=for-the-badge&labelColor=555555&logo=github" height="20">](https://github.com/Lurito/samevol)
[<img alt="crates.io" src="https://img.shields.io/crates/v/samevol.svg?style=for-the-badge&color=f09d13&logo=rust" height="20">](https://crates.io/crates/samevol)
[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-samevol-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs" height="20">](https://docs.rs/samevol)
[<img alt="docs.rs" src="https://img.shields.io/crates/l/samevol/0.1.1?color=d22128&style=for-the-badge&logo=apache" height="20">](./LICENSE)

轻量级 Windows 工具库，用于检测两个路径是否位于同一存储卷。

## 功能特性

- 🚀 基于 Windows API 的快速卷设备路径比较
- 📌 自动处理路径规范化和挂载点解析
- 🔄 内置卷映射缓存支持手动刷新
- 🛡️ 安全的无效路径错误处理
- 💽 支持物理驱动器和 VHD(X) 虚拟硬盘

## 安装

添加到 `Cargo.toml`:
```powershell
cargo add samevol
```

## 使用示例

基础用法:
```rust
use samevol::is_same_vol;

fn main() {
    let path1 = r"C:\Windows\System32";
    let path2 = r"D:\Data\test.txt";
    
    println!("是否同一卷? {}", is_same_vol(path1, path2)); // false
}
```

解析某路径对应卷的设备路径:
```rust
use samevol::resolve_device_path;

fn main() {
    let path = r"C:\Windows\System32\drivers\etc\hosts";
    let device_path = resolve_device_path(path).expect("Failed to resolve volume");
    println!("设备路径: {}", device_path);
}
```

强制刷新卷映射:
```rust
use samevol::reinitialize_volume_map;

fn main() {
    // 系统存储配置变更后调用
    let count = reinitialize_volume_map().expect("映射刷新失败");
    println!("已重新加载 {} 个卷映射", count);
}
```

## 贡献指南

欢迎提交 issue 和 PR！

## 著作权和开源协议

Apache 2.0 © 2025 爱佐

本存储库根据 Apache 2.0 许可证授权。有关详细信息，请参阅 [LICENSE](./LICENSE) 文件。

人工智能使用说明：部分代码、注释和文档是在 [深度求索 DeepSeek AI](https://chat.deepseek.com/) 的辅助下编写的。
