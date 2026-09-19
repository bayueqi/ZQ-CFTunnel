use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Emitter, Manager, State};

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x08000000;

#[derive(Default)]
pub struct AppState {
    server_process: Arc<Mutex<HashMap<String, Child>>>,
    /// 客户端隧道支持多开：key 为 `域名|本地端口`，可同时桥接多条隧道。
    client_process: Arc<Mutex<HashMap<String, ClientTunnelProc>>>,
    remote_process: Arc<Mutex<HashMap<String, Child>>>,
    quick_process: Arc<Mutex<HashMap<String, Child>>>,
}

/// 一条正在运行的客户端隧道（`cloudflared access tcp` 桥接进程）。
struct ClientTunnelProc {
    domain: String,
    port: String,
    child: Child,
}

/// 回传前端的客户端隧道状态（不含进程句柄）。
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ClientTunnelState {
    pub key: String,
    pub domain: String,
    pub port: String,
}

/// 客户端隧道进程表 key：同一「域名 + 本地端口」视为同一条桥接。
fn client_tunnel_key(domain: &str, port: &str) -> String {
    format!("{}|{}", domain.trim(), port.trim())
}

/// 回传前端的云端托管隧道状态（key 为隧道 ID，与进程表 key 一致）。
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RemoteTunnelState {
    pub key: String,
}

/// 某条云端托管隧道拉取到的 ingress 配置，随 key 一起下发。
/// 多条隧道并行时前端据此把配置分组显示，避免糊成一片分不清是哪条隧道。
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RemoteConfigUpdate {
    pub key: String,
    pub config: String,
}

/// 云端 ingress 里的一条转发规则（只读展示用）。
///
/// `hostname` / `path` 可能同时为空：那就是 ingress 末尾的兜底规则（catch-all），
/// 前端据此显示成「(默认)」。`path` 非空表示该规则只匹配特定路径。
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TunnelIngressRule {
    pub hostname: String,
    pub path: String,
    pub service: String,
}

/// 一条隧道的云端配置快照。
///
/// `source` 是 Cloudflare 标注的配置来源：`cloudflare` = 云端托管（在面板/API 改），
/// `local` = 本地托管（cloudflared 用本机 config.yml 跑起来后上报的）。
/// `version` 每次写入都会递增，可用来判断「刚改的有没有生效」。
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TunnelConfig {
    pub source: String,
    pub version: u64,
    pub rules: Vec<TunnelIngressRule>,
}

/// 一条「主机名路由」（Zero Trust 的路由资源，独立于 ingress）。
///
/// 注意它跟上面 `TunnelIngressRule` 不是一回事：主机名路由是把某个主机名
/// （可以是 `office-1.local` 这种内网名）指到隧道，落在公共 DNS 之外，
/// 面板上是单独一页、单独一套 API。
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TunnelHostnameRoute {
    pub hostname: String,
    pub comment: String,
}

/// 一条「CIDR 路由」（= 私有网络网段经 WARP 访问）。
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TunnelCidrRoute {
    pub network: String,
    pub comment: String,
}

/// 一条隧道除 ingress 之外的其余两页路由（只读展示用）。
///
/// 两块数据来自两个不同的账号级端点，**失败互不影响**（比如 token 少了
/// `Cloudflare One Networks Read` 权限时主机名路由 403，CIDR 路由仍可能正常），
/// 所以各自带一个 `*_error`：`None` = 读取成功（列表可能为空）。
/// 整条命令不会因为其中一块失败而报错，前端也能分别显示「无」与「读取失败」。
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TunnelRouteSet {
    pub hostname_routes: Vec<TunnelHostnameRoute>,
    pub cidr_routes: Vec<TunnelCidrRoute>,
    pub hostname_error: Option<String>,
    pub cidr_error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TunnelInfo {
    pub id: String,
    pub name: String,
    pub created: String,
    pub connections: String,
    pub tunnel_type: String,
}

/// 一条指向隧道的 DNS 绑定记录（Cloudflare 区域内的 CNAME）。
///
/// `id` 是 Cloudflare 的 dns_record_id，改名与解绑都以它为操作对象，
/// 避免按域名匹配时因重名或已改动而操作到错误的记录。
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DnsBinding {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LogPayload {
    pub message: String,
    pub level: String,
    pub source: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct QuickUrlPayload {
    pub key: String,
    pub url: String,
}

// ============ 数据存放：界面数据在软件目录，cloudflared 凭证用默认目录 ============
//
// 布局（<软件目录> = 可执行文件所在目录，比如 D:\CFTunnel）：
//
//   <软件目录>\cftunnel.exe          主程序
//   <软件目录>\uninstall.exe         卸载器
//   <软件目录>\cloudflared.exe       自动下载的 cloudflared（不再跟着「启动时的工作目录」乱跑）
//   <软件目录>\data\webview\         WebView2 用户数据目录：界面设置、隧道列表、token 全在这里的 localStorage
//   %USERPROFILE%\.cloudflared\      cloudflared 凭证：cert.pem（授权登录）+ <隧道ID>.json（隧道密钥）
//
// 用可执行文件所在目录而不是「当前工作目录」，是为了无论从资源管理器、快捷方式还是
// 其它程序启动，数据都落在同一个地方。
// 而凭证特意**不**放进软件目录：安装目录会被覆盖安装 / 卸载整目录清空（2026-09-19 真实事故），
// 详见 default_cloudflared_dir 的注释。

/// 软件安装目录：可执行文件所在目录，取不到时退回当前工作目录。
fn app_dir() -> PathBuf {
    if let Ok(current_exe) = std::env::current_exe() {
        if let Some(exe_dir) = current_exe.parent() {
            return exe_dir.to_path_buf();
        }
    }
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

/// 软件数据根目录：`<软件目录>\data`。
fn app_data_dir() -> PathBuf {
    app_dir().join("data")
}

/// WebView2 用户数据目录：`<软件目录>\data\webview`。
///
/// 界面里的语言 / 主题 / 端口 / 客户端隧道列表都保存在这个目录下的
/// localStorage 里，删掉它等于恢复出厂设置。
/// （云端托管的隧道 Token 不落盘：每次启动都从 cert.pem 现取。）
fn webview_data_dir() -> PathBuf {
    app_data_dir().join("webview")
}

/// cloudflared 的默认凭证目录 `~/.cloudflared`（Windows 即 `%USERPROFILE%\.cloudflared`）。
///
/// 授权证书 `cert.pem` 与隧道密钥 `<隧道ID>.json` 一律放这里，跟 cloudflared 自己的默认行为一致。
///
/// **不再做「便携化」**：早先版本把凭证收进 `<软件目录>\data\cloudflared`，但 data 目录就在
/// 安装目录里面，用户覆盖安装 / 卸载时会被 NSIS 整目录清空，凭证跟着全丢（2026-09-19 真实事故）。
/// 现在交回默认位置 —— 重装软件不会碰用户主目录，凭证天然安全。
fn default_cloudflared_dir() -> PathBuf {
    std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .map(PathBuf::from)
        .unwrap_or_default()
        .join(".cloudflared")
}

/// 当前生效的 cert.pem：固定为 cloudflared 的默认位置 `~/.cloudflared/cert.pem`。
fn origin_cert_path() -> PathBuf {
    default_cloudflared_dir().join("cert.pem")
}

/// 递归复制目录（std 没有现成的 copy_dir）。
fn copy_dir_all(src: &Path, dst: &Path) -> std::io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let target = dst.join(entry.file_name());
        if file_type.is_dir() {
            copy_dir_all(&entry.path(), &target)?;
        } else {
            // 单个文件复制失败（被占用等）不阻断整体迁移
            let _ = fs::copy(entry.path(), &target);
        }
    }
    Ok(())
}

/// 首次启动时把旧版 WebView2 数据目录（`%LOCALAPPDATA%\<identifier>`）整体搬到软件目录，
/// 否则用户会看到隧道列表和 token 全部「消失」。
fn migrate_legacy_webview_profile() {
    let target = webview_data_dir();
    if target.exists() {
        return;
    }
    let mut candidates: Vec<PathBuf> = Vec::new();
    if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
        candidates.push(PathBuf::from(local_app_data).join("com.zhishifenzi.cloudflared-gui"));
    }
    if let Ok(roaming_app_data) = std::env::var("APPDATA") {
        // 更早期构建（可执行文件叫 cftunnel-app.exe 那版）留下的旧 profile
        candidates.push(PathBuf::from(roaming_app_data).join("cftunnel-app.exe"));
    }
    for src in candidates {
        if src.join("EBWebView").exists() && copy_dir_all(&src, &target).is_ok() {
            return;
        }
    }
}

/// 优先获取软件安装目录下的 cloudflared 可执行文件，不存在时回退到系统环境变量 PATH 中的程序
pub fn get_cloudflared_executable() -> PathBuf {
    #[cfg(target_os = "windows")]
    let exe_name = "cloudflared.exe";
    #[cfg(not(target_os = "windows"))]
    let exe_name = "cloudflared";

    // 1. 软件安装目录（下载也固定装到这里，保证「程序在哪、依赖就在哪」）
    let install_path = app_dir().join(exe_name);
    if install_path.exists() {
        return install_path;
    }

    // 2. 兼容：当前工作目录（开发时手动放的 cloudflared 仍可用）
    if let Ok(cwd) = std::env::current_dir() {
        let local_path = cwd.join(exe_name);
        if local_path.exists() {
            return local_path;
        }
    }

    // 3. 回退到系统 PATH
    PathBuf::from(exe_name)
}

fn create_base_command() -> Command {
    let program = get_cloudflared_executable();
    let mut cmd = Command::new(program);
    #[cfg(target_os = "windows")]
    cmd.creation_flags(CREATE_NO_WINDOW);

    // 凭证一律使用 cloudflared 自己的默认位置：`%USERPROFILE%\.cloudflared`，此处不做任何改写。
    // 早先版本做过「便携化」——改写 HOME/USERPROFILE，并把 TUNNEL_ORIGIN_CERT 钉到
    // `<软件目录>\data\cloudflared`。但 data 目录就在安装目录里，覆盖安装/卸载会被 NSIS
    // 整目录清空，凭证随之全丢（2026-09-19 真实事故）。回退默认位置后，重装软件不再影响凭证。

    cmd
}

/// 把 cloudflared 的 stderr 整理成一行可读文本。
///
/// cloudflared 出错时会把一整行 JSON 日志吐到 stderr，形如
/// `{"level":"error","message":"Cannot find a valid certificate for your origin at the path:\n\n D:\\...cert.pem","originCertPath":"...","time":"..."}`，
/// 后面再跟一行纯文本收尾说明；而 message 字段本身还带换行。
/// 原样塞进控制台会把一行日志撑成好几行，看着像好几条错误混在一起，
/// 所以这里取出 JSON 里的 message、把换行压平，多行再拼成一整句。
fn tidy_cloudflared_error(stderr: &str) -> String {
    let mut parts: Vec<String> = Vec::new();
    for line in stderr.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let text = serde_json::from_str::<serde_json::Value>(trimmed)
            .ok()
            .and_then(|v| {
                v.get("message")
                    .and_then(|m| m.as_str())
                    .map(str::to_string)
            })
            .unwrap_or_else(|| trimmed.to_string());
        let flat = text.split_whitespace().collect::<Vec<_>>().join(" ");
        if !flat.is_empty() {
            parts.push(flat);
        }
    }
    parts.join(" ")
}

/// 整理 cloudflared 原始输出里的**一行**，用于逐行转发到控制台。
///
/// 与 `tidy_cloudflared_error` 是同一套整理逻辑，区别在于调用场景：
/// 这里是「每读一行就转发一行」，所以
///   · 空行返回 None —— 直接丢掉，否则控制台会出现「只有标签、没有正文」的空行；
///   · JSON 行只取 `message` —— 否则同一行里会**再出现一次层级信息**
///     （`{"level":"error","message":"…"}` 送进去，左边已经有 `[ERROR]` 标签了），
///     还白搭一长串 `originCertPath` / `time` 之类的无关字段。
fn tidy_cloudflared_line(line: &str) -> Option<String> {
    let text = tidy_cloudflared_error(line);
    if text.is_empty() {
        None
    } else {
        Some(text)
    }
}

/// 判断 cloudflared 一行输出该配哪个级别标签。
///
/// 优先读 JSON 形态里的 `level` 字段：关键词猜测会把 `{"level":"info",…,"error":null}`
/// 这类**只是带了个 error 字段**的普通日志误判成错误，于是好好的 info 行被挂上 `[ERROR]`。
/// 只有非 JSON 行（cloudflared 纯文本格式形如 `2026-…Z ERR …`）才退回关键词判断。
fn cloudflared_line_level(line: &str) -> &'static str {
    if let Ok(v) = serde_json::from_str::<serde_json::Value>(line.trim()) {
        if let Some(lv) = v.get("level").and_then(|x| x.as_str()) {
            let lv = lv.to_ascii_lowercase();
            return if lv.starts_with("err") || lv == "fatal" || lv == "critical" {
                "error"
            } else if lv.starts_with("warn") {
                "warn"
            } else {
                "info"
            };
        }
    }
    if line.contains("ERR") || line.contains("error") {
        "error"
    } else if line.contains("WRN") || line.contains("warn") {
        "warn"
    } else {
        "info"
    }
}

#[tauri::command]
fn list_tunnels() -> Result<Vec<TunnelInfo>, String> {
    let mut cmd = create_base_command();
    cmd.args(["tunnel", "list", "--output", "json"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let output = cmd.output().map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            "未找到 cloudflared 可执行文件。请先在「cloudflared」Tab 点击「安装 cloudflared」或将其放置在应用根目录下。".to_string()
        } else {
            format!("执行 cloudflared 失败: {}", e)
        }
    })?;

    let stdout_str = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr_str = String::from_utf8_lossy(&output.stderr).to_string();

    // 如果 stdout 为空，可能是没有隧道或出错
    if stdout_str.trim().is_empty() {
        if !stderr_str.trim().is_empty() {
            return Err(tidy_cloudflared_error(&stderr_str));
        }
        return Ok(Vec::new());
    }

    // 解析 JSON 数组
    #[derive(Deserialize)]
    struct RawTunnel {
        id: String,
        name: String,
        created_at: String,
        connections: Vec<serde_json::Value>,
    }

    let raw_list: Vec<RawTunnel> = serde_json::from_str(&stdout_str)
        .map_err(|e| format!("解析隧道列表失败: {}", e))?;

    // 隧道密钥（<隧道ID>.json）与 cert.pem 同处：cloudflared 的默认目录 ~/.cloudflared
    let cred_dir = default_cloudflared_dir();

    let mut list = Vec::new();
    for t in raw_list {
        let connections_str = if t.connections.is_empty() {
            String::new()
        } else {
            t.connections.iter()
                .filter_map(|c| c.get("colo_name").and_then(|v| v.as_str()).map(|s| format!("1x{}", s)))
                .collect::<Vec<_>>()
                .join(", ")
        };
        let cred_path = cred_dir.join(format!("{}.json", t.id));
        let tunnel_type = if cred_path.exists() {
            "local".to_string()
        } else {
            "remote".to_string()
        };
        list.push(TunnelInfo {
            id: t.id,
            name: t.name,
            created: t.created_at,
            connections: connections_str,
            tunnel_type,
        });
    }

    Ok(list)
}

#[tauri::command]
fn create_tunnel(name: String) -> Result<String, String> {
    let trimmed = name.trim();
    if trimmed.is_empty() || !trimmed.chars().all(|c| c.is_ascii_alphabetic()) {
        return Err("隧道名不能为空且只能包含纯字母 (a-z, A-Z)".to_string());
    }

    let mut cmd = create_base_command();
    cmd.args(["tunnel", "create", trimmed])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let output = cmd.output().map_err(|e| format!("执行命令失败: {}", e))?;
    let out_str = String::from_utf8_lossy(&output.stdout).to_string();
    let err_str = String::from_utf8_lossy(&output.stderr).to_string();

    if output.status.success() {
        // 新建隧道生成的 <隧道ID>.json 密钥就落在 cert.pem 同目录（~/.cloudflared），
        // 与软件读取的位置一致，不需要额外搬运。
        Ok(if out_str.trim().is_empty() { err_str } else { out_str })
    } else {
        Err(if !err_str.trim().is_empty() {
            tidy_cloudflared_error(&err_str)
        } else {
            tidy_cloudflared_error(&out_str)
        })
    }
}

#[tauri::command]
fn delete_tunnel(name: String) -> Result<String, String> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err("隧道名不能为空".to_string());
    }

    // 删除隧道前，先清理该隧道绑定的所有域名（Cloudflare 侧的 CNAME 记录），
    // 避免留下指向已删除隧道的孤儿域名。找不到 tunnel_id（例如从未绑定过域名、
    // 或未完成授权登录没有 cert.pem）时不做清理，也不阻断删除隧道本身。
    let mut dns_cleanup_note = String::new();
    if let Some(tunnel_id) = find_tunnel_id_by_name(trimmed) {
        match get_hostnames_for_tunnel(&tunnel_id) {
            Ok(bindings) => {
                if bindings.is_empty() {
                    dns_cleanup_note = "（该隧道无绑定域名）".to_string();
                } else {
                    let mut removed = 0usize;
                    let mut failed = 0usize;
                    for b in &bindings {
                        match delete_dns_record_by_id(&b.id) {
                            Ok(_) => removed += 1,
                            Err(e) => {
                                failed += 1;
                                eprintln!("删除绑定域名 {} 失败: {}", b.name, e);
                            }
                        }
                    }
                    dns_cleanup_note = format!(
                        "；已清理绑定域名 {}/{} 条",
                        removed,
                        bindings.len()
                    );
                    if failed > 0 {
                        dns_cleanup_note.push_str(&format!("（{} 条失败）", failed));
                    }
                }
            }
            Err(e) => {
                dns_cleanup_note = format!("；域名清理跳过（{}）", e);
            }
        }
    }

    // 一律带 `--force` 强删：只要云端还有任意一个 cloudflared 副本连着这条隧道
    // （可能是别的机器、也可能是本机没被软件纳管的进程），
    // 不带 -f 的删除会被 Cloudflare 以 `code: 1022 This tunnel has active connections` 拒绝。
    // 用户点删除的意图很明确，这里不再区分「有没有服务在跑」。
    let mut cmd = create_base_command();
    cmd.args(["tunnel", "delete", "-f", trimmed])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let output = cmd.output().map_err(|e| format!("执行删除命令失败: {}", e))?;
    let out_str = String::from_utf8_lossy(&output.stdout).to_string();
    let err_str = String::from_utf8_lossy(&output.stderr).to_string();

    if output.status.success() {
        Ok(format!("隧道 {} 已成功删除{}", trimmed, dns_cleanup_note))
    } else {
        Err(if !err_str.trim().is_empty() {
            tidy_cloudflared_error(&err_str)
        } else {
            tidy_cloudflared_error(&out_str)
        })
    }
}

#[tauri::command]
fn route_dns_tunnel(name: String, hostname: String) -> Result<String, String> {
    let trimmed_name = name.trim();
    let trimmed_hostname = hostname.trim();

    if trimmed_name.is_empty() || !trimmed_name.chars().all(|c| c.is_ascii_alphabetic()) {
        return Err("隧道名不能为空且只能包含纯字母 (a-z, A-Z)".to_string());
    }
    if trimmed_hostname.is_empty() || trimmed_hostname.chars().any(|c| c.is_whitespace()) {
        return Err("域名不能为空且不能包含空格".to_string());
    }

    let mut cmd = create_base_command();
    cmd.args(["tunnel", "route", "dns", trimmed_name, trimmed_hostname])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let output = cmd.output().map_err(|e| format!("执行命令失败: {}", e))?;
    let out_str = String::from_utf8_lossy(&output.stdout).to_string();
    let err_str = String::from_utf8_lossy(&output.stderr).to_string();

    if output.status.success() {
        Ok(if out_str.trim().is_empty() { err_str } else { out_str })
    } else {
        Err(if !err_str.trim().is_empty() {
            tidy_cloudflared_error(&err_str)
        } else {
            tidy_cloudflared_error(&out_str)
        })
    }
}

/// 检查指定名称的隧道是否已存在。
///
/// 通过 `cloudflared tunnel list` 的默认文本输出解析隧道名列表，
/// 不依赖 `--output json`，以兼容所有 cloudflared 版本（旧版不支持 json 输出）。
fn tunnel_exists(name: &str) -> Result<bool, String> {
    let mut cmd = create_base_command();
    cmd.args(["tunnel", "list"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let output = cmd.output().map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            "未找到 cloudflared 可执行文件。请先在「cloudflared」Tab 点击「安装 cloudflared」或将其放置在应用根目录下。".to_string()
        } else {
            format!("执行 cloudflared 失败: {}", e)
        }
    })?;

    let stdout_str = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr_str = String::from_utf8_lossy(&output.stderr).to_string();

    // 未登录或执行出错时（此时无法确定隧道是否存在），视为失败并提示
    if !output.status.success() {
        return Err(if !stderr_str.trim().is_empty() {
            tidy_cloudflared_error(&stderr_str)
        } else {
            tidy_cloudflared_error(&stdout_str)
        });
    }

    // 逐行解析默认文本输出，隧道名位于第二列。
    // 输出形如：
    //   ID                                    NAME        CREATED              CONNECTIONS
    //   <uuid>                                <name>      <ts>                 <...>
    // 跳过表头行与空行，按空白切分取第 2 列作为隧道名。
    let target = name.trim();
    for line in stdout_str.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let cols: Vec<&str> = trimmed.split_whitespace().collect();
        if cols.len() < 2 {
            continue;
        }
        // 跳过表头（NAME 所在行）
        if cols[1].eq_ignore_ascii_case("NAME") {
            continue;
        }
        if cols[1] == target {
            return Ok(true);
        }
    }

    Ok(false)
}

/// 校验协议是否属于 cloudflared 官方支持的类型。
///
/// 支持的协议（service 前缀）：http、https、tcp、ssh、rdp、smb、unix、unix+tls，
/// 以及特殊模式 hello_world（内置测试服务器）。
fn is_supported_protocol(protocol: &str) -> bool {
    matches!(
        protocol,
        "http" | "https" | "tcp" | "ssh" | "rdp" | "smb" | "unix" | "unix+tls" | "hello_world"
    )
}

#[tauri::command]
fn start_server_tunnel(
    app: AppHandle,
    state: State<'_, AppState>,
    name: String,
    port: String,
    protocol: String,
    unix_socket: Option<String>,
) -> Result<String, String> {
    let name_trimmed = name.trim();
    let port_trimmed = port.trim();

    if name_trimmed.is_empty() || !name_trimmed.chars().all(|c| c.is_ascii_alphabetic()) {
        return Err("隧道名字必须为纯字母".to_string());
    }

    let protocol_trimmed = protocol.trim();
    if !is_supported_protocol(protocol_trimmed) {
        return Err(format!("不支持的协议类型: {}", protocol_trimmed));
    }

    // hello_world 为内置测试服务器，无需端口；其余协议都需要合法端口
    if protocol_trimmed != "hello_world"
        && (port_trimmed.is_empty() || port_trimmed.parse::<u16>().is_err())
    {
        return Err("端口号必须为 1-65535 的纯数字".to_string());
    }

    // unix / unix+tls 协议需要提供套接字路径
    let socket_trimmed = unix_socket.as_deref().map(|s| s.trim()).unwrap_or("");
    if (protocol_trimmed == "unix" || protocol_trimmed == "unix+tls")
        && socket_trimmed.is_empty()
    {
        return Err("unix / unix+tls 协议必须填写套接字路径".to_string());
    }

    // 启动前先确认隧道已存在，避免 cloudflared 旧版快捷语法自动创建隧道
    if !tunnel_exists(name_trimmed)? {
        return Err(format!(
            "隧道 [{}] 不存在，请先点击「创建隧道」创建后再启动",
            name_trimmed
        ));
    }

    let mut proc_guard = state.server_process.lock().map_err(|e| e.to_string())?;
    // 同一隧道名重复启动时，先停掉旧实例再启动新的（按隧道名去重）
    if let Some(mut old) = proc_guard.remove(name_trimmed) {
        let _ = old.kill();
    }

    let url_arg = if protocol_trimmed == "hello_world" {
        String::new()
    } else if protocol_trimmed == "unix" {
        format!("unix:{}", socket_trimmed)
    } else if protocol_trimmed == "unix+tls" {
        format!("unix+tls:{}", socket_trimmed)
    } else {
        format!("{}://127.0.0.1:{}", protocol_trimmed, port_trimmed)
    };

    let mut cmd = create_base_command();
    if protocol_trimmed == "hello_world" {
        cmd.args(["tunnel", "--name", name_trimmed, "--hello-world"]);
    } else {
        cmd.args(["tunnel", "--name", name_trimmed, "--url", &url_arg]);
    }
    cmd.stdout(Stdio::piped()).stderr(Stdio::piped());

    let mut child = cmd.spawn().map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            "未找到 cloudflared 程序，请先点击「安装 cloudflared」".to_string()
        } else {
            format!("启动服务端隧道失败: {}", e)
        }
    })?;

    let stdout = child.stdout.take();
    let stderr = child.stderr.take();

    let app_clone1 = app.clone();
    if let Some(out) = stdout {
        thread::spawn(move || {
            let reader = BufReader::new(out);
            for line in reader.lines().flatten() {
                // 逐行整理后再转发：丢掉空行、JSON 行只留 message（见 tidy_cloudflared_line）
                if let Some(text) = tidy_cloudflared_line(&line) {
                    let _ = app_clone1.emit(
                        "log-message",
                        LogPayload {
                            message: text,
                            level: "info".to_string(),
                            source: "server".to_string(),
                        },
                    );
                }
            }
        });
    }

    let app_clone2 = app.clone();
    if let Some(err) = stderr {
        thread::spawn(move || {
            let reader = BufReader::new(err);
            for line in reader.lines().flatten() {
                if let Some(text) = tidy_cloudflared_line(&line) {
                    let _ = app_clone2.emit(
                        "log-message",
                        LogPayload {
                            message: text,
                            level: cloudflared_line_level(&line).to_string(),
                            source: "server".to_string(),
                        },
                    );
                }
            }
        });
    }

    proc_guard.insert(name_trimmed.to_string(), child);

    let start_msg = if protocol_trimmed == "hello_world" {
        format!("已启动服务端隧道 [{}]（hello_world 内置测试服务器）", name_trimmed)
    } else if protocol_trimmed == "unix" || protocol_trimmed == "unix+tls" {
        format!(
            "已启动服务端隧道 [{}] 本地转发 [{}]",
            name_trimmed, url_arg
        )
    } else {
        format!(
            "已启动服务端隧道 [{}] 本地转发 [{}://127.0.0.1:{}]",
            name_trimmed, protocol_trimmed, port_trimmed
        )
    };
    let _ = app.emit(
        "log-message",
        LogPayload {
            message: format!("[INFO] {}", start_msg),
            level: "success".to_string(),
            source: "server".to_string(),
        },
    );

    Ok(start_msg)
}

#[tauri::command]
fn stop_server_tunnel(app: AppHandle, state: State<'_, AppState>, name: Option<String>) -> Result<String, String> {
    let mut proc_guard = state.server_process.lock().map_err(|e| e.to_string())?;
    if let Some(n) = name.as_deref().map(|s| s.trim()).filter(|s| !s.is_empty()) {
        // 停止指定名称的隧道
        if let Some(mut child) = proc_guard.remove(n) {
            let _ = child.kill();
            let _ = app.emit(
                "log-message",
                LogPayload {
                    message: format!("[INFO] 服务端隧道 [{}] 已停止", n),
                    level: "warn".to_string(),
                    source: "server".to_string(),
                },
            );
            Ok(format!("服务端隧道 [{}] 已停止", n))
        } else {
            Ok(format!("隧道 [{}] 当前未在运行", n))
        }
    } else {
        // 停止全部
        let count = proc_guard.len();
        for (_, mut child) in proc_guard.drain() {
            let _ = child.kill();
        }
        let _ = app.emit(
            "log-message",
            LogPayload {
                message: format!("[INFO] 已停止全部服务端隧道（共 {} 个）", count),
                level: "warn".to_string(),
                source: "server".to_string(),
            },
        );
        if count > 0 {
            Ok(format!("已停止全部服务端隧道（共 {} 个）", count))
        } else {
            Ok("当前没有正在运行的服务端隧道".to_string())
        }
    }
}

#[tauri::command]
fn start_client_tunnel(
    app: AppHandle,
    state: State<'_, AppState>,
    domain: String,
    port: String,
) -> Result<String, String> {
    let domain_trimmed = domain.trim();
    let port_trimmed = port.trim();

    if domain_trimmed.is_empty() {
        return Err("隧道域名不能为空".to_string());
    }
    if port_trimmed.is_empty() || port_trimmed.parse::<u16>().is_err() {
        return Err("本地监听端口必须为 1-65535 的纯数字".to_string());
    }

    let key = client_tunnel_key(domain_trimmed, port_trimmed);

    let mut proc_guard = state.client_process.lock().map_err(|e| e.to_string())?;
    // 先剔除已自行退出的实例，避免「进程早没了却还占着 key」
    proc_guard.retain(|_k, p| matches!(p.child.try_wait(), Ok(None)));
    if proc_guard.contains_key(&key) {
        return Err(format!(
            "客户端隧道 [{}:{}] 已在运行，无需重复连接",
            domain_trimmed, port_trimmed
        ));
    }
    // --url 指向的是 cloudflared 在本机要「监听」的地址，一个端口只能被一条隧道占用。
    // 两条隧道用同一端口时，后启动的那条会 bind 失败并立刻退出，所以这里提前拦下。
    if let Some(holder) = proc_guard.values().find(|p| p.port == port_trimmed) {
        return Err(format!(
            "本地端口 [{}] 已被隧道 [{}] 占用，请为当前隧道换一个本地端口",
            port_trimmed, holder.domain
        ));
    }

    let url_arg = format!("tcp://127.0.0.1:{}", port_trimmed);
    let mut cmd = create_base_command();
    cmd.args(["access", "tcp", "--hostname", domain_trimmed, "--url", &url_arg])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = cmd.spawn().map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            "未找到 cloudflared 命令，请先点击「安装 cloudflared」".to_string()
        } else {
            format!("启动客户端隧道失败: {}", e)
        }
    })?;

    let stdout = child.stdout.take();
    let stderr = child.stderr.take();

    let app_clone1 = app.clone();
    if let Some(out) = stdout {
        thread::spawn(move || {
            let reader = BufReader::new(out);
            for line in reader.lines().flatten() {
                if let Some(text) = tidy_cloudflared_line(&line) {
                    let _ = app_clone1.emit(
                        "log-message",
                        LogPayload {
                            message: text,
                            level: "info".to_string(),
                            source: "client".to_string(),
                        },
                    );
                }
            }
        });
    }

    let app_clone2 = app.clone();
    if let Some(err) = stderr {
        thread::spawn(move || {
            let reader = BufReader::new(err);
            for line in reader.lines().flatten() {
                if let Some(text) = tidy_cloudflared_line(&line) {
                    let _ = app_clone2.emit(
                        "log-message",
                        LogPayload {
                            message: text,
                            level: cloudflared_line_level(&line).to_string(),
                            source: "client".to_string(),
                        },
                    );
                }
            }
        });
    }

    // spawn 成功不代表隧道可用：监听本机端口的动作发生在 cloudflared 进程内部。
    // 这里等最多 600ms 看进程是否已经退出（端口被别的程序占用、域名无效等都会这样），
    // 一旦退出就直接返回失败，避免界面显示「已连接」而后端其实没有可用进程。
    for _ in 0..12 {
        thread::sleep(std::time::Duration::from_millis(50));
        match child.try_wait() {
            Ok(Some(_)) => {
                // 给 stderr 读取线程留点时间，把 cloudflared 的原始报错刷到控制台
                thread::sleep(std::time::Duration::from_millis(120));
                return Err(format!(
                    "客户端隧道 [{}:{}] 启动后立即退出，请检查本地端口 [{}] 是否已被其它程序占用、或隧道域名是否正确（详细报错见控制台日志）",
                    domain_trimmed, port_trimmed, port_trimmed
                ));
            }
            Ok(None) => {}
            Err(_) => break,
        }
    }

    proc_guard.insert(
        key,
        ClientTunnelProc {
            domain: domain_trimmed.to_string(),
            port: port_trimmed.to_string(),
            child,
        },
    );

    let start_msg = format!("已连接客户端隧道 [{}] -> 本地监听端口 [{}]", domain_trimmed, port_trimmed);
    let _ = app.emit(
        "log-message",
        LogPayload {
            message: format!("[INFO] {}", start_msg),
            level: "success".to_string(),
            source: "client".to_string(),
        },
    );

    Ok(start_msg)
}

#[tauri::command]
fn stop_client_tunnel(
    app: AppHandle,
    state: State<'_, AppState>,
    domain: String,
    port: String,
) -> Result<String, String> {
    let key = client_tunnel_key(&domain, &port);
    let mut proc_guard = state.client_process.lock().map_err(|e| e.to_string())?;
    if let Some(mut entry) = proc_guard.remove(&key) {
        let _ = entry.child.kill();
        let msg = format!(
            "客户端隧道 [{}:{}] 已断开",
            entry.domain, entry.port
        );
        let _ = app.emit(
            "log-message",
            LogPayload {
                message: format!("[INFO] {}", msg),
                level: "warn".to_string(),
                source: "client".to_string(),
            },
        );
        Ok(msg)
    } else {
        Ok("该客户端隧道当前未在运行".to_string())
    }
}

#[tauri::command]
fn is_server_running(state: State<'_, AppState>) -> Vec<String> {
    let mut names = Vec::new();
    if let Ok(mut guard) = state.server_process.lock() {
        // 清理已退出的进程
        guard.retain(|_k, child| match child.try_wait() {
            Ok(None) => true,
            _ => false,
        });
        names = guard.keys().cloned().collect();
    }
    names.sort();
    names
}

#[tauri::command]
fn list_client_tunnels(state: State<'_, AppState>) -> Vec<ClientTunnelState> {
    let mut list = Vec::new();
    if let Ok(mut guard) = state.client_process.lock() {
        // 顺带回收已退出的进程，前端拿到的永远是真的在跑的实例
        guard.retain(|_k, p| matches!(p.child.try_wait(), Ok(None)));
        for (key, p) in guard.iter() {
            list.push(ClientTunnelState {
                key: key.clone(),
                domain: p.domain.clone(),
                port: p.port.clone(),
            });
        }
    }
    list.sort_by(|a, b| a.key.cmp(&b.key));
    list
}

#[tauri::command]
fn check_cloudflared_version() -> Result<String, String> {
    let exe_path = get_cloudflared_executable();
    let mut cmd = create_base_command();
    cmd.arg("--version")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let output = cmd.output().map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            "未找到 cloudflared 可执行程序，请先点击「安装 cloudflared」".to_string()
        } else {
            format!("检查版本失败: {}", e)
        }
    })?;

    let out = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let err = String::from_utf8_lossy(&output.stderr).trim().to_string();

    let version_str = if !out.is_empty() {
        out
    } else if !err.is_empty() {
        err
    } else {
        "未知版本".to_string()
    };

    Ok(format!("{} (路径: {})", version_str, exe_path.display()))
}

#[tauri::command]
fn update_cloudflared(app: AppHandle) -> Result<String, String> {
    // 检查项目目录下是否已有程序
    let exe_path = get_cloudflared_executable();
    let _ = app.emit(
        "log-message",
        LogPayload {
            message: format!("[INFO] 正在检查 cloudflared 更新 (当前目标路径: {})...", exe_path.display()),
            level: "info".to_string(),
            source: "misc".to_string(),
        },
    );

    // 优先调用内置 update 指令
    let mut cmd = create_base_command();
    cmd.arg("update")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let output = cmd.output().map_err(|e| format!("执行更新检查失败: {}", e))?;
    let out = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let err = String::from_utf8_lossy(&output.stderr).trim().to_string();

    let res = if !out.is_empty() { out } else { err };
    let _ = app.emit(
        "log-message",
        LogPayload {
            message: format!("[INFO] 更新检查结果: {}", res),
            level: "info".to_string(),
            source: "misc".to_string(),
        },
    );

    Ok(res)
}

#[tauri::command]
fn download_and_install_cloudflared(
    app: AppHandle,
    download_url: String,
    filename: String,
) -> Result<String, String> {
    let app_c = app.clone();
    thread::spawn(move || {
        let _ = app_c.emit(
            "log-message",
            LogPayload {
                message: format!("[INFO] 开始连接官方源下载 cloudflared: {}", download_url),
                level: "info".to_string(),
                source: "misc".to_string(),
            },
        );

        let agent = ureq::builder()
            .timeout(std::time::Duration::from_secs(60))
            .build();

        let response = match agent
            .get(&download_url)
            .set("User-Agent", "Cloudflare-Tunnel-GUI")
            .call()
        {
            Ok(res) => res,
            Err(err) => {
                let _ = app_c.emit(
                    "log-message",
                    LogPayload {
                        message: format!("[ERROR] 下载请求失败: {}", err),
                        level: "error".to_string(),
                        source: "misc".to_string(),
                    },
                );
                return;
            }
        };

        let total_size: u64 = response
            .header("content-length")
            .and_then(|len| len.parse().ok())
            .unwrap_or(0);

        let mut reader = response.into_reader();
        // 固定装到软件安装目录：下载出来的 cloudflared.exe 与主程序放在一起，
        // 不跟着「启动时的工作目录」乱跑（旧实现用 current_dir，从别处启动会到处乱丢）。
        let target_dir = app_dir();
        let _ = fs::create_dir_all(&target_dir);

        #[cfg(target_os = "windows")]
        let target_exe_name = "cloudflared.exe";
        #[cfg(not(target_os = "windows"))]
        let target_exe_name = "cloudflared";

        let temp_file_path = target_dir.join(format!("{}.tmp", filename));
        let final_exe_path = target_dir.join(target_exe_name);

        let mut file = match File::create(&temp_file_path) {
            Ok(f) => f,
            Err(e) => {
                let _ = app_c.emit(
                    "log-message",
                    LogPayload {
                        message: format!("[ERROR] 创建临时下载文件失败: {}", e),
                        level: "error".to_string(),
                        source: "misc".to_string(),
                    },
                );
                return;
            }
        };

        let mut downloaded: u64 = 0;
        let mut buffer = [0u8; 65536]; // 64KB buffer
        let mut last_percent: u32 = 0;
        let mut last_log_time = std::time::Instant::now();

        loop {
            match reader.read(&mut buffer) {
                Ok(0) => break, // EOF
                Ok(n) => {
                    if let Err(e) = file.write_all(&buffer[..n]) {
                        let _ = app_c.emit(
                            "log-message",
                            LogPayload {
                                message: format!("[ERROR] 写入文件失败: {}", e),
                                level: "error".to_string(),
                                source: "misc".to_string(),
                            },
                        );
                        let _ = fs::remove_file(&temp_file_path);
                        return;
                    }
                    downloaded += n as u64;

                    if total_size > 0 {
                        let percent = ((downloaded as f64 / total_size as f64) * 100.0) as u32;
                        let now = std::time::Instant::now();
                        // 每提升 5% 或每隔 800ms 打印一次下载进度
                        if percent >= last_percent + 5 || now.duration_since(last_log_time).as_millis() > 800 {
                            last_percent = percent;
                            last_log_time = now;
                            let dl_mb = downloaded as f64 / 1024.0 / 1024.0;
                            let total_mb = total_size as f64 / 1024.0 / 1024.0;
                            let _ = app_c.emit(
                                "log-message",
                                LogPayload {
                                    message: format!(
                                        "[INFO] [下载进度] {:>3}% ({:.2} MB / {:.2} MB) 正在下载 {}...",
                                        percent, dl_mb, total_mb, filename
                                    ),
                                    level: "info".to_string(),
                                    source: "misc".to_string(),
                                },
                            );
                        }
                    }
                }
                Err(e) => {
                    let _ = app_c.emit(
                        "log-message",
                        LogPayload {
                            message: format!("[ERROR] 读取下载数据流中断: {}", e),
                            level: "error".to_string(),
                            source: "misc".to_string(),
                        },
                    );
                    let _ = fs::remove_file(&temp_file_path);
                    return;
                }
            }
        }

        drop(file);

        // 重命名临时文件为最终可执行文件
        if let Err(_e) = fs::rename(&temp_file_path, &final_exe_path) {
            // 如果存在 Windows 文件锁，尝试覆盖拷贝
            if let Err(copy_err) = fs::copy(&temp_file_path, &final_exe_path) {
                let _ = app_c.emit(
                    "log-message",
                    LogPayload {
                        message: format!("[ERROR] 安装替换 cloudflared 文件失败: {}", copy_err),
                        level: "error".to_string(),
                        source: "misc".to_string(),
                    },
                );
                let _ = fs::remove_file(&temp_file_path);
                return;
            }
            let _ = fs::remove_file(&temp_file_path);
        }

        // Unix 系统添加执行权限
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = fs::set_permissions(&final_exe_path, fs::Permissions::from_mode(0o755));
        }

        let _ = app_c.emit(
            "log-message",
            LogPayload {
                message: format!("[SUCCESS] [下载进度 100%] cloudflared 安装成功！程序已保存至应用根目录: {}", final_exe_path.display()),
                level: "success".to_string(),
                source: "misc".to_string(),
            },
        );

        // 验证运行安装后的版本
        if let Ok(ver_res) = check_cloudflared_version() {
            let _ = app_c.emit(
                "log-message",
                LogPayload {
                    message: format!("[INFO] 已成功载入运行版本: {}", ver_res),
                    level: "info".to_string(),
                    source: "misc".to_string(),
                },
            );
        }
    });

    Ok("已在后台开始下载与安装流程，请观察控制台实时进度".to_string())
}

/// 授权登录期间，stdout / stderr 两个日志线程共享的一次性状态。
///
/// 背景（真实踩过的 bug：授权页面被打开两次）：
/// `cloudflared tunnel login` **自己就会帮用户打开浏览器**（Cloudflare 官方文档原话：
/// "The client will launch a browser window…"）。本机 cloudflared 2026.9.0 实测，
/// 它的这句自白就是「我有没有帮你打开」的判据：
///   - 正常 PATH（能调起系统打开器）→ `A browser window should have opened at the following URL:`
///   - 清空 PATH（调不起）        → `Please open the following URL and log in with your Cloudflare account:`
///
/// 原实现是 stdout、stderr 各起一个线程，各自持有一个**局部** `opened` 布尔，
/// 逐行扫到 `https://` 就 `open::that(url)`。cloudflared 已经开过一次，我们又开一次，
/// 用户就看到两个授权页面。现在两个线程共用这一份状态：URL 只可能被决策一次，
/// 且默认不再重复打开。
#[derive(Default)]
struct LoginWatch {
    /// 是否已经为这条链接做过决定（跨两条流共享，杜绝重复打开）
    handled: bool,
    /// cloudflared 自述「我已经帮你把浏览器打开了」
    cf_opened: bool,
    /// cloudflared 自述「我打不开，请你自己开」
    cf_failed: bool,
    /// 看到了、但暂时还判不定的授权链接。
    /// 需要它是因为：「我开好了 / 我打不开」那两句和链接本身不保证落在同一条流、
    /// 也不保证链接在后面。先把链接挂起来，等判据到齐再决定，这样无论两条流谁先谁后，
    /// 结论都一样。
    pending_url: Option<String>,
}

/// 逐行观察 cloudflared 输出。
/// 返回需要额外补打的日志（消息, 级别）；不需要补打时返回 None。
fn watch_login_line(line: &str, watch: &Arc<Mutex<LoginWatch>>) -> Option<(String, String)> {
    let mut w = match watch.lock() {
        Ok(guard) => guard,
        // 某个线程 panic 导致锁中毒时，继续用内部数据，别把功能带崩
        Err(poisoned) => poisoned.into_inner(),
    };

    // 判据一：cloudflared 说自己已经帮你打开了。
    if line.contains("should have opened") {
        w.cf_opened = true;
    }
    // 判据二：cloudflared 说自己打不开。
    // 这里千万**不能**改用 "browser failed to open"：那句话在**成功**横幅里也有
    // （"If the browser failed to open, please visit the URL above directly in your
    // browser."）。眼下它是被判据一抢先盖住了才没出事，可一旦哪版 cloudflared 不再
    // 输出 "should have opened" 那半句，它就会把"成功"判成"失败"，又变回开两次。
    if line.contains("Please open the following URL") {
        w.cf_failed = true;
    }

    // 先登记链接（只认第一条），再依据当前掌握的判据决定
    if w.pending_url.is_none() {
        if let Some(idx) = line.find("https://") {
            let url = line[idx..].split_whitespace().next().unwrap_or("").to_string();
            if !url.is_empty() {
                w.pending_url = Some(url);
            }
        }
    }

    if w.handled {
        return None;
    }
    let url = w.pending_url.clone()?;
    if !w.cf_opened && !w.cf_failed {
        // 判据还没到，这次不决定；等后面带判据的行进来再定。
        // 若始终等不到（比如将来 cloudflared 把这两句文案全改了），
        // 就保持「什么都不做」——这段输出里链接本身已经原样打进控制台了，
        // 用户仍能看到并手动打开。宁可让用户手点，也不再冒「又开两次」的风险。
        return None;
    }

    w.handled = true;
    let cf_opened = w.cf_opened;
    drop(w);

    if cf_opened {
        // cloudflared 已经开过了，我们绝不能再开第二次
        Some((
            format!("[INFO] 授权链接已由 cloudflared 自动在浏览器中打开: {}", url),
            "info".to_string(),
        ))
    } else {
        // 走到这里必然意味着 cf_failed（上面已排除「判据未到」的情况）：它打不开，我们补一次
        let _ = open::that(&url);
        Some((
            format!("[INFO] cloudflared 未能自动打开浏览器，已代为打开授权链接: {}", url),
            "success".to_string(),
        ))
    }
}

#[tauri::command]
fn login_cloudflared(app: AppHandle) -> Result<String, String> {
    let mut cmd = create_base_command();
    // 显式把 cert.pem 钉到 cloudflared 的默认目录（~/.cloudflared）。默认行为本来就在这里，
    // 显式指定是为了保证目录存在，同时不受上游默认值变化影响。
    let cert_dir = default_cloudflared_dir();
    let _ = fs::create_dir_all(&cert_dir);
    let cert = cert_dir.join("cert.pem");
    cmd.env("TUNNEL_ORIGIN_CERT", &cert);
    cmd.args(["tunnel", "login"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = cmd.spawn().map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            "未找到 cloudflared 可执行程序，请先点击「安装 cloudflared」".to_string()
        } else {
            format!("执行授权登录失败: {}", e)
        }
    })?;

    let stdout = child.stdout.take();
    let stderr = child.stderr.take();

    let _ = app.emit(
        "log-message",
        LogPayload {
            message: "[INFO] 已启动 Cloudflared 授权，正在获取登录链接...".to_string(),
            level: "info".to_string(),
            source: "misc".to_string(),
        },
    );

    // 两个线程共用，保证授权链接只被决策一次
    let watch = Arc::new(Mutex::new(LoginWatch::default()));

    let app_c1 = app.clone();
    let watch_c1 = Arc::clone(&watch);
    if let Some(out) = stdout {
        thread::spawn(move || {
            let reader = BufReader::new(out);
            for line in reader.lines().flatten() {
                if let Some(text) = tidy_cloudflared_line(&line) {
                    let _ = app_c1.emit(
                        "log-message",
                        LogPayload {
                            message: text,
                            level: "info".to_string(),
                            source: "misc".to_string(),
                        },
                    );
                }
                if let Some((message, level)) = watch_login_line(&line, &watch_c1) {
                    let _ = app_c1.emit(
                        "log-message",
                        LogPayload {
                            message,
                            level,
                            source: "misc".to_string(),
                        },
                    );
                }
            }
        });
    }

    let app_c2 = app.clone();
    let watch_c2 = Arc::clone(&watch);
    if let Some(err) = stderr {
        thread::spawn(move || {
            let reader = BufReader::new(err);
            for line in reader.lines().flatten() {
                if let Some(text) = tidy_cloudflared_line(&line) {
                    let _ = app_c2.emit(
                        "log-message",
                        LogPayload {
                            message: text,
                            level: cloudflared_line_level(&line).to_string(),
                            source: "misc".to_string(),
                        },
                    );
                }
                if let Some((message, level)) = watch_login_line(&line, &watch_c2) {
                    let _ = app_c2.emit(
                        "log-message",
                        LogPayload {
                            message,
                            level,
                            source: "misc".to_string(),
                        },
                    );
                }
            }
        });
    }

    Ok("Cloudflared 授权流程已启动".to_string())
}

#[tauri::command]
fn open_external_url(url: String) -> Result<(), String> {
    open::that(&url).map_err(|e| format!("无法打开链接: {}", e))
}

#[tauri::command]
fn show_main_window(app: AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}


#[tauri::command]
fn minimize_window(window: tauri::WebviewWindow) -> Result<(), String> {
    window.minimize().map_err(|e| e.to_string())
}

#[tauri::command]
fn toggle_maximize_window(window: tauri::WebviewWindow) -> Result<bool, String> {
    let is_max = window.is_maximized().unwrap_or(false);
    if is_max {
        window.unmaximize().map_err(|e| e.to_string())?;
        Ok(false)
    } else {
        window.maximize().map_err(|e| e.to_string())?;
        Ok(true)
    }
}

#[tauri::command]
fn close_window(window: tauri::WebviewWindow) -> Result<(), String> {
    window.hide().map_err(|e| e.to_string())
}

#[tauri::command]
fn is_window_maximized(window: tauri::WebviewWindow) -> bool {
    window.is_maximized().unwrap_or(false)
}

#[tauri::command]
fn exit_app(app: AppHandle, state: State<'_, AppState>) {
    if let Ok(mut guard) = state.server_process.lock() {
        for (_, mut child) in guard.drain() {
            let _ = child.kill();
        }
    }
    if let Ok(mut guard) = state.client_process.lock() {
        for (_, mut entry) in guard.drain() {
            let _ = entry.child.kill();
        }
    }
    if let Ok(mut guard) = state.remote_process.lock() {
        for (_, mut child) in guard.drain() {
            let _ = child.kill();
        }
    }
    if let Ok(mut guard) = state.quick_process.lock() {
        for (_, mut child) in guard.drain() {
            let _ = child.kill();
        }
    }
    app.exit(0);
}

#[tauri::command]
fn open_cloudflared_config_dir(app: AppHandle) -> Result<String, String> {
    // 打开 cloudflared 的默认凭证目录（cert.pem 与隧道密钥都在这里）
    let config_dir = default_cloudflared_dir();

    if !config_dir.exists() {
        let _ = fs::create_dir_all(&config_dir);
    }

    let dir_str = config_dir.to_string_lossy().to_string();

    open::that(&config_dir).map_err(|e| format!("打开配置目录失败: {}", e))?;

    let _ = app.emit(
        "log-message",
        LogPayload {
            message: format!("[INFO] 已在文件资源管理器中打开配置目录: {}", dir_str),
            level: "info".to_string(),
            source: "misc".to_string(),
        },
    );

    Ok(dir_str)
}

/// 从用户输入中提取 Token（支持直接粘贴 token 或完整 install 命令）
fn extract_token(raw: &str) -> Result<String, String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err("Token 不能为空".to_string());
    }
    if trimmed.starts_with("eyJ") {
        return Ok(trimmed.to_string());
    }
    trimmed
        .split_whitespace()
        .find(|s| s.starts_with("eyJ"))
        .map(|s| s.to_string())
        .ok_or_else(|| "无法从输入中提取有效 Token（应以 eyJ 开头）".to_string())
}

/// 从 cloudflared 日志行中解析云端 ingress 配置，返回格式化的规则文本
fn extract_ingress_from_log(line: &str) -> Option<String> {
    let marker = "Updated to new configuration config=\"";
    let start = line.find(marker)? + marker.len();
    let rest = &line[start..];
    let end = rest.find("\" version=")?;
    let escaped = &rest[..end];
    let unescaped = escaped.replace("\\\"", "\"");
    let parsed: serde_json::Value = serde_json::from_str(&unescaped).ok()?;
    let ingress = parsed.get("ingress")?.as_array()?;
    let mut lines = Vec::new();
    for rule in ingress {
        let hostname = rule.get("hostname").and_then(|v| v.as_str()).unwrap_or("(默认)");
        let service = rule.get("service").and_then(|v| v.as_str()).unwrap_or("?");
        lines.push(format!("{}  →  {}", hostname, service));
    }
    if lines.is_empty() {
        return None;
    }
    Some(lines.join("\n"))
}

/// 用隧道 ID 或名称换取该隧道的运行 Token（`cloudflared tunnel token <x>`）。
///
/// Token 存在 Cloudflare 云端，靠 cert.pem 的账号级授权就能随时换出来，
/// 所以界面完全不需要用户手填 Token，也就不必把 Token 明文写进 localStorage。
/// 注意：凭证固定在 `~/.cloudflared`，所以这里必须先确认证书存在，否则 cloudflared 会直接报错。
fn tunnel_token_of(id_or_name: &str) -> Result<String, String> {
    let target = id_or_name.trim();
    if target.is_empty() {
        return Err("隧道 ID 不能为空".to_string());
    }
    if !origin_cert_path().exists() {
        return Err(
            "未找到 Cloudflare 授权证书，请先在「配置」页点击「Cloudflared 授权登录」".to_string(),
        );
    }

    let mut cmd = create_base_command();
    cmd.args(["tunnel", "token", target])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let output = cmd.output().map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            "未找到 cloudflared 程序，请先在「配置」页点击「安装 cloudflared」".to_string()
        } else {
            format!("获取隧道 Token 失败: {}", e)
        }
    })?;

    if !output.status.success() {
        let err_str = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let out_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let reason = if !err_str.is_empty() { err_str } else { out_str };
        return Err(format!("无法获取隧道 [{}] 的 Token: {}", target, reason));
    }

    extract_token(&String::from_utf8_lossy(&output.stdout))
        .map_err(|_| format!("隧道 [{}] 未返回有效 Token", target))
}

/// 按隧道 ID 启动云端托管：先现取 Token，再 `tunnel run --token`。
///
/// 进程表 key 用「隧道 ID」而不是 Token：Token 是每次现取的运行口令，
/// 而隧道 ID 是前端隧道列表里的稳定标识，只有拿 ID 做 key，
/// 前端才能把「正在跑的进程」和「列表里的那一行」对上号。
#[tauri::command]
fn start_remote_tunnel_by_id(
    app: AppHandle,
    state: State<'_, AppState>,
    tunnel_id: String,
    tunnel_name: String,
) -> Result<String, String> {
    let key = tunnel_id.trim().to_string();
    if key.is_empty() {
        return Err("隧道 ID 不能为空".to_string());
    }
    let display = if tunnel_name.trim().is_empty() {
        key.clone()
    } else {
        tunnel_name.trim().to_string()
    };

    // Token 只在本次启动里现取现用：不落盘、不写日志、不上界面
    let token = tunnel_token_of(&display)?;

    let mut proc_guard = state.remote_process.lock().map_err(|e| e.to_string())?;
    if let Some(mut old) = proc_guard.remove(&key) {
        let _ = old.kill();
    }

    let mut cmd = create_base_command();
    cmd.args(["tunnel", "run", "--token", &token])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = cmd.spawn().map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            "未找到 cloudflared 程序，请先在「配置」页点击「安装 cloudflared」".to_string()
        } else {
            format!("启动云端托管失败: {}", e)
        }
    })?;

    let stdout = child.stdout.take();
    let stderr = child.stderr.take();

    let app_clone1 = app.clone();
    if let Some(out) = stdout {
        thread::spawn(move || {
            let reader = BufReader::new(out);
            for line in reader.lines().flatten() {
                if let Some(text) = tidy_cloudflared_line(&line) {
                    let _ = app_clone1.emit(
                        "log-message",
                        LogPayload {
                            message: text,
                            level: "info".to_string(),
                            source: "remote".to_string(),
                        },
                    );
                }
            }
        });
    }

    let app_clone2 = app.clone();
    // 事件必须带上「是哪条隧道的配置」：闭包 move 走副本，key 本身后面还要写进进程表
    let key_for_config = key.clone();
    if let Some(err) = stderr {
        thread::spawn(move || {
            let reader = BufReader::new(err);
            for line in reader.lines().flatten() {
                // 抓 ingress 必须喂**原始行**（配置就在那行 JSON 里），所以在整理之前做
                if let Some(ingress_text) = extract_ingress_from_log(&line) {
                    let _ = app_clone2.emit(
                        "remote-config-update",
                        RemoteConfigUpdate {
                            key: key_for_config.clone(),
                            config: ingress_text,
                        },
                    );
                }
                if let Some(text) = tidy_cloudflared_line(&line) {
                    let _ = app_clone2.emit(
                        "log-message",
                        LogPayload {
                            message: text,
                            level: cloudflared_line_level(&line).to_string(),
                            source: "remote".to_string(),
                        },
                    );
                }
            }
        });
    }

    proc_guard.insert(key, child);

    let start_msg = format!("已启动云端托管 [{}]", display);
    let _ = app.emit(
        "log-message",
        LogPayload {
            message: format!("[INFO] {}", start_msg),
            level: "success".to_string(),
            source: "remote".to_string(),
        },
    );

    Ok(start_msg)
}

#[tauri::command]
fn stop_remote_tunnel(
    app: AppHandle,
    state: State<'_, AppState>,
    key: Option<String>,
    tunnel_name: Option<String>,
) -> Result<String, String> {
    let mut proc_guard = state.remote_process.lock().map_err(|e| e.to_string())?;
    if let Some(k) = key.as_deref().map(|s| s.trim()).filter(|s| !s.is_empty()) {
        if let Some(mut child) = proc_guard.remove(k) {
            let _ = child.kill();
            // 进程表的 key 是隧道 ID，日志里显示 ID 用户看不懂，所以用前端传来的隧道名，
            // 没传就退回 ID（正常路径前端一定会传）。
            let display = tunnel_name
                .as_deref()
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .unwrap_or(k);
            let msg = format!("服务端隧道 [{}] 已停止", display);
            let _ = app.emit(
                "log-message",
                LogPayload {
                    message: format!("[INFO] {}", msg),
                    level: "warn".to_string(),
                    source: "remote".to_string(),
                },
            );
            Ok(msg)
        } else {
            Ok("指定的服务端隧道当前未在运行".to_string())
        }
    } else {
        let count = proc_guard.len();
        for (_, mut child) in proc_guard.drain() {
            let _ = child.kill();
        }
        let _ = app.emit(
            "log-message",
            LogPayload {
                message: format!("[INFO] 已停止全部服务端隧道（共 {} 个）", count),
                level: "warn".to_string(),
                source: "remote".to_string(),
            },
        );
        if count > 0 {
            Ok(format!("已停止全部服务端隧道（共 {} 个）", count))
        } else {
            Ok("当前没有正在运行的服务端隧道".to_string())
        }
    }
}

#[tauri::command]
fn is_remote_running(state: State<'_, AppState>) -> Vec<String> {
    let mut keys = Vec::new();
    if let Ok(mut guard) = state.remote_process.lock() {
        guard.retain(|_k, child| match child.try_wait() {
            Ok(None) => true,
            _ => false,
        });
        keys = guard.keys().cloned().collect();
    }
    keys.sort();
    keys
}

/// 列出当前真正在跑的云端托管隧道（key = 隧道 ID）。
/// 与客户端列表同样先 try_wait() 回收已退出的进程，避免前端拿到假状态。
#[tauri::command]
fn list_remote_tunnels(state: State<'_, AppState>) -> Vec<RemoteTunnelState> {
    let mut keys = Vec::new();
    if let Ok(mut guard) = state.remote_process.lock() {
        guard.retain(|_k, child| match child.try_wait() {
            Ok(None) => true,
            _ => false,
        });
        keys = guard.keys().cloned().collect();
    }
    keys.sort();
    keys.into_iter()
        .map(|key| RemoteTunnelState { key })
        .collect()
}

/// 从一行日志中尝试提取 trycloudflare.com 临时域名。
/// cloudflared 的 quick tunnel 域名可能出现在 stdout 或 stderr 中，
/// 行格式形如: `|  https://xxx.trycloudflare.com  |`，故需两边都检测。
fn try_extract_quick_url(line: &str) -> Option<String> {
    let idx = line.find("trycloudflare.com")?;
    let start = line[..idx].rfind("https://").unwrap_or(0);
    let url = line[start..]
        .split_whitespace()
        .next()
        .unwrap_or("")
        .trim_matches(|c| c == '|' || c == ' ' || c == '*')
        .to_string();
    if url.starts_with("https://") && url.contains("trycloudflare.com") {
        Some(url)
    } else {
        None
    }
}

/// 临时链接（Quick Tunnel）：`cloudflared tunnel --url <协议>://127.0.0.1:<端口>`
/// 免登录、免凭证、免绑定域名，Cloudflare 自动分配一个临时 trycloudflare.com 域名。
/// 支持协议与命名隧道一致：http/https/tcp/ssh/rdp/smb 需端口；
/// unix/unix+tls 需套接字路径（--url unix:/path/to/socket）；
/// hello_world 使用内置测试服务器（cloudflared tunnel --hello-world）。
/// 每次启动域名随机变化，进程停止即失效。
#[tauri::command]
fn start_quick_tunnel(
    app: AppHandle,
    state: State<'_, AppState>,
    port: String,
    protocol: String,
    unix_socket: String,
) -> Result<String, String> {
    let port_trimmed = port.trim();
    let protocol_trimmed = protocol.trim();
    let socket_trimmed = unix_socket.trim();

    if !is_supported_protocol(protocol_trimmed) {
        return Err(format!("不支持的协议类型: {}", protocol_trimmed));
    }

    // 构造 --url 参数：hello_world / unix / unix+tls 无需端口，其余协议需要合法端口
    let url_arg = match protocol_trimmed {
        "hello_world" => String::new(),
        "unix" | "unix+tls" => {
            if socket_trimmed.is_empty() {
                return Err("unix / unix+tls 协议必须填写套接字路径".to_string());
            }
            format!("{}:{}", protocol_trimmed, socket_trimmed)
        }
        _ => {
            if port_trimmed.is_empty() || port_trimmed.parse::<u16>().is_err() {
                return Err("端口号必须为 1-65535 的纯数字".to_string());
            }
            format!("{}://127.0.0.1:{}", protocol_trimmed, port_trimmed)
        }
    };

    // 进程表 key：hello_world 无端口/路径，用固定 key；其余直接用 --url 参数
    let key = if protocol_trimmed == "hello_world" {
        "hello_world".to_string()
    } else {
        url_arg.clone()
    };

    let mut proc_guard = state.quick_process.lock().map_err(|e| e.to_string())?;
    // 同一 key 重复启动时，先停掉旧实例
    if let Some(mut old) = proc_guard.remove(&key) {
        let _ = old.kill();
    }

    let mut cmd = create_base_command();
    if protocol_trimmed == "hello_world" {
        cmd.args(["tunnel", "--hello-world", "--no-autoupdate"]);
    } else {
        cmd.args(["tunnel", "--url", &url_arg, "--no-autoupdate"]);
    }
    cmd.stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = cmd.spawn().map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            "未找到 cloudflared 程序，请先点击「安装 cloudflared」".to_string()
        } else {
            format!("启动临时链接失败: {}", e)
        }
    })?;

    let stdout = child.stdout.take();
    let stderr = child.stderr.take();

    // 从输出流中抓取 trycloudflare.com 临时域名并回传前端
    let app_emit = app.clone();
    let key_stdout = key.clone();
    if let Some(out) = stdout {
        thread::spawn(move || {
            let reader = BufReader::new(out);
            for line in reader.lines().flatten() {
                if let Some(text) = tidy_cloudflared_line(&line) {
                    let _ = app_emit.emit(
                        "log-message",
                        LogPayload {
                            message: text,
                            level: "info".to_string(),
                            source: "quick".to_string(),
                        },
                    );
                }
                if let Some(url) = try_extract_quick_url(&line) {
                    let _ = app_emit.emit(
                        "quick-tunnel-url",
                        QuickUrlPayload { key: key_stdout.clone(), url },
                    );
                }
            }
        });
    }

    let app_emit2 = app.clone();
    let key_stderr = key.clone();
    if let Some(err) = stderr {
        thread::spawn(move || {
            let reader = BufReader::new(err);
            for line in reader.lines().flatten() {
                if let Some(text) = tidy_cloudflared_line(&line) {
                    let _ = app_emit2.emit(
                        "log-message",
                        LogPayload {
                            message: text,
                            level: cloudflared_line_level(&line).to_string(),
                            source: "quick".to_string(),
                        },
                    );
                }
                // cloudflared 的 quick tunnel 域名实际从 stderr 输出，需在此处抓取
                if let Some(url) = try_extract_quick_url(&line) {
                    let _ = app_emit2.emit(
                        "quick-tunnel-url",
                        QuickUrlPayload { key: key_stderr.clone(), url },
                    );
                }
            }
        });
    }

    proc_guard.insert(key.clone(), child);

    let start_desc = if protocol_trimmed == "hello_world" {
        "hello_world 内置测试服务器".to_string()
    } else if protocol_trimmed == "unix" || protocol_trimmed == "unix+tls" {
        format!("{}:{}", protocol_trimmed, socket_trimmed)
    } else {
        format!("{}://127.0.0.1:{}", protocol_trimmed, port_trimmed)
    };
    let start_msg = format!("已启动临时链接 [{}]（临时域名生成中...）", start_desc);
    let _ = app.emit(
        "log-message",
        LogPayload {
            message: format!("[INFO] {}", start_msg),
            level: "success".to_string(),
            source: "quick".to_string(),
        },
    );

    Ok(start_msg)
}

#[tauri::command]
fn stop_quick_tunnel(app: AppHandle, state: State<'_, AppState>, key: Option<String>) -> Result<String, String> {
    let mut proc_guard = state.quick_process.lock().map_err(|e| e.to_string())?;
    if let Some(k) = key.as_deref().map(|s| s.trim()).filter(|s| !s.is_empty()) {
        if let Some(mut child) = proc_guard.remove(k) {
            let _ = child.kill();
            let _ = app.emit(
                "log-message",
                LogPayload {
                    message: format!("[INFO] 临时链接 [{}] 已停止，临时域名已失效", k),
                    level: "warn".to_string(),
                    source: "quick".to_string(),
                },
            );
            Ok(format!("临时链接 [{}] 已停止", k))
        } else {
            Ok("指定的临时链接当前未在运行".to_string())
        }
    } else {
        let count = proc_guard.len();
        for (_, mut child) in proc_guard.drain() {
            let _ = child.kill();
        }
        let _ = app.emit(
            "log-message",
            LogPayload {
                message: format!("[INFO] 已停止全部临时链接（共 {} 个），临时域名已失效", count),
                level: "warn".to_string(),
                source: "quick".to_string(),
            },
        );
        if count > 0 {
            Ok(format!("已停止全部临时链接（共 {} 个）", count))
        } else {
            Ok("当前没有正在运行的临时链接".to_string())
        }
    }
}

#[tauri::command]
fn is_quick_running(state: State<'_, AppState>) -> Vec<String> {
    let mut keys = Vec::new();
    if let Ok(mut guard) = state.quick_process.lock() {
        guard.retain(|_k, child| match child.try_wait() {
            Ok(None) => true,
            _ => false,
        });
        keys = guard.keys().cloned().collect();
    }
    keys.sort();
    keys
}

/// 读取 Cloudflare 凭据（软件目录下的 `cert.pem`，回退旧的 `~/.cloudflared/cert.pem`），
/// 返回 `(apiToken, zoneID)`。
///
/// cert.pem 由 `cloudflared tunnel login` 生成，PEM 正文是 base64 编码的 JSON，
/// 内含 apiToken 与 zoneID。该 token 属于 Cloudflare 的 DNS:Edit 权限组
/// （`cloudflared tunnel route dns` 正是用它创建记录），因此读写删除都可用。
/// 注意：证书只授权单个区域，其它区域的记录既看不到也改不了。
fn cloudflare_credentials() -> Result<(String, String), String> {
    let json = read_origin_cert_json()?;

    let api_token = cert_json_str(&json, &["apiToken", "t"], "API Token")?;
    let zone_id = cert_json_str(&json, &["zoneID"], "zoneID")?;

    Ok((api_token, zone_id))
}

/// 从 cert.pem 解出 `(apiToken, accountID)`。
///
/// 隧道相关接口（列表 / 配置 / 删除）都是**账号级**路径 `/accounts/{account_id}/cfd_tunnel`，
/// 用不上 zoneID，所以单独提供一个取 accountID 的入口。
fn cloudflare_account_credentials() -> Result<(String, String), String> {
    let json = read_origin_cert_json()?;

    let api_token = cert_json_str(&json, &["apiToken", "t"], "API Token")?;
    let account_id = cert_json_str(&json, &["accountID"], "accountID")?;

    Ok((api_token, account_id))
}

/// 解析 cert.pem ：PEM 头尾之间的正文是 base64 编码的 JSON。
fn read_origin_cert_json() -> Result<serde_json::Value, String> {
    let cert_path = origin_cert_path();

    let raw = fs::read_to_string(&cert_path)
        .map_err(|_| "未找到 cert.pem，请先完成「Cloudflared 授权登录」".to_string())?;

    // 提取 PEM 内的 base64 正文
    let b64 = raw
        .lines()
        .filter(|l| !l.starts_with("-----"))
        .collect::<Vec<_>>()
        .join("");
    let decoded = base64_decode(&b64).map_err(|e| format!("cert.pem 解析失败: {}", e))?;
    serde_json::from_str(&decoded).map_err(|e| format!("cert.pem 内容解析失败: {}", e))
}

/// 按候选键名依次取字符串字段，取不到时给出统一格式的可读错误。
fn cert_json_str(json: &serde_json::Value, keys: &[&str], label: &str) -> Result<String, String> {
    keys.iter()
        .find_map(|k| json.get(*k).and_then(|v| v.as_str()))
        .map(|s| s.to_string())
        .ok_or_else(|| format!("cert.pem 中未找到 {}", label))
}

/// 从 Cloudflare API 的错误响应体中提取可读信息（形如 `[81044] Record does not exist.`）。
fn extract_cf_error(body: &str) -> String {
    if let Ok(v) = serde_json::from_str::<serde_json::Value>(body) {
        if let Some(errs) = v.get("errors").and_then(|x| x.as_array()) {
            let msgs: Vec<String> = errs
                .iter()
                .map(|e| {
                    let code = e.get("code").and_then(|c| c.as_i64()).unwrap_or(0);
                    let msg = e.get("message").and_then(|m| m.as_str()).unwrap_or("");
                    if code != 0 {
                        format!("[{}] {}", code, msg)
                    } else {
                        msg.to_string()
                    }
                })
                .filter(|s| !s.trim().is_empty())
                .collect();
            if !msgs.is_empty() {
                return msgs.join("; ");
            }
        }
    }
    body.trim().to_string()
}

/// 发起一次 Cloudflare API 请求并返回响应体文本。
///
/// ureq 会把 4xx/5xx 视为错误，这里统一转成带 Cloudflare 错误码的可读信息，
/// 避免把 `Response code 403` 这种无用提示抛给界面。
fn cf_api_request(
    method: &str,
    url: &str,
    token: &str,
    json_body: Option<&str>,
) -> Result<String, String> {
    let agent = ureq::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build();

    let req = agent
        .request(method, url)
        .set("Authorization", &format!("Bearer {}", token))
        .set("User-Agent", "Cloudflare-Tunnel-GUI");

    let result = match json_body {
        Some(body) => req
            .set("Content-Type", "application/json")
            .send_string(body),
        None => req.call(),
    };

    match result {
        Ok(resp) => resp
            .into_string()
            .map_err(|e| format!("读取 API 响应失败: {}", e)),
        Err(ureq::Error::Status(code, resp)) => {
            let body = resp.into_string().unwrap_or_default();
            Err(format!(
                "Cloudflare API 返回 {}: {}",
                code,
                extract_cf_error(&body)
            ))
        }
        Err(e) => Err(format!("Cloudflare API 请求失败: {}", e)),
    }
}

/// 校验 Cloudflare 隧道 ID（标准 UUID：8-4-4-4-12 十六进制）。
///
/// 该 ID 会被直接拼进请求 URL，必须严格校验以防路径注入。
fn is_valid_tunnel_id(id: &str) -> bool {
    let parts: Vec<&str> = id.split('-').collect();
    parts.len() == 5
        && [8usize, 4, 4, 4, 12]
            .iter()
            .zip(parts.iter())
            .all(|(want, got)| got.len() == *want && got.chars().all(|c| c.is_ascii_hexdigit()))
}

/// 读取某条隧道的云端 ingress 配置（只读）。
///
/// 走 Cloudflare REST API 而不是解析 cloudflared 的运行日志：**隧道不必处于运行状态也能看**，
/// 而且能顺带拿到 `source`（local / cloudflare）与版本号。
/// 凭证直接复用 cert.pem 里的 apiToken（`cloudflared tunnel route dns` 用的就是它），
/// 实测该 token 具备读写隧道配置的权限，因此界面无需让用户另外填 API Token。
#[tauri::command]
fn fetch_tunnel_config(tunnel_id: String) -> Result<TunnelConfig, String> {
    let id = tunnel_id.trim();
    if !is_valid_tunnel_id(id) {
        return Err("隧道 ID 格式不正确".to_string());
    }

    let (api_token, account_id) = cloudflare_account_credentials()?;
    let url = format!(
        "https://api.cloudflare.com/client/v4/accounts/{}/cfd_tunnel/{}/configurations",
        account_id, id
    );
    let body = cf_api_request("GET", &url, &api_token, None)?;
    let json: serde_json::Value =
        serde_json::from_str(&body).map_err(|e| format!("解析 API 响应失败: {}", e))?;
    let result = json
        .get("result")
        .ok_or_else(|| "Cloudflare 未返回该隧道的配置".to_string())?;

    let source = result
        .get("source")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let version = result.get("version").and_then(|v| v.as_u64()).unwrap_or(0);

    let mut rules: Vec<TunnelIngressRule> = Vec::new();
    if let Some(ingress) = result
        .get("config")
        .and_then(|c| c.get("ingress"))
        .and_then(|v| v.as_array())
    {
        for rule in ingress {
            rules.push(TunnelIngressRule {
                hostname: rule
                    .get("hostname")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                path: rule
                    .get("path")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                service: rule
                    .get("service")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
            });
        }
    }

    Ok(TunnelConfig {
        source,
        version,
        rules,
    })
}

/// 读取某条隧道在面板另外两页里的路由（只读）：「主机名路由」与「CIDR 路由」。
///
/// 这两页与 ingress 是**三份彼此独立的数据**，端点也各不相同：
/// - 主机名路由：`GET /accounts/{acct}/zerotrust/routes/hostname`（账号级资源，
///   支持 `tunnel_id` 查询参数过滤；**不是** ingress，也不是 `/cfd_tunnel/{id}/routes`，后者实测 404）；
/// - CIDR 路由：`GET /accounts/{acct}/teamnet/routes`（同样是账号级端点，
///   隧道路径下没有 routes 子资源）。
///
/// 两块都过了 `tunnel_id` 查询参数，返回里再按 `tunnel_id` 过滤一遍兜底，
/// 免得同账号下别的隧道的条目混进来。
///
/// 任一块失败只把原因写进对应的 `*_error`，不让整条命令失败 —— 否则一个 403
/// 会把已经拿到的 ingress 配置一起遮掉（token 缺少 Cloudflare One Networks 权限时就会这样）。
#[tauri::command]
fn fetch_tunnel_routes(tunnel_id: String) -> Result<TunnelRouteSet, String> {
    let id = tunnel_id.trim();
    if !is_valid_tunnel_id(id) {
        return Err("隧道 ID 格式不正确".to_string());
    }

    let (api_token, account_id) = cloudflare_account_credentials()?;

    let hostname_url = format!(
        "https://api.cloudflare.com/client/v4/accounts/{}/zerotrust/routes/hostname?tunnel_id={}&per_page=100",
        account_id, id
    );
    let (hostname_routes, hostname_error) =
        match cf_api_request("GET", &hostname_url, &api_token, None) {
            Ok(body) => match parse_hostname_routes(&body, id) {
                Ok(list) => (list, None),
                Err(e) => (Vec::new(), Some(e)),
            },
            Err(e) => (Vec::new(), Some(e)),
        };

    let cidr_url = format!(
        "https://api.cloudflare.com/client/v4/accounts/{}/teamnet/routes?tunnel_id={}&per_page=100",
        account_id, id
    );
    let (cidr_routes, cidr_error) = match cf_api_request("GET", &cidr_url, &api_token, None) {
        Ok(body) => match parse_cidr_routes(&body, id) {
            Ok(list) => (list, None),
            Err(e) => (Vec::new(), Some(e)),
        },
        Err(e) => (Vec::new(), Some(e)),
    };

    Ok(TunnelRouteSet {
        hostname_routes,
        cidr_routes,
        hostname_error,
        cidr_error,
    })
}

/// 解析主机名路由列表：`{ result: [{ hostname, comment, tunnel_id, deleted_at }] }`。
fn parse_hostname_routes(body: &str, tunnel_id: &str) -> Result<Vec<TunnelHostnameRoute>, String> {
    let json: serde_json::Value =
        serde_json::from_str(body).map_err(|e| format!("解析 API 响应失败: {}", e))?;
    let mut out = Vec::new();
    if let Some(items) = json.get("result").and_then(|v| v.as_array()) {
        for item in items {
            if !route_belongs_to_tunnel(item, tunnel_id) {
                continue;
            }
            out.push(TunnelHostnameRoute {
                hostname: json_str_field(item, "hostname"),
                comment: json_str_field(item, "comment"),
            });
        }
    }
    Ok(out)
}

/// 解析 CIDR 路由列表：`{ result: [{ network, comment, tunnel_id, deleted_at }] }`。
fn parse_cidr_routes(body: &str, tunnel_id: &str) -> Result<Vec<TunnelCidrRoute>, String> {
    let json: serde_json::Value =
        serde_json::from_str(body).map_err(|e| format!("解析 API 响应失败: {}", e))?;
    let mut out = Vec::new();
    if let Some(items) = json.get("result").and_then(|v| v.as_array()) {
        for item in items {
            if !route_belongs_to_tunnel(item, tunnel_id) {
                continue;
            }
            out.push(TunnelCidrRoute {
                network: json_str_field(item, "network"),
                comment: json_str_field(item, "comment"),
            });
        }
    }
    Ok(out)
}

/// 路由条目是否属于指定隧道：未删除，且 `tunnel_id` 相符。
///
/// `tunnel_id` 字段缺失时按「属于」处理并放行：请求已经带了 `tunnel_id` 查询参数，
/// 由服务端做过一次过滤，这里只是兜底，没必要因为字段缺失把内容吞掉。
fn route_belongs_to_tunnel(item: &serde_json::Value, tunnel_id: &str) -> bool {
    let deleted = item.get("deleted_at").map(|v| !v.is_null()).unwrap_or(false);
    if deleted {
        return false;
    }
    let owner = json_str_field(item, "tunnel_id");
    owner.is_empty() || owner == tunnel_id
}

/// 取字符串字段值，缺失或为 null 时给空串（面板的「描述」列本来就可空）。
fn json_str_field(value: &serde_json::Value, key: &str) -> String {
    value
        .get(key)
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string()
}

/// 校验 Cloudflare DNS 记录 ID（32 位十六进制）。
///
/// 该 ID 会被直接拼进请求 URL，必须严格校验以防路径注入。
fn is_valid_record_id(id: &str) -> bool {
    id.len() == 32 && id.chars().all(|c| c.is_ascii_hexdigit())
}

/// 校验域名格式：不能为空、不含空白或协议/路径符号、至少两级、每级字符合法且不以下划线外的符号开头。
fn is_valid_hostname(hostname: &str) -> bool {
    let h = hostname.trim();
    if h.is_empty() || h.len() > 253 {
        return false;
    }
    if h.chars().any(|c| c.is_whitespace()) {
        return false;
    }
    if h.contains("://") || h.contains('/') || h.contains('?') || h.contains('#') || h.contains('@') {
        return false;
    }
    if h.starts_with('.') || h.ends_with('.') || h.starts_with('-') {
        return false;
    }
    let labels: Vec<&str> = h.split('.').collect();
    if labels.len() < 2 {
        return false;
    }
    labels.iter().all(|l| {
        !l.is_empty()
            && l.len() <= 63
            && !l.starts_with('-')
            && !l.ends_with('-')
            && l.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
    })
}

/// 从本机软件目录下的 `cert.pem`（回退 `~/.cloudflared/cert.pem`）提取 Cloudflare API Token，查询
/// 该账号域名下所有指向隧道（*.cfargotunnel.com）的 CNAME 记录，
/// 返回 `{ 隧道ID: [绑定域名...] }` 的映射，供前端隧道列表展示绑定域名。
///
/// 通过 `cloudflared tunnel list --output json` 按隧道名查找其 Cloudflare tunnel_id。
/// 找不到（隧道不存在或列表失败）返回 None，不抛错，供删除前清理域名等场景容错使用。
fn find_tunnel_id_by_name(name: &str) -> Option<String> {
    let mut cmd = create_base_command();
    cmd.args(["tunnel", "list", "--output", "json"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let output = cmd.output().ok()?;
    if !output.status.success() {
        return None;
    }
    let stdout_str = String::from_utf8_lossy(&output.stdout);
    #[derive(Deserialize)]
    struct RawTunnel {
        id: String,
        name: String,
    }
    let raw_list: Vec<RawTunnel> = serde_json::from_str(stdout_str.trim()).ok()?;
    raw_list
        .into_iter()
        .find(|t| t.name == name)
        .map(|t| t.id)
}

/// 一条指向隧道的 DNS 记录，附带其指向的 tunnel_id（从 CNAME content 解析）。
#[derive(Debug, Clone)]
struct RawDnsBinding {
    id: String,
    name: String,
    tunnel_id: String,
}

/// 分页拉取 Cloudflare 区域内全部指向隧道的 CNAME 记录（含 tunnel_id）。
fn fetch_all_dns_bindings() -> Result<Vec<RawDnsBinding>, String> {
    let (api_token, zone_id) = cloudflare_credentials()?;
    let mut out: Vec<RawDnsBinding> = Vec::new();
    let mut page: u64 = 1;

    loop {
        let url = format!(
            "https://api.cloudflare.com/client/v4/zones/{}/dns_records?per_page=100&page={}",
            zone_id, page
        );
        let body = cf_api_request("GET", &url, &api_token, None)?;
        let json: serde_json::Value = serde_json::from_str(&body)
            .map_err(|e| format!("解析 API 响应失败: {}", e))?;

        if let Some(result) = json.get("result").and_then(|v| v.as_array()) {
            for rec in result {
                // 仅关注指向隧道的 CNAME 记录，content 形如 <tunnel-id>.cfargotunnel.com
                let content = rec.get("content").and_then(|v| v.as_str()).unwrap_or("");
                let Some(tunnel_id) = content.strip_suffix(".cfargotunnel.com") else {
                    continue;
                };
                let name = rec.get("name").and_then(|v| v.as_str()).unwrap_or("");
                let id = rec.get("id").and_then(|v| v.as_str()).unwrap_or("");
                if name.is_empty() || id.is_empty() {
                    continue;
                }
                out.push(RawDnsBinding {
                    id: id.to_string(),
                    name: name.to_string(),
                    tunnel_id: tunnel_id.to_string(),
                });
            }
        }

        let total_pages = json
            .get("result_info")
            .and_then(|v| v.get("total_pages"))
            .and_then(|v| v.as_u64())
            .unwrap_or(1);
        if page >= total_pages || page >= 50 {
            break;
        }
        page += 1;
    }

    Ok(out)
}

/// 拉取指向指定 tunnel_id 的全部 DNS 绑定记录，供删除隧道时的批量清理复用。
fn get_hostnames_for_tunnel(tunnel_id: &str) -> Result<Vec<DnsBinding>, String> {
    let all = fetch_all_dns_bindings()?;
    let mut bindings: Vec<DnsBinding> = all
        .into_iter()
        .filter(|b| b.tunnel_id == tunnel_id)
        .map(|b| DnsBinding { id: b.id, name: b.name })
        .collect();
    bindings.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(bindings)
}

/// 删除单条 DNS 记录（解除域名与隧道的绑定），供删除隧道时的批量清理复用。
fn delete_dns_record_by_id(record_id: &str) -> Result<(), String> {
    let trimmed_id = record_id.trim();
    if !is_valid_record_id(trimmed_id) {
        return Err(format!("DNS 记录 ID 格式不正确: {}", trimmed_id));
    }
    let (api_token, zone_id) = cloudflare_credentials()?;
    let url = format!(
        "https://api.cloudflare.com/client/v4/zones/{}/dns_records/{}",
        zone_id, trimmed_id
    );
    let body = cf_api_request("DELETE", &url, &api_token, None)?;
    let json: serde_json::Value =
        serde_json::from_str(&body).map_err(|e| format!("解析 API 响应失败: {}", e))?;
    if json.get("success").and_then(|v| v.as_bool()) != Some(true) {
        return Err(format!("删除域名失败: {}", extract_cf_error(&body)));
    }
    Ok(())
}

/// 每条记录带上 Cloudflare 的 dns_record_id，前端据此执行改名/解绑。
/// 区域记录数可能超过单页上限，按 result_info.total_pages 翻页取全。
#[tauri::command]
fn get_tunnel_hostnames() -> Result<HashMap<String, Vec<DnsBinding>>, String> {
    let all = fetch_all_dns_bindings()?;

    let mut map: HashMap<String, Vec<DnsBinding>> = HashMap::new();
    for b in all {
        map.entry(b.tunnel_id)
            .or_insert_with(Vec::new)
            .push(DnsBinding { id: b.id, name: b.name });
    }

    // 同一隧道下的域名按字典序排列，避免每次刷新顺序漂移
    for v in map.values_mut() {
        v.sort_by(|a, b| a.name.cmp(&b.name));
    }

    Ok(map)
}

/// 修改 DNS 绑定记录的域名（仅重命名 DNS 记录，不改动隧道 ingress 配置）。
///
/// 对应 `PATCH /zones/{zone_id}/dns_records/{record_id}`，只提交 name 字段，
/// 因此记录的 type、content（指向的隧道）、proxied、ttl 等均保持不变。
#[tauri::command]
fn rename_dns_route(record_id: String, hostname: String) -> Result<String, String> {
    let trimmed_id = record_id.trim();
    let trimmed_hostname = hostname.trim();

    if !is_valid_record_id(trimmed_id) {
        return Err("DNS 记录 ID 格式不正确".to_string());
    }
    if !is_valid_hostname(trimmed_hostname) {
        return Err("域名格式不正确 (例如: mc.example.com)".to_string());
    }

    let (api_token, zone_id) = cloudflare_credentials()?;
    let url = format!(
        "https://api.cloudflare.com/client/v4/zones/{}/dns_records/{}",
        zone_id, trimmed_id
    );
    let payload = serde_json::json!({ "name": trimmed_hostname }).to_string();
    let body = cf_api_request("PATCH", &url, &api_token, Some(&payload))?;

    let json: serde_json::Value =
        serde_json::from_str(&body).map_err(|e| format!("解析 API 响应失败: {}", e))?;
    if json.get("success").and_then(|v| v.as_bool()) != Some(true) {
        return Err(format!("修改域名失败: {}", extract_cf_error(&body)));
    }

    Ok(format!("域名已修改为 {}", trimmed_hostname))
}

/// 删除 DNS 绑定记录，即解除域名与隧道的绑定。
///
/// 只删 Cloudflare 侧的 CNAME 记录，不影响隧道本身，也不改动 ingress 配置；
/// 删除后该域名立即无法再通过隧道访问。
#[tauri::command]
fn delete_dns_route(record_id: String) -> Result<String, String> {
    delete_dns_record_by_id(&record_id)?;
    Ok("域名绑定已删除".to_string())
}

/// 简易 base64 解码（标准字符集）
fn base64_decode(input: &str) -> Result<String, String> {
    let alphabet = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut buf = Vec::new();
    let mut acc: u32 = 0;
    let mut bits = 0;
    for c in input.bytes() {
        if c == b'=' {
            break;
        }
        if c.is_ascii_whitespace() {
            continue;
        }
        let idx = alphabet.iter().position(|&a| a == c);
        let val = match idx {
            Some(i) => i as u32,
            None => return Err("包含非法 base64 字符".to_string()),
        };
        acc = (acc << 6) | val;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            buf.push((acc >> bits) as u8);
        }
    }
    Ok(String::from_utf8_lossy(&buf).to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState::default())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            // 已存在实例时，唤起主窗口
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.unminimize();
                let _ = window.set_focus();
            }
        }))
        .setup(|app| {
            // 首次启动把旧的 WebView2 数据目录整体搬到软件目录，避免隧道列表/token 变空。
            // （凭证不在这里处理：cloudflared 的默认目录 ~/.cloudflared 就是它的归宿。）
            migrate_legacy_webview_profile();

            // 便携化：主窗口改由代码创建，把 WebView2 数据目录指定为 <软件目录>\data\webview。
            // 配套要求 tauri.conf.json 里该窗口设置 "create": false，否则会被自动创建、
            // 这里再建会因 label 重复而失败。
            if let Some(window_config) = app.config().app.windows.first().cloned() {
                let mut window_builder =
                    tauri::WebviewWindowBuilder::from_config(app.handle(), &window_config)?;
                // 软件目录不可写时（例如装到 Program Files 又没管理员权限）退回 WebView2 默认
                // 数据目录，宁可数据不集中，也不能让窗口起不来。
                if fs::create_dir_all(webview_data_dir()).is_ok() {
                    window_builder = window_builder.data_directory(webview_data_dir());
                }
                window_builder.build()?;
            }

            // 创建系统托盘右键菜单
            let show_i = MenuItem::with_id(app, "show", "🖥️ 显示主窗口", true, None::<&str>)?;
            let quit_i = MenuItem::with_id(app, "quit", "🚪 退出程序", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_i, &quit_i])?;

            let mut builder = TrayIconBuilder::with_id("main-tray")
                .tooltip("CFTunnel")
                .menu(&menu)
                .show_menu_on_left_click(false);

            if let Some(icon) = app.default_window_icon() {
                builder = builder.icon(icon.clone());
            }

            builder
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.unminimize();
                            let _ = window.set_focus();
                        }
                    }
                    "quit" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.unminimize();
                            let _ = window.set_focus();
                            let _ = window.emit("show-exit-confirm", ());
                        }
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| match event {
                    TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    }
                    | TrayIconEvent::DoubleClick {
                        button: MouseButton::Left,
                        ..
                    } => {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.unminimize();
                            let _ = window.set_focus();
                        }
                    }
                    _ => {}
                })
                .build(app)?;

            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .invoke_handler(tauri::generate_handler![
            list_tunnels,
            create_tunnel,
            delete_tunnel,
            fetch_tunnel_config,
            fetch_tunnel_routes,
            route_dns_tunnel,
            start_server_tunnel,
            stop_server_tunnel,
            start_client_tunnel,
            stop_client_tunnel,
            is_server_running,
            list_client_tunnels,
            check_cloudflared_version,
            update_cloudflared,
            download_and_install_cloudflared,
            login_cloudflared,
            open_external_url,
            open_cloudflared_config_dir,
            show_main_window,
            exit_app,
            minimize_window,
            toggle_maximize_window,
            close_window,
            is_window_maximized,
            start_remote_tunnel_by_id,
            stop_remote_tunnel,
            is_remote_running,
            list_remote_tunnels,
            start_quick_tunnel,
            stop_quick_tunnel,
            is_quick_running,
            get_tunnel_hostnames,
            rename_dns_route,
            delete_dns_route
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
