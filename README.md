# CFTunnel

<div align="center">
<img src="public/cloudflared.ico" width="96" height="96" alt="CFTunnel Logo" />
<h3>基于 Tauri 2 + Vue 3 + Rust 的 Cloudflare Tunnel 图形客户端</h3>
<p>把 <code>cloudflared</code> 的命令行操作收进一个窗口。</p>
</div>

<p align="center">
  <a href="#下载">下载</a> ·
  <a href="#快速开始">快速开始</a> ·
  <a href="#具体配置">具体配置</a> ·
  <a href="#协议">协议</a> ·
  <a href="#开发">开发</a> 
</p>

---

## 下载

从 [Releases](https://github.com/bayueqi/ZQ-CFTunnel/releases) 选择对应平台的安装包：

| 系统 | 安装包 |
| --- | --- |
| Windows | 安装版、便携版 |
| macOS | DMG（Intel 芯片 / Apple 芯片各一份） |
| Linux | AppImage、deb、rpm（x64 / arm64） |

安装包内不含 cloudflared，首次启动后需在「⚙️ 配置」页点击「📦 安装 cloudflared」。

## 快速开始

| 步骤 | 位置 | 做什么 |
| :--- | :--- | :--- |
| 1. 装 cloudflared | **⚙️ 配置** → 「📦 安装 cloudflared」 | 自动识别系统与 CPU 架构，从官方 Release 下载 |
| 2. 授权登录 | **⚙️ 配置** → 「🔑 Cloudflared 授权登录」 | 浏览器里**选中要授权的域名**，证书写入 `%USERPROFILE%\.cloudflared\cert.pem` |
| 3. 建隧道 | **🖥️ 服务端** → 固定 / 临时隧道 → 「＋ 创建」 | 选协议、填本机端口；固定隧道再填域名 |
| 4. 启动 | 列表行尾 **▶** | 临时隧道给你一个 `*.trycloudflare.com`，固定隧道用你绑定的域名 |

> 第 1、2 步只有**固定隧道**需要。临时隧道免登录、免凭证，装上 cloudflared 就能用。

前置条件：一个 Cloudflare 账号；要用固定隧道，再要一个托管在 Cloudflare 上的域名。

## 具体配置

### 服务端

#### 临时隧道

临时隧道免登录、免凭证，装好 cloudflared 即可使用。

进入 **🖥️ 服务端** → 临时隧道 → 「＋ 创建」，填写协议和本机端口，数秒后列表中出现该隧道。点击行尾 **▶** 启动，控制台会输出一个 `https://xxx.trycloudflare.com` 地址，发给他人即可访问。

地址每次启动都会重新生成，程序关闭后失效。可以同时创建多条临时隧道。

#### 固定隧道

固定隧道使用自己的域名，创建前需先完成「📦 安装 cloudflared」与「🔑 Cloudflared 授权登录」。

进入 **🖥️ 服务端** → 固定隧道 → 「＋ 创建」，填写隧道名（仅限英文字母，创建后不可修改），并填写配置，创建完成后点击列表行尾 **▶** 启动。

##### DNS 路由绑定

位于固定隧道下方，按隧道列出已绑定的域名。点击域名即可复制，并可就地改名或解绑。

##### 密码锁

为域名上锁后，未携带凭据的连接会被 Cloudflare 拒绝（403），浏览器、扫描器与客户端直连均无法进入。锁绑定在域名上，同一隧道下的多个域名各自独立。

上锁时生成一对「访问账号 / 访问密码」，分发给需要连接的人。更换密码后旧密码立即失效；解锁后恢复公开访问。

上锁需要一把具备写权限的 Cloudflare API 令牌，在仪表板 → 我的个人资料 → API 令牌 → 创建自定义令牌，添加以下权限：

| 权限 | 级别 |
| --- | --- |
| 帐户 · Access: Apps and Policies | 编辑 |
| 帐户 · Access: Service Tokens | 编辑 |

创建后填入「⚙️ 配置」页的「Access Token（密码锁凭证）」。

## 协议

| 协议 | 适用服务 |
| --- | --- |
| HTTP | 网站 |
| HTTPS | 本机已启用 TLS 的服务 |
| TCP | 游戏服务、SSH、数据库等 |
| UNIX | 本机套接字（仅 Linux / macOS） |
| UNIX + TLS | 本机套接字，叠加 TLS（仅 Linux / macOS） |
| Hello World | cloudflared 自带的测试服务 |


### 客户端配置

在需要连接的机器上进入「客户端 → 添加」，填写以下字段：

| 字段 | 说明 |
| --- | --- |
| 隧道域名 | 服务端建立的隧道域名 |
| 监听端口 | 本机监听端口，无需与服务端一致 |
| 访问账号 / 访问密码 | 目标域名启用密码锁时填写 |

启动后访问 `127.0.0.1:<监听端口>` 即可，支持同时运行多条连接。

不使用客户端时，可执行等价命令：

```bash
cloudflared access tcp --hostname mc.example.com --url tcp://127.0.0.1:25566 \
  --service-token-id <访问账号> --service-token-secret <访问密码>
```

## 开发

依赖 Node.js 20 / 22、Rust 1.77+ 与 C++ 编译环境（Windows 为 VS C++ Build Tools，macOS 为 Xcode Command Line Tools）。

```bash
npm install
npm run dev           # 仅运行前端，浏览器中为演示模式
npm run tauri dev     # 桌面端开发
npm run tauri build   # 本地打包，产物位于 src-tauri/target/release/bundle/
```


