/**
 * Tauri IPC 跨端兼容桥接层 (Tauri Desktop + Web / GitHub Pages 双模式)
 * - 当在 Tauri 桌面端运行时，直接调用原生 Rust IPC 后端；
 * - 当在标准浏览器 (如 GitHub Pages 静态网页) 运行时，自动进入仿真体验模式，让网页端也能获得 100% 完整交互体验。
 */

import { invoke as tauriInvoke } from '@tauri-apps/api/core';
import { listen as tauriListen, EventCallback, UnlistenFn } from '@tauri-apps/api/event';
import { activePack, fmt } from '../i18n';

/**
 * 演示模式的文案（与界面同语言）。
 * 这个模块没有 Vue 实例上下文，取不到 App.vue 里那个 t computed，
 * 所以走 i18n 的 activePack()——App.vue 在启动与切换语言时会同步当前语言。
 */
const L = () => activePack().logs;

export const isTauriEnvironment = (): boolean => {
  return typeof window !== 'undefined' && Boolean((window as any).__TAURI_INTERNALS__);
};

// 浏览器演示模式下的内存隧道数据
// tunnel_type 仍然保留（Rust 侧会返回），但界面已不再按它分列表 ——
// 固定隧道列表展示的是账号下的全部隧道。
// connections 必须是后端的真实格式（每条连接 `1x{colo}`，见 lib.rs 的 read_list_tunnels）：
// 隧道行名称旁的状态点按「本地进程在跑 / 云端还有连接 / 都没有」判三色，
// 以前这里写的 '4x Connections (…)' / 'Inactive' 前端解析不出机房，摘要会显示成乱码。
let mockTunnels = [
  {
    id: 'f83a21b4-49c0-4e2a-b7e1-893d11b0e91a',
    name: 'mc-server',
    created: '2026-08-20 14:32:10',
    connections: '1xhkg13, 1xhkg01, 1xhkg09, 1xhkg11',
    tunnel_type: 'local',
  },
  {
    id: '7a19c53e-1082-4411-9a77-4402ebcf8821',
    name: 'web-demo',
    created: '2026-08-28 09:15:00',
    connections: '1xnrt01, 1xnrt07',
    tunnel_type: 'remote',
  },
];

type LogListener = (event: { payload: { message: string; level: 'info' | 'warn' | 'error' | 'success'; source: string } }) => void;
const mockLogListeners: LogListener[] = [];

// 浏览器演示模式下的客户端隧道（支持多开，与 Rust 侧 client_process 语义一致）
let mockClients: Array<{ key: string; domain: string; port: string }> = [];

// 浏览器演示模式下的云端托管隧道（支持多开，与 Rust 侧 remote_process 语义一致：key = 隧道 ID）
let mockRemotes: Array<{ key: string }> = [];

// 演示模式下模拟后端下发的 ingress 配置（桌面端由 cloudflared 日志解析后推送）
type RemoteConfigListener = (event: { payload: { key: string; config: string } }) => void;
const mockRemoteConfigListeners: RemoteConfigListener[] = [];

// 演示模式下模拟临时隧道分配到的临时域名（桌面端由 cloudflared 日志解析后推送）
type QuickUrlListener = (event: { payload: { key: string; url: string } }) => void;
const mockQuickUrlListeners: QuickUrlListener[] = [];

export function emitMockLog(message: string, level: 'info' | 'warn' | 'error' | 'success' = 'info', source: string = 'system') {
  mockLogListeners.forEach(listener => {
    listener({
      payload: { message, level, source }
    });
  });
}

/**
 * 统一 invoke 调用
 */
export async function safeInvoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  if (isTauriEnvironment()) {
    return await tauriInvoke<T>(cmd, args);
  }

  // === 浏览器静态网页端 (GitHub Pages) 拟真模拟 ===
  console.log(`[Web Demo Mock Invoke] ${cmd}`, args);

  switch (cmd) {
    case 'list_tunnels':
      return [...mockTunnels] as unknown as T;

    case 'create_tunnel': {
      const name = (args?.name as string) || 'custom-tunnel';
      const newTunnel = {
        id: `${Math.random().toString(16).substr(2, 8)}-${Math.random().toString(16).substr(2, 4)}-4${Math.random().toString(16).substr(2, 3)}-${Math.random().toString(16).substr(2, 4)}-${Math.random().toString(16).substr(2, 12)}`,
        name,
        created: new Date().toISOString().replace('T', ' ').substr(0, 19),
        connections: '',
        // 本机新建的隧道带凭据文件，属于「固定域名」那一类
        tunnel_type: 'local',
      };
      mockTunnels.unshift(newTunnel);
      emitMockLog(`[SUCCESS] ${fmt(L().demo_tunnel_created_cred, { name, id: newTunnel.id })}`, 'success', 'server');
      return `${fmt(L().demo_tunnel_created, { name, id: newTunnel.id })}` as unknown as T;
    }

    case 'delete_tunnel': {
      const name = args?.name as string;
      mockTunnels = mockTunnels.filter(t => t.name !== name);
      emitMockLog(`[INFO] ${fmt(L().demo_tunnel_deleted, { name })}`, 'info', 'server');
      return `${fmt(L().demo_tunnel_deleted_toast, { name })}` as unknown as T;
    }

    case 'start_server_tunnel': {
      const name = (args?.name as string) || 'mc';
      const port = (args?.port as string) || '25565';
      emitMockLog(`[INFO] ${L().demo_server_connecting}`, 'info', 'server');
      setTimeout(() => {
        emitMockLog(`[INFO] ${fmt(L().demo_server_proxy_up, { port })}`, 'info', 'server');
        emitMockLog(`[SUCCESS] ${fmt(L().demo_server_connected, { name })}`, 'success', 'server');
      }, 500);
      return `${fmt(L().demo_server_started, { name, port })}` as unknown as T;
    }

    case 'stop_server_tunnel':
      emitMockLog(`[WARN] ${L().demo_server_closed}`, 'warn', 'server');
      return L().demo_server_stopped_toast as unknown as T;

    case 'start_client_tunnel': {
      const domain = ((args?.domain as string) || 'demo.domain.com').trim();
      const port = ((args?.port as string) || '25566').trim();
      const key = `${domain}|${port}`;
      if (mockClients.some(c => c.key === key)) {
        throw new Error(`${fmt(L().demo_client_already, { target: `${domain}:${port}` })}`);
      }
      mockClients.push({ key, domain, port });
      emitMockLog(`[INFO] ${fmt(L().demo_client_connecting, { domain })}...`, 'info', 'client');
      setTimeout(() => {
        emitMockLog(`[SUCCESS] ${fmt(L().demo_client_proxy_up, { port })}`, 'success', 'client');
      }, 400);
      return `${fmt(L().demo_client_connected, { domain, port })}` as unknown as T;
    }

    case 'stop_client_tunnel': {
      const domain = ((args?.domain as string) || '').trim();
      const port = ((args?.port as string) || '').trim();
      const key = `${domain}|${port}`;
      const idx = mockClients.findIndex(c => c.key === key);
      if (idx === -1) {
        return L().demo_client_not_running as unknown as T;
      }
      mockClients.splice(idx, 1);
      emitMockLog(`[INFO] ${fmt(L().demo_client_closed, { target: `${domain}:${port}` })}`, 'warn', 'client');
      return `${fmt(L().demo_client_closed, { target: `${domain}:${port}` })}` as unknown as T;
    }

    case 'list_client_tunnels':
      // 客户端支持多开：返回当前所有在跑的桥接进程快照
      return [...mockClients] as unknown as T;

    // ===== 隧道密码锁（Cloudflare Access）演示模拟 =====
    // 与 Rust 侧 tunnel_lock / tunnel_unlock / tunnel_rotate_password 语义一致
    case 'tunnel_lock': {
      const hostname = ((args?.hostname as string) || '').trim();
      if (!hostname) throw new Error(L().demo_domain_invalid);
      await new Promise(r => setTimeout(r, 700));
      emitMockLog(`[SUCCESS] ${fmt(L().demo_lock_ok, { hostname })}`, 'success', 'server');
      return {
        hostname,
        app_uid: `00000000-0000-4000-8000-${Math.random().toString(16).slice(2, 14).padEnd(12, '0')}`,
        app_name: `CFTunnel-${hostname.split('.')[0]}-demo`,
        token_uid: `00000000-0000-4000-8000-${Math.random().toString(16).slice(2, 14).padEnd(12, '0')}`,
        client_id: `${Math.random().toString(16).slice(2, 34).padEnd(32, '0')}.access`,
        client_secret: `cfast_demo_${Math.random().toString(36).slice(2, 58)}`,
      } as unknown as T;
    }

    case 'tunnel_unlock': {
      await new Promise(r => setTimeout(r, 500));
      emitMockLog(`[SUCCESS] ${L().demo_unlock_ok}`, 'success', 'server');
      return L().demo_unlock_ok_toast as unknown as T;
    }

    case 'tunnel_rotate_password': {
      const hostname = ((args?.hostname as string) || '').trim();
      await new Promise(r => setTimeout(r, 800));
      emitMockLog(`[SUCCESS] ${L().demo_rotate_ok}`, 'success', 'server');
      return {
        hostname,
        app_uid: `00000000-0000-4000-8000-${Math.random().toString(16).slice(2, 14).padEnd(12, '0')}`,
        app_name: `CFTunnel-${hostname.split('.')[0]}-demo`,
        token_uid: `00000000-0000-4000-8000-${Math.random().toString(16).slice(2, 14).padEnd(12, '0')}`,
        client_id: `${Math.random().toString(16).slice(2, 34).padEnd(32, '0')}.access`,
        client_secret: `cfast_demo_${Math.random().toString(36).slice(2, 58)}`,
      } as unknown as T;
    }

    case 'start_remote_tunnel_by_id': {
      const tunnelId = ((args?.tunnelId as string) || '').trim();
      const tunnelName = ((args?.tunnelName as string) || tunnelId).trim();
      if (!tunnelId) throw new Error(L().demo_missing_tunnel_id);
      if (mockRemotes.some(r => r.key === tunnelId)) {
        throw new Error(`${fmt(L().demo_remote_already, { tunnelName })}`);
      }
      mockRemotes.push({ key: tunnelId });
      emitMockLog(`[INFO] ${fmt(L().demo_remote_token, { tunnelName })}...`, 'info', 'remote');
      setTimeout(() => {
        emitMockLog(`[SUCCESS] ${fmt(L().demo_remote_connected, { tunnelName })}`, 'success', 'remote');
        // 桌面端这份配置来自 cloudflared 日志，演示模式直接造一份，否则配置卡片是空的
        mockRemoteConfigListeners.forEach(l => l({
          payload: {
            key: tunnelId,
            config: `${fmt(L().demo_default_config, { tunnelName })}`,
          },
        }));
      }, 600);
      return `${fmt(L().demo_remote_started, { tunnelName })}` as unknown as T;
    }

    case 'stop_remote_tunnel': {
      const key = ((args?.key as string) || '').trim();
      const tunnelName = ((args?.tunnelName as string) || key).trim();
      const idx = mockRemotes.findIndex(r => r.key === key);
      if (idx === -1) {
        return L().demo_server_not_running as unknown as T;
      }
      mockRemotes.splice(idx, 1);
      emitMockLog(`[INFO] ${fmt(L().demo_server_closed_named, { tunnelName })}`, 'warn', 'remote');
      return `${fmt(L().demo_server_closed_named, { tunnelName })}` as unknown as T;
    }

    case 'fetch_tunnel_config': {
      // 只读升级：桌面端走 Cloudflare API 读 ingress，演示模式直接造一份结构相同的数据
      const tunnelId = ((args?.tunnelId as string) || '').trim();
      if (!tunnelId) throw new Error(L().demo_tunnel_id_invalid);
      const tunnel = mockTunnels.find(t => t.id === tunnelId);
      const name = tunnel?.name || tunnelId;
      return {
        source: 'cloudflare',
        version: 12,
        rules: [
          { hostname: `${name}.example.com`, path: '', service: 'http://localhost:25565' },
          { hostname: '', path: '', service: 'http_status:404' },
        ],
      } as unknown as T;
    }

    case 'fetch_tunnel_routes': {
      // 主机名路由 / CIDR 路由：与 ingress 无关的两份独立数据，演示模式各给一种情况
      // （一个有数据、一个空列表），好让两种分支都能在网页端看到。
      // 注意 id 必须有：改名 / 删除都靠它定位，缺了演示端改不动。
      const tunnelId = ((args?.tunnelId as string) || '').trim();
      if (!tunnelId) throw new Error(L().demo_tunnel_id_invalid);
      const tunnel = mockTunnels.find(t => t.id === tunnelId);
      const name = tunnel?.name || tunnelId;
      return {
        hostname_routes: [{ id: `hr-${tunnelId.slice(0, 6)}`, hostname: `${name}.internal`, comment: L().demo_data }],
        cidr_routes: [],
        hostname_error: null,
        cidr_error: null,
      } as unknown as T;
    }

    // ===== 云端配置写入（「创建 / 修改隧道」弹窗的保存路径）演示模拟 =====
    // 与 Rust 侧 update_tunnel_config / *_route 语义一致：只回一句成功文案。
    case 'update_tunnel_config': {
      const ingress = (args?.ingress as unknown[]) || [];
      await new Promise(r => setTimeout(r, 400));
      emitMockLog(`[SUCCESS] ${fmt(L().demo_ingress_written, { count: ingress.length })}`, 'success', 'server');
      return L().demo_config_updated as unknown as T;
    }

    case 'create_hostname_route': {
      const hostname = ((args?.hostname as string) || '').trim();
      await new Promise(r => setTimeout(r, 250));
      emitMockLog(`[SUCCESS] ${fmt(L().demo_host_created, { hostname })}`, 'success', 'server');
      return `${fmt(L().demo_host_created_toast, { hostname })}` as unknown as T;
    }

    case 'delete_hostname_route':
      await new Promise(r => setTimeout(r, 250));
      emitMockLog(`[SUCCESS] ${L().demo_host_deleted}`, 'success', 'server');
      return L().demo_host_deleted_toast as unknown as T;

    case 'create_cidr_route': {
      const network = ((args?.network as string) || '').trim();
      await new Promise(r => setTimeout(r, 250));
      emitMockLog(`[SUCCESS] ${fmt(L().demo_cidr_created, { network })}`, 'success', 'server');
      return `${fmt(L().demo_cidr_created_toast, { network })}` as unknown as T;
    }

    case 'update_cidr_route':
      await new Promise(r => setTimeout(r, 250));
      emitMockLog(`[SUCCESS] ${L().demo_cidr_updated}`, 'success', 'server');
      return L().demo_cidr_updated_toast as unknown as T;

    case 'delete_cidr_route':
      await new Promise(r => setTimeout(r, 250));
      emitMockLog(`[SUCCESS] ${L().demo_cidr_deleted}`, 'success', 'server');
      return L().demo_cidr_deleted_toast as unknown as T;

    // ===== DNS 路由绑定（添加 / 改名 / 解绑）演示模拟 =====
    case 'route_dns_tunnel': {
      const hostname = ((args?.hostname as string) || '').trim();
      const name = ((args?.name as string) || '').trim();
      await new Promise(r => setTimeout(r, 300));
      emitMockLog(`[SUCCESS] ${fmt(L().demo_dns_bound, { hostname, name })}`, 'success', 'server');
      return `${fmt(L().demo_dns_bound_toast, { hostname, name })}` as unknown as T;
    }

    case 'rename_dns_route': {
      const hostname = ((args?.hostname as string) || '').trim();
      await new Promise(r => setTimeout(r, 300));
      emitMockLog(`[SUCCESS] ${fmt(L().demo_dns_renamed, { hostname })}`, 'success', 'server');
      return `${fmt(L().demo_dns_renamed_toast, { hostname })}` as unknown as T;
    }

    case 'delete_dns_route':
      await new Promise(r => setTimeout(r, 300));
      emitMockLog(`[SUCCESS] ${L().demo_dns_unbound}`, 'success', 'server');
      return L().demo_dns_unbound_toast as unknown as T;

    // ===== 临时隧道（quick tunnel）演示模拟 =====
    case 'start_quick_tunnel': {
      const port = ((args?.port as string) || '5244').trim();
      const protocol = ((args?.protocol as string) || 'http').trim();
      // key 必须与 Rust 侧进程表的 key 完全一致：hello_world 固定；
      // unix 用「协议:套接字路径」；其余是「协议://127.0.0.1:端口」
      const unixSocket = ((args?.unixSocket as string) || '').trim();
      const key = protocol === 'hello_world'
        ? 'hello_world'
        : protocol === 'unix' || protocol === 'unix+tls'
          ? `${protocol}:${unixSocket}`
          : `${protocol}://127.0.0.1:${port}`;
      emitMockLog(`[INFO] ${L().demo_quick_requesting}`, 'info', 'quick');
      // 桌面端临时域名由 cloudflared 日志解析后广播 quick-tunnel-url，演示模式照做，
      // 否则列表里那条「生成中...」永远等不到域名。
      setTimeout(() => {
        const url = `https://${Math.random().toString(36).slice(2, 10)}-${Date.now().toString(36).slice(-4)}.trycloudflare.com`;
        mockQuickUrlListeners.forEach(l => l({ payload: { key, url } }));
      }, 900);
      return `${fmt(L().demo_quick_started, { key })}` as unknown as T;
    }

    case 'stop_quick_tunnel': {
      const key = ((args?.key as string) || '').trim();
      emitMockLog(`[INFO] ${fmt(L().demo_quick_stopped, { key: key || L().demo_all })}`, 'warn', 'quick');
      return L().demo_quick_stopped_toast as unknown as T;
    }

    case 'is_quick_running':
      return [] as unknown as T;

    case 'list_remote_tunnels':
      // 云端托管支持多开：返回当前所有在跑的进程快照（key 为隧道 ID）
      return mockRemotes.map(r => ({ key: r.key })) as unknown as T;

    case 'is_remote_running':
      return mockRemotes.map(r => r.key) as unknown as T;

    case 'is_server_running':
      return [] as unknown as T;

    case 'check_cloudflared_version':
      return 'cloudflared version 2024.8.3 (built 2024-08-15-1234 UTC)' as unknown as T;

    case 'update_cloudflared':
      emitMockLog(`[INFO] ${L().demo_checking_version}`, 'info', 'misc');
      setTimeout(() => {
        emitMockLog(`[SUCCESS] ${L().demo_version_latest}`, 'success', 'misc');
      }, 600);
      return L().demo_version_latest_toast as unknown as T;

    case 'download_and_install_cloudflared': {
      const filename = (args?.filename as string) || 'cloudflared.exe';
      emitMockLog(`[INFO] ${fmt(L().demo_downloading, { filename })}...`, 'info', 'misc');
      let progress = 10;
      const interval = setInterval(() => {
        progress += 25;
        if (progress <= 100) {
          emitMockLog(`${fmt(L().demo_download_progress, { progress })}`, 'info', 'misc');
        } else {
          clearInterval(interval);
          emitMockLog(`[SUCCESS] ${fmt(L().demo_download_done, { filename })}`, 'success', 'misc');
        }
      }, 350);
      return `${fmt(L().demo_download_started, { filename })}` as unknown as T;
    }

    case 'login_cloudflared':
      window.open('https://dash.cloudflare.com', '_blank');
      emitMockLog(`[INFO] ${L().demo_login_page}`, 'info', 'misc');
      return L().demo_login_page_toast as unknown as T;

    case 'open_external_url':
      if (args?.url) {
        window.open(args.url as string, '_blank');
      }
      return 'OK' as unknown as T;

    case 'open_cloudflared_config_dir':
      emitMockLog(`[INFO] ${L().demo_open_config_dir}`, 'info', 'misc');
      return '%USERPROFILE%\\.cloudflared' as unknown as T;

    case 'minimize_window':
    case 'toggle_maximize_window':
    case 'close_window':
    case 'is_window_maximized':
      return false as unknown as T;

    case 'exit_app':
      alert(L().demo_exit_hint);
      return 'OK' as unknown as T;

    default:
      return null as unknown as T;
  }
}

/**
 * 统一 listen 监听
 */
export async function safeListen<T>(event: string, handler: EventCallback<T>): Promise<UnlistenFn> {
  if (isTauriEnvironment()) {
    return await tauriListen<T>(event, handler);
  }

  // 网页端 mock 事件分发
  if (event === 'log-message') {
    mockLogListeners.push(handler as unknown as LogListener);
  }

  if (event === 'remote-config-update') {
    mockRemoteConfigListeners.push(handler as unknown as RemoteConfigListener);
    return () => {
      const idx = mockRemoteConfigListeners.indexOf(handler as unknown as RemoteConfigListener);
      if (idx !== -1) mockRemoteConfigListeners.splice(idx, 1);
    };
  }

  if (event === 'quick-tunnel-url') {
    mockQuickUrlListeners.push(handler as unknown as QuickUrlListener);
    return () => {
      const idx = mockQuickUrlListeners.indexOf(handler as unknown as QuickUrlListener);
      if (idx !== -1) mockQuickUrlListeners.splice(idx, 1);
    };
  }

  return () => {
    const idx = mockLogListeners.indexOf(handler as unknown as LogListener);
    if (idx !== -1) mockLogListeners.splice(idx, 1);
  };
}
