# ExtendedSearch

<div align="center">
  <img src="assets/icons/main_icon.png" alt="ExtendedSearch Logo" width="128">
  
  **一个扩展的文件搜索工具**
  
  [![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org)
  [![Slint](https://img.shields.io/badge/slint-1.12.0-blue.svg)](https://slint.dev)
  [![License](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)
  [![Platform](https://img.shields.io/badge/platform-Windows-lightgrey.svg)](https://github.com/Decmofofs/ExtendedSearch)
</div>

## 说明
本项目当前为北京大学2024-2025秋季学期"Rust程序设计"课程的学生结课项目，小组成员为[LSZ2005](https://github.com/LSZ2005)， [Decmofofs](https://github.com/Decmofofs). 当前功能较为简陋，以后或许会再行完善。

此README文档基于 `Claude sonnet 4`生成。

## ✨ 特性

### 🔍 高级搜索功能
- **多目录搜索**: 同时在多个目录中搜索文件
- **正则表达式支持**: 支持文件名和完整路径的正则表达式匹配
- **文件类型过滤**: 根据文件属性进行精确筛选
- **大小范围过滤**: 按文件大小范围搜索
- **日期范围过滤**: 支持多种日期筛选模式（天、周、月、年、特定日期范围）
- **隐藏文件搜索**: 可选择是否搜索隐藏文件和文件夹
- **只读文件过滤**: 可选择是否包含只读文件

### 📊 文件管理功能
- **批量选择**: 支持单选、多选和全选文件
- **批量复制**: 将文件批量复制到指定文件夹
- **文件删除**: 安全删除选中的文件
- **文件映射**: 将搜索结果映射到指定目录结构
- **去重功能**: 基于文件哈希值自动去除重复文件


### 💾 数据管理
- **结果导出**: 将搜索结果导出为 JSON 格式
- **结果导入**: 从 JSON 文件导入之前的搜索结果



## 🚀 快速开始

### 系统要求

- **操作系统**: Windows 10+
- **Rust**: 1.70 或更高版本

### 安装方法

#### 从源码构建

1. **克隆仓库**
   ```bash
   git clone https://github.com/Decmofofs/ExtendedSearch.git
   cd ExtendedSearch
   ```

2. **安装 Rust**（如果尚未安装）
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source ~/.cargo/env
   ```

3. **构建项目**
   ```bash
   cargo build --release
   ```

4. **运行应用**
   ```bash
   cargo run --release
   ```

#### 预编译二进制文件

前往 [Releases](https://github.com/Decmofofs/ExtendedSearch/releases) 页面下载适合您操作系统的预编译版本。

## 📖 使用指南

### 基本搜索

1. **添加搜索目录**
   - 点击左侧边栏的"添加目录"按钮
   - 选择要搜索的文件夹
   - 勾选目录以包含在搜索中

2. **配置过滤器**
   - 点击"过滤设置"选项卡
   - 设置文件大小范围、日期范围等条件
   - 可选择是否计算文件哈希值

3. **开始搜索**
   - 切换到"搜索"选项卡
   - 点击"开始搜索"按钮
   - 查看搜索结果列表

### 高级功能

#### 正则表达式搜索
```regex
# 搜索所有 .txt 文件
.*\.txt$

# 搜索以 "log" 开头的文件
^log.*

# 搜索包含日期格式的文件名
\d{4}-\d{2}-\d{2}
```

#### 文件管理操作
- **选择文件**: 点击复选框选择单个或多个文件
- **全选/取消全选**: 使用顶部的全选按钮
- **删除文件**: 选择文件后点击"删除选中文件"
- **文件映射**: 指定源文件夹后，点击"映射"将文件复制到新位置

#### 数据导入导出
- **导出**: 点击"导出结果"将当前搜索结果保存为 JSON
- **导入**: 点击"导入结果"加载之前保存的搜索结果

## 🏗️ 项目结构

```
ExtendedSearch/
├── src/
│   ├── main.rs           # 应用程序入口点
│   ├── ui_handler.rs     # UI 事件处理和回调
│   ├── filter.rs         # 搜索过滤器逻辑
│   ├── helper.rs         # 搜索和文件操作工具
│   └── search_file/      # 文件搜索模块
│       ├── mod.rs        # 主搜索功能
│       ├── build_tree.rs # 目录树构建
│       └── settings.rs   # 搜索设置
├── ui/
│   ├── app-window.slint  # 主窗口布局
│   └── components/       # UI 组件
│       ├── content-area.slint
│       ├── custom-button.slint
│       ├── filter-settings-optimized.slint
│       ├── scope-selection.slint
│       ├── search-page.slint
│       ├── search-result-view.slint
│       └── sidebar.slint
├── assets/
│   ├── fonts/           # 字体文件
│   └── icons/           # 应用图标
├── Cargo.toml          # 项目依赖配置
└── README.md           # 项目说明文档
```

## 🛠️ 技术栈

- **核心语言**: [Rust](https://www.rust-lang.org/) - 系统级编程语言，保证性能和安全性
- **UI 框架**: [Slint](https://slint.dev/) - 现代化的 Rust GUI 工具包


## 🤝 贡献指南

欢迎贡献代码！请遵循以下步骤：

1. **Fork 此仓库**
2. **创建功能分支** (`git checkout -b feature/AmazingFeature`)
3. **提交更改** (`git commit -m 'Add some AmazingFeature'`)
4. **推送到分支** (`git push origin feature/AmazingFeature`)
5. **开启 Pull Request**

### 开发环境设置

```bash
# 克隆仓库
git clone https://github.com/Decmofofs/ExtendedSearch.git
cd ExtendedSearch

# 安装开发依赖
cargo install cargo-watch

# 开发模式运行（自动重载）
cargo watch -x "run"

# 代码格式化
cargo fmt

# 代码检查
cargo clippy
```

## 📝 更新日志

### v0.1.0 (2025-06-23)
- 🎉 初始版本发布
- ✅ 基本文件搜索功能
- ✅ 高级过滤器支持
- ✅ 文件管理操作
- ✅ 数据导入导出
- ✅ 现代化用户界面

## 📄 许可证

本项目基于 [MIT License](LICENSE) 许可证开源。

## 🙏 致谢

- [Slint UI](https://slint.dev/) - 提供优秀的 Rust GUI 框架
- [Rust 社区](https://www.rust-lang.org/community) - 提供丰富的生态系统
- 所有贡献者和用户的支持

## 📞 联系方式

- **问题反馈**: [GitHub Issues](https://github.com/Decmofofs/ExtendedSearch/issues)
- **功能建议**: [GitHub Discussions](https://github.com/Decmofofs/ExtendedSearch/discussions)
- **邮箱**: decmofofs@gmail.com

---

<div align="center">
  如果这个项目对您有帮助，请考虑给它一个 ⭐️！
</div>
