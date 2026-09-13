# CFTunnel

<div align="center">

<img src="public/cloudflared.ico" width="96" height="96" alt="CFTunnel Logo" />

<h3>基于 Tauri 2.0 + Vue 3 + Rust 构建的跨平台极简 Cloudflare 隧道管理客户端</h3>

</div>

---


## ✨ 核心特性一览

- 🪟 **Windows 11 Fluent Design 美学**：无边框自定义标题栏、亚克力毛玻璃质感、深色/浅色模式平滑切换、平滑滑动指示滑块。
- 🔀 **本地 / 远程隧道智能区分**：软件自动检测每个隧道是本地（凭证在本机）还是远程（凭证在云端），并在服务端页面分组展示。
- 🎵 **Web Audio API 纯代码合成音效**：内置轻快悬浮音、清亮点击音、复合 Tab 切换音以及阶梯成功音，支持数位笔/触控板悬浮手势。
- 🌐 **国际化多语言支持**：内置 6 大语言包（简体中文、繁體中文、English、Español、Português、日本語），切换语言即时生效。
- 📦 **在线一键安装/更新 Cloudflared**：自动检测操作系统与 CPU 架构，直连官方 Release 下载配置。
- 🔌 **全协议支持**：内置 HTTP / HTTPS / TCP / SSH / RDP / SMB / UNIX / UNIX+TLS 八大协议，另有 Hello World 内置测试服务器，一个客户端全搞定。
- ☁️ **远程隧道云端配置展示**：启动远程隧道时自动拉取并展示 Cloudflare 后台的 ingress 规则。
- 💻 **Windows Terminal 风格集成控制台**：实时捕获并展示守护进程输出，支持划词复制、拖拽调高。
- 📂 **配置文件目录一键直达**：智能打开 `~/.cloudflared`，附带醒目的凭证防泄露安全告警。
- ⚡ **Rust 极致体积压缩**：开启 `opt-level = "z"`、`lto = true`、`strip = true`、`panic = "abort"`，压榨二进制体积物理极限。

---

## 🔀 本地隧道 vs 远程隧道（核心概念）

CFTunnel 将隧道区分为两种类型，软件**一打开即自动识别**，无需手动配置：

| 对比维度 | 🟢 本地隧道 | 🟣 远程隧道 |
| :--- | :--- | :--- |
| **创建方式** | 在软件「服务端 → 本地」页点击「创建隧道」 | 在 Cloudflare Zero Trust 后台（网页）创建 |
| **凭证存放** | 本机 `~/.cloudflared/<隧道ID>.json` | 云端，以 Token（`eyJ...`）形式下发 |
| **识别依据** | 本机存在对应凭证文件 | 本机无凭证文件 |
| **启动命令** | `cloudflared tunnel --name <名称> --url <协议>://127.0.0.1:<端口>` | `cloudflared tunnel run --token <Token>` |
| **启动方式** | 软件里填名称+端口+协议，点「启动」 | 软件里粘贴 Token，点「启动远程隧道」 |
| **生命周期** | 临时，跟随软件（停止/退出即断开） | 临时，跟随软件（停止/退出即断开） |
| **域名绑定** | 软件内「DNS 路由绑定」（`route dns`） | Cloudflare 后台配置 ingress 规则（软件内只读展示） |
| **适用场景** | 快速把自己的本地服务发布到公网 | 使用 Cloudflare 后台统一管理、随时下发 Token 的隧道 |

> 💡 **一句话判断**：想临时开个隧道把自己的服务分享出去 → 用**本地隧道**；已经在 Cloudflare 后台建好了隧道、只想在软件里用 Token 跑起来 → 用**远程隧道**。

---

## 🚀 详细使用指南

### 准备工作
1. 注册并登录 [Cloudflare 账号](https://dash.cloudflare.com/)；
2. 准备一个已托管在 Cloudflare 上的域名（例如 `example.com`）。

---

### 第一步：安装与授权登录
1. 打开应用程序，切换至 **「⚙️ 配置」** 标签页；
2. 点击 **「📦 安装 cloudflared」**：程序将自动检测当前设备的系统与架构，并拉取最新官方二进制；
3. 点击 **「🔑 Cloudflared 授权登录」**：系统唤起默认浏览器，在 Cloudflare 网页上选择要授权的域名完成登录（证书保存在本地 `~/.cloudflared/cert.pem`）。

---

### 第二步：本地隧道（发布本地服务）

适用于把本地运行的服务（网站、Minecraft 游戏服、NAS 等）安全映射到公网：

1. 切换至 **「🖥️ 服务端」** 标签页，选择 **「本地隧道」** 模式；
2. 输入 **隧道名字**（纯英文字母，如 `mc`）与 **本地端口**（如 `25565`）；
3. 选择 **协议**（HTTP / HTTPS / TCP / SSH / RDP / SMB / UNIX / UNIX+TLS / Hello World），按需选择与本地服务匹配的类型；
4. 点击 **「➕ 创建隧道」**，创建成功后列表实时刷新并显示隧道 ID；
5. 选中隧道，点击 **「▶ 启动隧道」**，状态徽章变绿色「运行中」即公网隧道打通；
6. 在下方 **「DNS 路由绑定」** 输入隧道名和域名（如 `mc.example.com`），点击「绑定 DNS 路由」，即可通过该域名访问。

---

### 第三步：远程隧道（用 Token 启动云端隧道）

适用于已在 Cloudflare Zero Trust 后台创建好的隧道：

1. 切换至 **「🖥️ 服务端」** 标签页，选择 **「远程隧道」** 模式；
2. 在 **「远程隧道 Token」** 输入框粘贴 Token（可直接粘贴 `cloudflared.exe service install eyJ...` 完整命令，程序会自动提取 Token）；
3. 点击 **「▶ 启动远程隧道」**，程序以 `tunnel run --token` 临时启动（无需安装系统服务、无需管理员权限）；
4. 启动后，下方会**自动展示该隧道的云端 ingress 配置**（域名 → 服务映射）；
5. 点击 **「⏹ 停止远程隧道」** 或关闭软件，即可停止临时隧道。

---

### 第四步：客户端连接（访问远程服务）

适用于在异地设备上通过 Cloudflare 隧道直连已发布的远程服务端：

1. 切换至 **「💻 客户端」** 标签页；
2. 输入服务端绑定的 **隧道域名**（如 `mc.yourdomain.com`）与 **本地监听端口**（如 `25565`）；
3. 点击 **「🔗 连接客户端」**，连接成功后在本机访问 `127.0.0.1:25565` 即可享受内网般的直连体验。

---

### 第五步：凭证安全与配置文件目录
- 在 **「⚙️ 配置」** 页面点击 **「📂 打开本地配置文件目录」**，可快速查看 `~/.cloudflared` 目录；
- ⚠️ **安全警告**：请切勿将目录下的 `cert.pem` 证书文件与 `<tunnel_id>.json` 凭证，以及远程隧道 Token 展示或分享给任何人！

---

## 🛠️ 本地开发与源码编译

### 环境准备
- [Node.js](https://nodejs.org/) (推荐 v20 或 v22 LTS)
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
构建产物位于 `dist/` 目录下，可直接部署到任意静态服务器或 GitHub Pages，免下载即可在线体验 Windows 11 Fluent 交互全貌！

---

## 🤖 GitHub Actions 全平台自动化发布

项目内置完善的 [.github/workflows/release.yml](.github/workflows/release.yml) 矩阵构建脚本。只需推送版本标签（Tag），即可自动并行构建并发布全平台安装包至 GitHub Releases：

```bash
git tag v1.0.2
git push origin v1.0.2
```
