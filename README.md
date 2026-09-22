# CFTunnel

Cloudflare Tunnel 的图形客户端，Tauri 2 + Rust 外壳，Vue 3 界面。把本机服务发布到公网不用敲命令。

网页演示：<http://cftunnel.520jacky.dpdns.org/>（假数据，随便点）

## 下载

在 [Releases](https://github.com/bayueqi/ZQ-CFTunnel/releases) 挑自己系统的包：Windows 有安装版和便携版，macOS 有 DMG（Intel 与 Apple 芯片各一份），Linux 有 AppImage / deb / rpm（x64 与 arm64）。

安装包里不带 cloudflared，所以第一次打开先到「配置」页点一次「安装 cloudflared」。

## 用之前

- **临时隧道**：装上 cloudflared 就能用，不用登录。
- **固定隧道**：要用自己的域名，先去「配置 → Cloudflared 授权登录」，在浏览器里选中要授权的域名。

侧边栏就三页：服务端（子项「临时隧道」「固定隧道」）、客户端、配置。底部是控制台，打印日志，能划词复制。

## 建一条隧道

**临时隧道**：「服务端 → 临时隧道 → 创建」，填个本机端口，几秒后给你一个 `https://xxx.trycloudflare.com`，发给谁谁能访问。地址每次启动都会变，程序关了就没。

**固定隧道**：「服务端 → 固定隧道 → 创建」，填隧道名（纯英文字母，建好不能再改）、本机端口、域名，建好后点行尾 ▶ 启动。

创建和修改是同一个弹窗，一行一条路由：选协议、填端口、填域名。最后一行留作兜底，没匹配上的请求走它，默认回 404；想让所有请求都转过去，就把它的协议和端口填上。弹窗里另外两块是给 WARP 私有网络用的，一般不用动。

域名要改名或者解绑，去固定隧道下方的「DNS 路由绑定」，按隧道列出已绑域名，点域名即复制。

## 密码锁

给域名上锁之后，不带凭据的连接一律被 Cloudflare 拒掉（403），浏览器裸开、扫描器、客户端直连都进不来。锁挂在域名上，一条隧道绑了几个域名就各锁各的。

上锁会给你一对「访问账号 / 访问密码」，发给要连的人；换密码会让旧密码立刻作废；解锁就恢复公开。

上锁需要一把有写权限的 Cloudflare API 令牌：仪表板 → 我的个人资料 → API 令牌 → 创建自定义令牌，权限加两行 —— 帐户 · Access: Apps and Policies · 编辑，帐户 · Access: Service Tokens · 编辑。建好粘到「配置」页的 Access Token（密码锁凭证）里。

## 客户端

在别的机器上连回来：「客户端 → 添加」，填隧道域名、本机监听端口（不必和服务端一样），目标上了锁再把访问账号 / 访问密码填上。启动后在这台机器访问 `127.0.0.1:<端口>` 就行，可以同时开多条。

不想用软件的话，命令行等价写法：

```bash
cloudflared access tcp --hostname mc.example.com --url tcp://127.0.0.1:25566 \
  --service-token-id <访问账号> --service-token-secret <访问密码>
```

## 协议

HTTP 给网站，HTTPS 给本地已经套了 TLS 的服务，TCP 给游戏服 / SSH / 数据库这类，UNIX 和 UNIX + TLS 给本机套接字（仅 Linux / macOS），Hello World 是 cloudflared 自带的测试服务器。

游戏服和数据库别选 HTTP，选错了一连就断，日志里会写 `websocket: bad handshake`。

## 文件在哪

软件目录（`CFTunnel.exe` 旁边）里有 `cloudflared.exe`，以及 `data\webview\` —— 界面数据都在这里，包括主题、语言、隧道列表、密码锁记录。

凭证在 `%USERPROFILE%\.cloudflared\`：`cert.pem` 是授权登录证书，`<隧道ID>.json` 是隧道密钥。要备份就这两个地方都拷；删掉 `data\webview\` 等于恢复出厂设置。

## 开发

需要 Node.js 20 / 22、Rust 1.77+、C++ 编译环境（Windows 用 VS C++ Build Tools，macOS 用 Xcode Command Line Tools）。

```bash
npm install
npm run dev           # 只跑前端，浏览器里就是演示模式
npm run tauri dev     # 桌面端开发
npm run tauri build   # 本地打包，产物在 src-tauri/target/release/bundle/
```

界面都在 `src/App.vue`，文案在 `src/locales/zh_CN.json`，Rust 侧逻辑在 `src-tauri/src/lib.rs`。

## 发布

推 `v*` 标签触发 `release.yml`，六个目标构建完直接发 Release；推 `main` 只更新网页演示。

```bash
git tag v1.0.1
git push origin v1.0.1
```
