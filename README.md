# CFTunnel (Windows 11 Fluent 重制版)

<div align="center">

<img src="public/cloudflared.ico" width="96" height="96" alt="CFTunnel Logo" />

<h3>基于 Tauri 2.0 + Vue 3 + Rust 构建的跨平台极简 Cloudflare 隧道管理客户端</h3>



</div>

---

## 📖 项目起源与重制背景

本项目是早期开源的 Python 版 cloudflared_GUI 的**全新跨平台现代重制版 (Full Remake)**。

| 对比维度 | 👴 旧版 (Python 版本) | 🚀 现代重制版 (Tauri 2.0 + Rust) |
| :--- | :--- | :--- |
| **底层架构** | Python 3 + Tkinter / Qt 运行环境 | **Rust 底层核心 + Webkit 原生轻量渲染** |
| **安装包体积** | 需打包庞大的 Python 解释器（50MB+） | **极度轻量化，安装包仅约 3 ~ 5 MB** |
| **内存占用** | 运行时常驻内存 ~150MB+ | **极致低消耗，运行时仅约 25MB** |
| **UI 视觉设计** | 传统经典简陋窗口 | **Windows 11 Fluent 亚克力无边框现代美学** |
| **交互体验** | 仅支持单一语言与基础点击 | **Web Audio 纯合成音效、6国语言、动态终端、彩蛋** |
| **跨平台支持** | Windows 专属或跨平台配置繁琐 | **全面覆盖 Windows / macOS 4 大主流架构** |

---

## ✨ 核心特性一览

- 🪟 **Windows 11 Fluent Design 美学**：无边框自定义标题栏、亚克力毛玻璃质感、深色/浅色模式平滑 360° 旋转切换、平滑滑动指示滑块。
- 🎵 **Web Audio API 纯代码合成音效**：内置轻快悬浮音（`playHover`）、清亮点击音（`playClick`）、复合 Tab 切换音（`playTab`）以及阶梯四音阶成功音（`playSuccess`），支持数位笔/触控板空中悬浮手势。
- 🌐 **国际化多语言支持**：内置 6 大语言包（简体中文、繁體中文、English、Español、Português、日本語），切换语言即时生效。
- 📦 **在线一键安装/更新 Cloudflared**：前端自动精准检测操作系统（Windows / macOS）与 CPU 架构（x86_64, i686, aarch64），直连官方 Release 自动下载配置。
- 💻 **Windows Terminal 风格集成控制台**：实时捕获并展示守护进程输出、支持鼠标高亮选中划词复制、顶部把手支持上下自由拖拽调高。
- 📂 **跨平台配置文件目录一键直达**：智能动态解析并打开 `~/.cloudflared`，附带醒目的凭证防泄露安全告警条。
- ⚡ **Rust 极致体积压缩**：开启 `opt-level = "z"`、`lto = true`、`strip = true`、`panic = "abort"`，不计云端编译成本，压榨二进制体积物理极限。

---

## 🚀 详细使用指南

### 准备工作
1. 注册并登录 [Cloudflare 账号](https://dash.cloudflare.com/)；
2. 准备一个已托管在 Cloudflare 上的域名（例如 `example.com`）。

---

### 第一步：安装与授权登录
1. 打开应用程序，切换至 **「⚙️ 配置」** 标签页；
2. 点击 **「📦 安装 cloudflared」**：程序将自动检测您当前设备的系统与架构，并拉取最新官方二进制；
3. 点击 **「🔑 Cloudflared 授权登录」**：系统将唤起默认浏览器，请在打开的 Cloudflare 网页上选择您要授权的域名完成登录（授权证书将保存在本地 `~/.cloudflared/cert.pem`）。

---

### 第二步：服务端配置（创建与启动隧道）
适用于将本地运行的服务（如本地 Minecraft 游戏服、Web 网站、NAS 服务等）安全映射到公网：
1. 切换至 **「🖥️ 服务端」** 标签页；
2. 输入 **隧道名字**（纯英文字母，如 `mc`）与 **本地端口**（如 `25565`）；
3. 点击 **「➕ 创建隧道」**：创建成功后，隧道列表将实时刷新并显示新隧道的 ID；
4. 选中列表中创建的隧道，点击 **「▶ 启动隧道」**，状态徽章变为绿色「运行中」即代表公网隧道打通！

---

### 第三步：客户端连接（访问远程服务）
适用于在异地客户端设备上，通过 Cloudflare 隧道直接连接已发布的远程服务端：
1. 切换至 **「💻 客户端」** 标签页；
2. 输入由服务端绑定的 **隧道域名**（如 `mc.yourdomain.com`）与 **本地监听端口**（如 `25565`）；
3. 点击 **「🔗 连接客户端」**：连接成功后，您只需在本地应用中访问 `127.0.0.1:25565` 即可享受内网般的直连体验。

---

### 第四步：凭证安全与配置文件目录
- 在 **「⚙️ 配置」** 页面点击 **「📂 打开本地配置文件目录」**，可快速唤起系统的文件资源管理器查看 `~/.cloudflared` 目录；
- ⚠️ **安全警告**：请切勿将目录下的 `cert.pem` 证书文件与 `<tunnel_id>.json` 凭证展示或分享给任何人！

---

## 🛠️ 本地开发与源码编译

### 环境准备
- [Node.js](https://nodejs.org/) (推荐 v18 或 v20 LTS)
- [Rust](https://www.rust-lang.org/) (推荐 1.75+)
- C++ 编译环境 (Windows 上为 Visual Studio C++ Build Tools，macOS 上为 Xcode Command Line Tools)

### 1. 克隆代码并安装依赖
```bash
git clone https://github.com/bayueqi/ZQ-CFTunnel.git
cd ZQ-CFTunnel
npm install
```

### 2. 启动前端与 Tauri 本地开发
```bash
npm run tauri dev
```

### 3. 本地打包发布版本
```bash
npm run tauri build
```
构建出的安装包与便携版将输出在 `src-tauri/target/release/bundle/` 目录下。

---

## 🌐 静态网页体验版 (GitHub Pages)

本项目已实现 **Tauri Desktop 原生运行 + Web 浏览器无缝仿真** 双模式。
构建静态网页版本：
```bash
npm run build
```
构建产物直接位于 `dist/` 目录下，可直接将 `dist/` 部署到任意静态服务器或 GitHub Pages，免下载即可在线体验 Windows 11 Fluent 交互全貌！

---

## 🤖 GitHub Actions 全平台自动化发布

项目内置完善的 [.github/workflows/release.yml](.github/workflows/release.yml) 矩阵构建脚本。只需推送版本标签（Tag），即可自动并行构建并发布全平台安装包至 GitHub Releases：

```bash
git tag v1.0.0
git push origin v1.0.0
```
