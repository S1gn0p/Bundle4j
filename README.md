# Bundle4j

**Bundle4j** 是一个专为 Windows 打造的高效 Java 单文件打包工具。使用 Rust 编写，提供极速的打包体验和现代化的拖拽图形界面。

只需将 Java 的 `.exe` 和 JRE 文件夹拖入窗口，即可生成一个包含完整运行时的独立 `.exe` 文件，无需用户额外安装 Java 环境。

## 教程视频(Bundle4j_Tutorial)[893.2KB]：

https://github.com/user-attachments/assets/9f78563b-b994-4a13-90e0-c68a554a0ea5



## ✨ 核心特性

- 🖱️ **极简拖拽**：同时拖入 `.exe` 和 `jre` 文件夹，一键打包。
- 📦 **自动压缩**：内置 Deflate 压缩算法，大幅减小最终生成文件的体积（例如将 375MB 压缩至 131MB）。
- 🚀 **一键自解压**：生成的单文件在运行时，会自动将 JRE 释放到临时目录并启动你的 Java 程序。

## 📥 如何使用

1. 前往本仓库的 [Releases](https://github.com/S1gn0p/Bundle4j/releases) 页面，下载最新的 `Bundle4j.exe`。
2. 双击运行 `Bundle4j.exe`。
3. 将你的 Java 程序(使用Launch4j打包的exe java程序) `.exe` 和 `jre` 文件夹拖入窗口对应位置中。
4. 点击“开始生成单文件”，同目录下会生成 `SingleApp.exe`。
5. 将这个生成好的文件发给任何人，他们无需安装 Java 即可直接双击运行！

## 🛠️ 如何从源码编译

确保你已安装了 Rust 环境，然后在终端中执行：

```bash
# 克隆仓库
git clone https://github.com/你的用户名/Bundle4j.git
cd Bundle4j

# 编译 Release 版本
cargo build --release
