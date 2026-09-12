use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Read, Write};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
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
    server_process: Arc<Mutex<Option<Child>>>,
    client_process: Arc<Mutex<Option<Child>>>,
    remote_process: Arc<Mutex<Option<Child>>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TunnelInfo {
    pub id: String,
    pub name: String,
    pub created: String,
    pub connections: String,
    pub tunnel_type: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LogPayload {
    pub message: String,
    pub level: String,
    pub source: String,
}

/// 优先获取本地应用目录下的 cloudflared 可执行文件，不存在时回退到系统环境变量 PATH 中的程序
pub fn get_cloudflared_executable() -> PathBuf {
    #[cfg(target_os = "windows")]
    let exe_name = "cloudflared.exe";
    #[cfg(not(target_os = "windows"))]
    let exe_name = "cloudflared";

    // 1. 检查当前工作目录 / 应用根目录
    if let Ok(cwd) = std::env::current_dir() {
        let local_path = cwd.join(exe_name);
        if local_path.exists() {
            return local_path;
        }
    }

    // 2. 检查主程序自身所在的目录
    if let Ok(current_exe) = std::env::current_exe() {
        if let Some(exe_dir) = current_exe.parent() {
            let local_path = exe_dir.join(exe_name);
            if local_path.exists() {
                return local_path;
            }
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
    cmd
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
            return Err(stderr_str);
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

    let home = std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .map(PathBuf::from)
        .unwrap_or_default();
    let cloudflared_dir = home.join(".cloudflared");

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
        let cred_path = cloudflared_dir.join(format!("{}.json", t.id));
        let tunnel_type = if cred_path.exists() { "local".to_string() } else { "remote".to_string() };
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
        Ok(if out_str.trim().is_empty() { err_str } else { out_str })
    } else {
        Err(if !err_str.trim().is_empty() { err_str } else { out_str })
    }
}

#[tauri::command]
fn delete_tunnel(name: String) -> Result<String, String> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err("隧道名不能为空".to_string());
    }

    let mut cmd = create_base_command();
    cmd.args(["tunnel", "delete", trimmed])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let output = cmd.output().map_err(|e| format!("执行删除命令失败: {}", e))?;
    let out_str = String::from_utf8_lossy(&output.stdout).to_string();
    let err_str = String::from_utf8_lossy(&output.stderr).to_string();

    if output.status.success() {
        Ok(format!("隧道 {} 已成功删除", trimmed))
    } else {
        Err(if !err_str.trim().is_empty() { err_str } else { out_str })
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
        Err(if !err_str.trim().is_empty() { err_str } else { out_str })
    }
}

/// 检查指定名称的隧道是否已存在（通过 cloudflared tunnel list 查询）
fn tunnel_exists(name: &str) -> Result<bool, String> {
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

    if !output.status.success() {
        return Err(if !stderr_str.trim().is_empty() {
            stderr_str
        } else {
            stdout_str
        });
    }

    // stdout 为空说明账号下没有任何隧道
    if stdout_str.trim().is_empty() {
        return Ok(false);
    }

    #[derive(Deserialize)]
    struct RawTunnelName {
        name: String,
    }

    let raw_list: Vec<RawTunnelName> = serde_json::from_str(&stdout_str)
        .map_err(|e| format!("解析隧道列表失败: {}", e))?;

    Ok(raw_list.iter().any(|t| t.name == name))
}

#[tauri::command]
fn start_server_tunnel(
    app: AppHandle,
    state: State<'_, AppState>,
    name: String,
    port: String,
    protocol: String,
) -> Result<String, String> {
    let name_trimmed = name.trim();
    let port_trimmed = port.trim();

    if name_trimmed.is_empty() || !name_trimmed.chars().all(|c| c.is_ascii_alphabetic()) {
        return Err("隧道名字必须为纯字母".to_string());
    }
    if port_trimmed.is_empty() || port_trimmed.parse::<u16>().is_err() {
        return Err("端口号必须为 1-65535 的纯数字".to_string());
    }

    let protocol_trimmed = protocol.trim();
    if protocol_trimmed != "http" && protocol_trimmed != "tcp" {
        return Err("协议必须为 http 或 tcp".to_string());
    }

    // 启动前先确认隧道已存在，避免 cloudflared 旧版快捷语法自动创建隧道
    if !tunnel_exists(name_trimmed)? {
        return Err(format!(
            "隧道 [{}] 不存在，请先点击「创建隧道」创建后再启动",
            name_trimmed
        ));
    }

    let mut proc_guard = state.server_process.lock().map_err(|e| e.to_string())?;
    if let Some(ref mut child) = *proc_guard {
        let _ = child.kill();
        *proc_guard = None;
    }

    let url_arg = format!("{}://127.0.0.1:{}", protocol_trimmed, port_trimmed);
    let mut cmd = create_base_command();
    cmd.args(["tunnel", "--name", name_trimmed, "--url", &url_arg])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

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
                let _ = app_clone1.emit(
                    "log-message",
                    LogPayload {
                        message: line,
                        level: "info".to_string(),
                        source: "server".to_string(),
                    },
                );
            }
        });
    }

    let app_clone2 = app.clone();
    if let Some(err) = stderr {
        thread::spawn(move || {
            let reader = BufReader::new(err);
            for line in reader.lines().flatten() {
                let level = if line.contains("ERR") || line.contains("error") {
                    "error"
                } else if line.contains("WRN") || line.contains("warn") {
                    "warn"
                } else {
                    "info"
                };
                let _ = app_clone2.emit(
                    "log-message",
                    LogPayload {
                        message: line,
                        level: level.to_string(),
                        source: "server".to_string(),
                    },
                );
            }
        });
    }

    *proc_guard = Some(child);

    let start_msg = format!(
        "已启动服务端隧道 [{}] 本地转发 [{}://127.0.0.1:{}]",
        name_trimmed, protocol_trimmed, port_trimmed
    );
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
fn stop_server_tunnel(app: AppHandle, state: State<'_, AppState>) -> Result<String, String> {
    let mut proc_guard = state.server_process.lock().map_err(|e| e.to_string())?;
    if let Some(mut child) = proc_guard.take() {
        let _ = child.kill();
        let msg = "[INFO] 服务端隧道已停止".to_string();
        let _ = app.emit(
            "log-message",
            LogPayload {
                message: msg.clone(),
                level: "warn".to_string(),
                source: "server".to_string(),
            },
        );
        Ok("服务端隧道已停止".to_string())
    } else {
        Ok("当前没有正在运行的服务端隧道".to_string())
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

    let mut proc_guard = state.client_process.lock().map_err(|e| e.to_string())?;
    if let Some(ref mut child) = *proc_guard {
        let _ = child.kill();
        *proc_guard = None;
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
                let _ = app_clone1.emit(
                    "log-message",
                    LogPayload {
                        message: line,
                        level: "info".to_string(),
                        source: "client".to_string(),
                    },
                );
            }
        });
    }

    let app_clone2 = app.clone();
    if let Some(err) = stderr {
        thread::spawn(move || {
            let reader = BufReader::new(err);
            for line in reader.lines().flatten() {
                let level = if line.contains("ERR") || line.contains("error") {
                    "error"
                } else if line.contains("WRN") || line.contains("warn") {
                    "warn"
                } else {
                    "info"
                };
                let _ = app_clone2.emit(
                    "log-message",
                    LogPayload {
                        message: line,
                        level: level.to_string(),
                        source: "client".to_string(),
                    },
                );
            }
        });
    }

    *proc_guard = Some(child);

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
fn stop_client_tunnel(app: AppHandle, state: State<'_, AppState>) -> Result<String, String> {
    let mut proc_guard = state.client_process.lock().map_err(|e| e.to_string())?;
    if let Some(mut child) = proc_guard.take() {
        let _ = child.kill();
        let msg = "[INFO] 客户端连接已断开".to_string();
        let _ = app.emit(
            "log-message",
            LogPayload {
                message: msg.clone(),
                level: "warn".to_string(),
                source: "client".to_string(),
            },
        );
        Ok("客户端连接已断开".to_string())
    } else {
        Ok("当前没有正在运行的客户端连接".to_string())
    }
}

#[tauri::command]
fn is_server_running(state: State<'_, AppState>) -> bool {
    if let Ok(mut guard) = state.server_process.lock() {
        if let Some(ref mut child) = *guard {
            match child.try_wait() {
                Ok(None) => true,
                _ => {
                    *guard = None;
                    false
                }
            }
        } else {
            false
        }
    } else {
        false
    }
}

#[tauri::command]
fn is_client_running(state: State<'_, AppState>) -> bool {
    if let Ok(mut guard) = state.client_process.lock() {
        if let Some(ref mut child) = *guard {
            match child.try_wait() {
                Ok(None) => true,
                _ => {
                    *guard = None;
                    false
                }
            }
        } else {
            false
        }
    } else {
        false
    }
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
        let target_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

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

#[tauri::command]
fn login_cloudflared(app: AppHandle) -> Result<String, String> {
    let mut cmd = create_base_command();
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

    let app_c1 = app.clone();
    if let Some(out) = stdout {
        thread::spawn(move || {
            let reader = BufReader::new(out);
            let mut opened = false;
            for line in reader.lines().flatten() {
                let _ = app_c1.emit(
                    "log-message",
                    LogPayload {
                        message: line.clone(),
                        level: "info".to_string(),
                        source: "misc".to_string(),
                    },
                );
                if line.contains("https://") && !opened {
                    opened = true;
                    if let Some(idx) = line.find("https://") {
                        let url: String = line[idx..].split_whitespace().next().unwrap_or("").to_string();
                        if !url.is_empty() {
                            let _ = open::that(&url);
                            let _ = app_c1.emit(
                                "log-message",
                                LogPayload {
                                    message: format!("[INFO] 已自动在默认浏览器中打开授权链接: {}", url),
                                    level: "success".to_string(),
                                    source: "misc".to_string(),
                                },
                            );
                        }
                    }
                }
            }
        });
    }

    let app_c2 = app.clone();
    if let Some(err) = stderr {
        thread::spawn(move || {
            let reader = BufReader::new(err);
            let mut opened = false;
            for line in reader.lines().flatten() {
                let _ = app_c2.emit(
                    "log-message",
                    LogPayload {
                        message: line.clone(),
                        level: "info".to_string(),
                        source: "misc".to_string(),
                    },
                );
                if line.contains("https://") && !opened {
                    opened = true;
                    if let Some(idx) = line.find("https://") {
                        let url: String = line[idx..].split_whitespace().next().unwrap_or("").to_string();
                        if !url.is_empty() {
                            let _ = open::that(&url);
                            let _ = app_c2.emit(
                                "log-message",
                                LogPayload {
                                    message: format!("[INFO] 已自动在默认浏览器中打开授权链接: {}", url),
                                    level: "success".to_string(),
                                    source: "misc".to_string(),
                                },
                            );
                        }
                    }
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
        if let Some(mut child) = guard.take() {
            let _ = child.kill();
        }
    }
    if let Ok(mut guard) = state.client_process.lock() {
        if let Some(mut child) = guard.take() {
            let _ = child.kill();
        }
    }
    if let Ok(mut guard) = state.remote_process.lock() {
        if let Some(mut child) = guard.take() {
            let _ = child.kill();
        }
    }
    app.exit(0);
}

#[tauri::command]
fn open_cloudflared_config_dir(app: AppHandle) -> Result<String, String> {
    let home = std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .map(PathBuf::from)
        .map_err(|_| "无法获取用户主目录".to_string())?;

    let config_dir = home.join(".cloudflared");

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

#[tauri::command]
fn start_remote_tunnel(
    app: AppHandle,
    state: State<'_, AppState>,
    token: String,
) -> Result<String, String> {
    let token_trimmed = extract_token(&token)?;

    let mut proc_guard = state.remote_process.lock().map_err(|e| e.to_string())?;
    if let Some(ref mut child) = *proc_guard {
        let _ = child.kill();
        *proc_guard = None;
    }

    let mut cmd = create_base_command();
    cmd.args(["tunnel", "run", "--token", &token_trimmed])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = cmd.spawn().map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            "未找到 cloudflared 程序，请先在「配置」页点击「安装 cloudflared」".to_string()
        } else {
            format!("启动远程隧道失败: {}", e)
        }
    })?;

    let stdout = child.stdout.take();
    let stderr = child.stderr.take();

    let app_clone1 = app.clone();
    if let Some(out) = stdout {
        thread::spawn(move || {
            let reader = BufReader::new(out);
            for line in reader.lines().flatten() {
                let _ = app_clone1.emit(
                    "log-message",
                    LogPayload {
                        message: line,
                        level: "info".to_string(),
                        source: "remote".to_string(),
                    },
                );
            }
        });
    }

    let app_clone2 = app.clone();
    if let Some(err) = stderr {
        thread::spawn(move || {
            let reader = BufReader::new(err);
            for line in reader.lines().flatten() {
                let level = if line.contains("ERR") || line.contains("error") {
                    "error"
                } else if line.contains("WRN") || line.contains("warn") {
                    "warn"
                } else {
                    "info"
                };
                if let Some(ingress_text) = extract_ingress_from_log(&line) {
                    let _ = app_clone2.emit("remote-config-update", ingress_text);
                }
                let _ = app_clone2.emit(
                    "log-message",
                    LogPayload {
                        message: line,
                        level: level.to_string(),
                        source: "remote".to_string(),
                    },
                );
            }
        });
    }

    *proc_guard = Some(child);

    let start_msg = "已启动远程隧道（tunnel run --token）".to_string();
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
fn stop_remote_tunnel(app: AppHandle, state: State<'_, AppState>) -> Result<String, String> {
    let mut proc_guard = state.remote_process.lock().map_err(|e| e.to_string())?;
    if let Some(mut child) = proc_guard.take() {
        let _ = child.kill();
        let _ = app.emit(
            "log-message",
            LogPayload {
                message: "[INFO] 远程隧道已停止".to_string(),
                level: "warn".to_string(),
                source: "remote".to_string(),
            },
        );
        Ok("远程隧道已停止".to_string())
    } else {
        Ok("当前没有正在运行的远程隧道".to_string())
    }
}

#[tauri::command]
fn is_remote_running(state: State<'_, AppState>) -> bool {
    state.remote_process.lock().map(|g| g.is_some()).unwrap_or(false)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState::default())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_os::init())
        .setup(|app| {
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
            route_dns_tunnel,
            start_server_tunnel,
            stop_server_tunnel,
            start_client_tunnel,
            stop_client_tunnel,
            is_server_running,
            is_client_running,
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
            start_remote_tunnel,
            stop_remote_tunnel,
            is_remote_running
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
