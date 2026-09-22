# CFTunnel

Cloudflare Tunnel 的图形客户端，基于 Tauri 2（Rust）与 Vue 3，用于把本机服务发布到公网，无需使用命令行。

在线演示：<http://cftunnel.520jacky.dpdns.org/>（演示数据，可自由点击）

## 下载

从 [Releases](https://github.com/bayueqi/ZQ-CFTunnel/releases) 选择对应平台的安装包：

| 系统 | 安装包 |
| --- | --- |
| Windows | 安装版、便携版 |
| macOS | DMG（Intel 芯片 / Apple 芯片各一份） |
| Linux | AppImage、deb、rpm（x64 / arm64） |

安装包内不含 cloudflared，首次启动后需在「配置」页点击「安装 cloudflared」。

## 两种隧道

| 对比项 | 临时隧道 | 固定隧道 |
| --- | --- | --- |
| 使用前提 | 安装 cloudflared | 安装 cloudflared，并完成授权登录 |
| 访问地址 | `https://xxx.trycloudflare.com`，每次启动随机生成 | 自有域名，固定不变 |
| 有效期 | 程序关闭即失效 | 长期有效 |
| 适用场景 | 临时分享 | 长期对外提供服务 |

使用固定隧道前，先进入「配置 → Cloudflared 授权登录」，在浏览器中选中要授权的域名。

## 界面

侧边栏共三页：服务端（子项「临时隧道」「固定隧道」）、客户端、配置。底部为日志控制台。

## 建一条隧道

**临时隧道**：进入「服务端 → 临时隧道 → 创建」，填写本机端口，数秒后生成访问地址。

**固定隧道**：进入「服务端 → 固定隧道 → 创建」，填写以下字段：

| 字段 | 说明 |
| --- | --- |
| 隧道名 | 仅限英文字母，创建后不可修改 |
| 本机端口 | 被发布服务所在的端口 |
| 域名 | 已授权域名下的一条记录 |

创建完成后，点击列表行尾的 ▶ 启动。

创建与修改共用同一弹窗，路由按行填写：

| 字段 | 说明 |
| --- | --- |
| 协议 | 见下方「协议」一节 |
| 端口 | 本机服务端口 |
| 域名 | 绑定的域名 |

最后一行为兜底路由，未匹配的请求由该行处理，默认为 404；需要全部转发时，填写该行的协议与端口。

## DNS 路由绑定

位于固定隧道下方，按隧道列出已绑定的域名。点击域名即可复制，并可就地改名或解绑。

## 密码锁

为域名上锁后，未携带凭据的连接会被 Cloudflare 拒绝（403），浏览器、扫描器与客户端直连均无法进入。锁绑定在域名上，同一隧道下的多个域名各自独立。

上锁时生成一对「访问账号 / 访问密码」，分发给需要连接的人。更换密码后旧密码立即失效；解锁后恢复公开访问。

上锁需要一把具备写权限的 Cloudflare API 令牌，在仪表板 → 我的个人资料 → API 令牌 → 创建自定义令牌，添加以下权限：

| 权限 | 级别 |
| --- | --- |
| 帐户 · Access: Apps and Policies | 编辑 |
| 帐户 · Access: Service Tokens | 编辑 |

创建后填入「配置」页的「Access Token（密码锁凭证）」。

## 客户端

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

## 协议

| 协议 | 适用服务 |
| --- | --- |
| HTTP | 网站 |
| HTTPS | 本机已启用 TLS 的服务 |
| TCP | 游戏服务、SSH、数据库等 |
| UNIX | 本机套接字（仅 Linux / macOS） |
| UNIX + TLS | 本机套接字，叠加 TLS（仅 Linux / macOS） |
| Hello World | cloudflared 自带的测试服务 |

游戏服务与数据库请勿选择 HTTP，否则连接会中断，日志中会提示 `websocket: bad handshake`。

## 开发

依赖 Node.js 20 / 22、Rust 1.77+ 与 C++ 编译环境（Windows 为 VS C++ Build Tools，macOS 为 Xcode Command Line Tools）。

```bash
npm install
npm run dev           # 仅运行前端，浏览器中为演示模式
npm run tauri dev     # 桌面端开发
npm run tauri build   # 本地打包，产物位于 src-tauri/target/release/bundle/
```

界面代码位于 `src/App.vue`，文案位于 `src/locales/zh_CN.json`，Rust 侧逻辑位于 `src-tauri/src/lib.rs`。

## 发布

推送 `v*` 标签触发 `release.yml`，完成六个目标的构建后自动发布 Release；推送 `main` 仅更新在线演示。

```bash
git tag v1.0.1
git push origin v1.0.1
```
