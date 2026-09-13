# CFTunnel

<div align="center">

<img src="public/cloudflared.ico" width="96" height="96" alt="CFTunnel Logo" />

<h3>基于 Tauri 2.0 + Vue 3 + Rust 构建的跨平台极简 Cloudflare 隧道管理客户端</h3>

</div>

---

## 目录

- [本地隧道 vs 远程隧道](#本地隧道-vs-远程隧道)
- [支持的全部协议](#支持的全部协议)
- [前提条件](#前提条件)
- [安装与授权登录](#安装与授权登录)
- [使用本地隧道（发布本地服务）](#使用本地隧道发布本地服务)
- [使用远程隧道（用 Token 启动云端隧道）](#使用远程隧道用-token-启动云端隧道)
- [客户端连接（访问远程服务）](#客户端连接访问远程服务)
- [核心特性一览](#核心特性一览)
- [本地开发与源码编译](#本地开发与源码编译)
- [GitHub Actions 全平台自动化发布](#github-actions-全平台自动化发布)

---

## 本地隧道 vs 远程隧道

CFTunnel 里的隧道分两种，本质区别在于 **「隧道的配置写在哪、谁说了算」**：

| 对比维度 | 🟢 本地隧道 | 🟣 远程隧道 |
| :--- | :--- | :--- |
| **配置写在哪** | 本机，随用随建随改 | Cloudflare 网页后台，一次写死 |
| **灵活性** | ⬆️ 高：协议、端口随时改，改完即生效 | ⬇️ 低：协议/端口在网页端 ingress 里固定，改要回后台 |
| **创建方式** | 软件「服务端 → 本地」页点「创建隧道」 | Cloudflare Zero Trust 后台（网页）创建 |
| **凭证存放** | 本机 `~/.cloudflared/<隧道ID>.json` | 云端，以 Token（`eyJ...`）形式下发 |
| **识别依据** | 本机存在对应凭证文件 | 本机无凭证文件 |
| **启动命令** | `cloudflared tunnel --name <名称> --url <协议>://127.0.0.1:<端口>` | `cloudflared tunnel run --token <Token>` |
| **生命周期** | 临时，跟随软件（停止/退出即断开） | 临时，跟随软件（停止/退出即断开） |
| **域名绑定** | 软件内「DNS 路由绑定」（`route dns`） | Cloudflare 后台配置 ingress 规则（软件内只读展示） |
| **适用场景** | 快速把自己的本地服务发布到公网，频繁调整 | 团队统一在后台管理，向多端批量下发 Token |

> 💡 **一句话判断**：想临时开个隧道、且要频繁改协议/端口 → 用**本地隧道**；已经（或希望在）Cloudflare 后台统一写死配置、只想用 Token 跑起来 → 用**远程隧道**。

---

## 支持的全部协议

软件在「服务端」页提供协议下拉选择，对应 Cloudflare Tunnel 官方支持的 service 类型：

| 协议 | 用途 | 默认端口 | 说明 |
| :--- | :--- | :--- | :--- |
| **HTTP** | 网站 / 网页服务 | 任意（如 80） | 暴露网页，最常用 |
| **HTTPS** | 直连本地 TLS 网站服务 | 任意（如 443） | 本地已启用 HTTPS 时直连，不二次解密 |
| **TCP** | 任意 TCP 服务 / 游戏服 | 任意 | 最通用的"万能管道"，Minecraft 等游戏服用它 |
| **SSH** | 远程终端 | 22 | 本质是 TCP，端口可改 |
| **RDP** | 远程桌面 | 3389 | 本质是 TCP，端口可改 |
| **SMB** | Windows 文件共享 | 445 | 本质是 TCP，端口可改 |
| **UNIX** | Unix 套接字 | —（填套接字路径） | 仅 Linux/macOS |
| **UNIX+TLS** | Unix 套接字 + TLS | —（填套接字路径） | 仅 Linux/macOS |
| **Hello World** | 内置测试服务器 | —（无需端口） | 验证隧道本身是否打通，零配置 |

> ⚠️ **关于"默认端口"**：SSH/RDP/SMB 标注的 22/3389/445 只是**默认值**，你的服务监听在哪个端口，软件里就填哪个端口。它们本质上都是 TCP 流量，选 TCP 一样能通。
>
> ⚠️ **关于"端口是否要一致"**：服务端端口必须 = 本地服务实际监听端口；客户端（本地监听）端口可任填本机空闲端口，两端**无需一致**，隧道会自动做映射。

---

## 前提条件

使用 CFTunnel 之前，需要先满足以下三项：

1. **一个 Cloudflare 账号**：注册并登录 [dash.cloudflare.com](https://dash.cloudflare.com/)；
2. **一个托管在 Cloudflare 上的域名**：例如 `example.com`，且该域名的 DNS 由 Cloudflare 接管；
3. **cloudflared 已安装并授权登录**：见下一节。


---

## 安装与授权登录

1. 打开 CFTunnel，切换至 **「⚙️ 配置」** 标签页；
2. 点击 **「📦 安装 cloudflared」**：自动检测系统与 CPU 架构，从官方 Release 拉取最新二进制；
3. 点击 **「🔑 Cloudflared 授权登录」**：唤起默认浏览器，在 Cloudflare 网页上**选择你要授权的域名**完成登录，证书保存在本地 `~/.cloudflared/cert.pem`。

> 授权登录这一步，就是把「你的 Cloudflare 账号 + 某个域名」的操控权限下放给本机 cloudflared，之后才能在本机创建隧道、绑定该域名的 DNS 路由。

---

## 使用本地隧道（发布本地服务）

本地隧道灵活性最高：协议、端口随时在软件里改，改完即启动生效。完整流程：

1. **创建隧道**：切到「🖥️ 服务端 → 本地隧道」，输入隧道名（纯英文字母，如 `mc`），点「➕ 创建隧道」；
2. **绑定 DNS 路由**：在下方的「DNS 路由绑定」填入隧道名 + 域名（如 `mc.example.com`），点「绑定 DNS 路由」，让 `mc.example.com` 指向这条隧道；
3. **启动隧道**：选择协议（HTTP / HTTPS / TCP / SSH / RDP / SMB / UNIX / UNIX+TLS / Hello World），填好端口，点「▶ 启动隧道」，状态变绿「运行中」即打通；
4. **访问**：在浏览器或对应客户端通过 `mc.example.com` 访问你的本地服务。

> 本地隧道的 ingress 规则由软件在启动时按你填的协议/端口即时生成，所以想换端口、换协议，改一下重新启动即可，不用回网页后台。

---

## 使用远程隧道（用 Token 启动云端隧道）

远程隧道适合已经在 Cloudflare 后台写死配置、只想在本机用 Token 跑起来的场景：

1. **在 Cloudflare 后台配置**：进入 Zero Trust → Networks → Tunnels，创建隧道并**在网页里写死 ingress 规则**（域名 → 服务的映射）；Cloudflare 会生成一条安装命令，形如：
   ```
   cloudflared.exe service install eyJhIjoi...
   ```
2. **软件里粘贴 Token**：切到「🖥️ 服务端 → 远程隧道」，把上面整条命令（或其中以 `eyJ` 开头的 Token）粘贴进输入框，点「▶ 启动远程隧道」；
3. **查看云端配置**：启动后软件会自动拉取并展示该隧道在后台写死的 ingress 规则（域名 → 服务），**只读展示，不可在软件内修改**；
4. **停止**：点「⏹ 停止远程隧道」或关闭软件即断开。

> 远程隧道与本地隧道最大的区别：**协议和端口的修改必须回 Cloudflare 网页后台做**，软件只负责"拿着 Token 把隧道跑起来"这一件事。

---

## 客户端连接（访问远程服务）

在异地设备上，通过 Cloudflare 隧道直连已发布的服务端：

1. 切到 **「💻 客户端」** 标签页；
2. 输入服务端绑定的**隧道域名**（如 `mc.example.com`）与**本地监听端口**（可任填本机空闲端口，如 `25566`，无需与服务端一致）；
3. 点「🔗 连接客户端」，成功后在本机访问 `127.0.0.1:<本地监听端口>`，即可获得内网般的直连体验。

---

## 核心特性一览

- 🪟 **Windows 11 Fluent Design 美学**：无边框自定义标题栏、亚克力毛玻璃质感、深色/浅色模式平滑切换。
- 🔀 **本地 / 远程隧道智能区分**：自动检测每个隧道是本地（凭证在本机）还是远程（凭证在云端），分组展示。
- 🔌 **全协议支持**：HTTP / HTTPS / TCP / SSH / RDP / SMB / UNIX / UNIX+TLS 八大协议 + Hello World 内置测试服务器。
- ☁️ **远程隧道云端配置展示**：启动远程隧道时自动拉取并展示 Cloudflare 后台 ingress 规则。
- 🌐 **国际化多语言支持**：简体中文、繁體中文、English、Español、Português、日本語 六语言即时切换。
- 📦 **在线一键安装 / 更新 Cloudflared**：自动检测操作系统与 CPU 架构，直连官方 Release。
- 💻 **Windows Terminal 风格集成控制台**：实时捕获守护进程输出，支持划词复制、拖拽调高。
- 📂 **配置文件目录一键直达**：智能打开 `~/.cloudflared`，附带凭证防泄露安全告警。
- 🎵 **Web Audio API 纯代码合成音效**：内置轻快悬浮音、点击音、Tab 切换音与成功音。

---

## 本地开发与源码编译

### 环境准备
- [Node.js](https://nodejs.org/) (推荐 v20 或 v22 LTS)
- [Rust](https://www.rust-lang.org/) (推荐 1.75+)
- C++ 编译环境 (Windows 为 Visual Studio C++ Build Tools，macOS 为 Xcode Command Line Tools)

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

## GitHub Actions 全平台自动化发布

项目内置 [.github/workflows/release.yml](.github/workflows/release.yml) 矩阵构建脚本，推送版本标签即自动并行构建并发布全平台安装包至 GitHub Releases：

```bash
git tag v1.0.2
git push origin v1.0.2
```
