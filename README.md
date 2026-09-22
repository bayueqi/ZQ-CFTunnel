# CFTunnel

Cloudflare Tunnel 的图形客户端。Tauri 2 + Rust 做外壳，Vue 3 做界面，把 `cloudflared` 建隧道、绑域名、发证书那一串命令收进一个窗口 —— 本机服务发布到公网不用敲命令，也不用写 `config.yml`。

网页演示：<http://cftunnel.520jacky.dpdns.org/>（跑的是同一套前端，数据是编的，随便点）

## 下载

在 [Releases](https://github.com/bayueqi/ZQ-CFTunnel/releases) 里挑自己系统的包：

- Windows：安装版 / 便携版，x64 与 ia32
- macOS：DMG，Intel 与 Apple 芯片各一份
- Linux：AppImage / deb / rpm，x64 与 arm64

安装包里不带 cloudflared —— 它比应用本身大得多，更新也频繁。所以**第一次打开先到「配置」页点一次「安装 cloudflared」**，软件会把官方二进制下到应用目录，之后所有功能都靠它。

## 上手三步

1. **配置 → 安装 cloudflared**：必须做，只做一次。
2. **配置 → Cloudflared 授权登录**：只有固定隧道需要。浏览器里选中要用的域名，证书写进 `%USERPROFILE%\.cloudflared\cert.pem`。
3. **服务端 → 临时隧道 或 固定隧道 → 创建**：填本机端口，固定隧道再填个域名；建好后点行尾 ▶ 启动。

侧边栏就三页：服务端、客户端、配置。底部是控制台，实时打印 cloudflared 输出和软件自己的操作日志，留最近 500 条，能划词复制，也能拖着调高。

## 临时隧道

不用登录、不用域名，启动后给你一个 `https://xxx.trycloudflare.com`，发给谁谁能访问。代价是地址每次启动都变、进程一停就作废。用来临时分享、验证连通性正合适。

## 固定隧道

要用自己的固定域名就走这条。配置存在 Cloudflare 云端（就是隧道详情页里那份 ingress 路由），本机不生成配置文件，改完边缘即时下发，不用重启隧道。填了域名的话，保存时会顺手把 DNS 记录指过来 —— 只补不删，不会动别处在用的 CNAME。

创建和修改是同一个弹窗，里面三块：

**已发布应用程序路由**（一行一条规则：协议 + 本地端口 + 域名）

- 域名可以留空，留空的那行不会被写进云端。Cloudflare 只允许一条不带域名的规则，而且必须在最后，那条由兜底行占着。
- 末行是兜底行，永远在最后、永远写进云端，默认 `http_status:404`（匹配不上的请求回 404）；把它换成某个端口，匹配不上的请求就全转过去。
- 协议选 UNIX 时端口那格变成套接字路径，选 Hello World 时整格隐藏。
- 反解不出协议和端口的存量规则会落到「其他（原始值）」，原样保留，不会被改坏。
- 端口不在 1–65535、域名格式不对、同一个域名出现在两条规则里，保存时标红；域名留空的行不参与校验。

**主机名路由 / CIDR 路由**：给 WARP 私有网络访问用的，一般留空。它俩和 ingress 是三份彼此独立的数据，某一块读失败只在那一块提示，不影响另外两块。

想改域名或解绑，去下方的 **DNS 路由绑定** 面板：按隧道分组列出账号下所有已绑域名，点域名即复制，旁边是改名和解绑。解绑会删掉 DNS 记录，并把云端 ingress 里指向它的那一行一起去掉。改域名、解绑、删隧道时，该域名下的密码锁也会顺手清掉，云端不留没人管的 Access 应用。

## 密码锁

给域名上锁之后，不带凭据的连接一律被 Cloudflare 拒掉（403）—— 浏览器裸开、扫描器、客户端直连都进不来。

锁挂在**域名**上，不是隧道上：换端口、换协议都不用换锁；一条隧道绑了几个域名就各锁各的，密码互不相同。上锁时软件会给你一对「访问账号 / 访问密码」，发给要连的人；换密码会让旧密码立刻作废、正在连的会被断开；解锁就恢复公开。

上锁走的是 Cloudflare Access 的管理接口，需要一把有写权限的 API 令牌。在 Cloudflare 仪表板 → 我的个人资料 → API 令牌 → 创建自定义令牌，权限加两行：

- 帐户 · Access: Apps and Policies · 编辑
- 帐户 · Access: Service Tokens · 编辑

「帐户资源」选上你的帐户，建好后把那串令牌粘进「配置」页的 **Access Token（密码锁凭证）**。授权登录那张证书能管隧道和域名路由，但对 Access 只有只读权限 —— 所以上锁这一步省不掉。

## 客户端

在别的机器上把远程服务映射到本机：**客户端 → 添加**，填隧道域名、本机监听端口（不必和服务端一致），目标上了锁再把访问账号 / 访问密码填上。启动后在本机访问 `127.0.0.1:<监听端口>`，跟直连一样。可以同时开多条，互不影响，行内按钮分别是启动 / 停止、编辑（运行中置灰）、删除（二次确认）。

不想用软件的话，命令行等价写法：

```bash
cloudflared access tcp --hostname mc.example.com --url tcp://127.0.0.1:25566 \
  --service-token-id <访问账号> --service-token-secret <访问密码>
```

## 协议

HTTP 给网站，HTTPS 给本地已经套了 TLS 的服务，TCP 是万能管道（游戏服、SSH、RDP、SMB、数据库都选它），UNIX / UNIX+TLS 给本机套接字（填路径，仅 Linux / macOS），Hello World 是 cloudflared 自带的测试服务器，零配置就能验证隧道通不通。

游戏服、数据库这些别选 HTTP —— 选错了一连就断，日志里是 `websocket: bad handshake`。

## 文件在哪

软件目录（`CFTunnel.exe` 旁边）：`cloudflared.exe` 是「安装 cloudflared」下下来的；`data\webview\` 存界面数据 —— 主题、语言、隧道列表、密码锁记录、控制台高度。

授权凭证不在软件目录，交给 cloudflared 自己的默认位置 `%USERPROFILE%\.cloudflared\`：`cert.pem` 是授权登录证书，`<隧道ID>.json` 是隧道密钥。

- 备份就这两处都拷：软件目录下的 `data\`，以及 `%USERPROFILE%\.cloudflared\`。
- 删掉 `data\webview\` 等于恢复出厂设置，下次启动回到默认主题和空列表，登录状态不受影响。
- 凭证放在用户目录，覆盖安装和卸载都动不到；而且和命令行共用同一份 —— 软件不改子进程的用户目录，终端里跑 `cloudflared` 用的是同一个 `~/.cloudflared`。
- 启动隧道用的 Token 由后端现取现用，不落盘。

## 开发

需要 Node.js 20 / 22、Rust 1.77+，以及 C++ 编译环境（Windows 装 Visual Studio C++ Build Tools，macOS 装 Xcode Command Line Tools）。

```bash
git clone https://github.com/bayueqi/ZQ-CFTunnel.git
cd ZQ-CFTunnel
npm install

npm run dev           # 只跑前端，浏览器里就是演示模式，调样式最快
npm run tauri dev     # 桌面端开发，带真实后端
npm run tauri build   # 本地打包，产物在 src-tauri/target/release/bundle/
```

界面主体都在 `src/App.vue`（模板、逻辑、样式一个文件），文案在 `src/locales/zh_CN.json`，Rust 侧全部逻辑在 `src-tauri/src/lib.rs`。不在 Tauri 里跑的时候，`src/utils/tauriBridge.ts` 会接管所有 IPC 调用、返回内存里的演示数据，界面上每个交互都能点通 —— 网页演示就是这一套。

## 发布

推 `v*` 标签触发 `release.yml`，六个目标矩阵构建完直接发 Release；推 `main` 只跑 `deploy-pages.yml` 更新网页演示。**只推 main 不会发 Release。**

```bash
git tag v1.0.1
git push origin v1.0.1
```

打标签前把版本号统一到同一个：`package.json`、`package-lock.json`、`src-tauri/Cargo.toml`、`src-tauri/Cargo.lock`、`src-tauri/tauri.conf.json`。CI 构建前会清掉一切预置的 `binaries`，保证安装包里没有 cloudflared；`scripts/prepare-sidecars.mjs` 是给需要离线打包的场景留的。
