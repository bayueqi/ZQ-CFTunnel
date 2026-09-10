/**
 * Tauri IPC 跨端兼容桥接层 (Tauri Desktop + Web / GitHub Pages 双模式)
 * - 当在 Tauri 桌面端运行时，直接调用原生 Rust IPC 后端；
 * - 当在标准浏览器 (如 GitHub Pages 静态网页) 运行时，自动进入仿真体验模式，让网页端也能获得 100% 完整交互体验。
 */

import { invoke as tauriInvoke } from '@tauri-apps/api/core';
import { listen as tauriListen, EventCallback, UnlistenFn } from '@tauri-apps/api/event';

export const isTauriEnvironment = (): boolean => {
  return typeof window !== 'undefined' && Boolean((window as any).__TAURI_INTERNALS__);
};

// 浏览器演示模式下的内存隧道数据
let mockTunnels = [
  {
    id: 'f83a21b4-49c0-4e2a-b7e1-893d11b0e91a',
    name: 'mc-server',
    created: '2026-08-20 14:32:10',
    connections: '4x Connections (HKG, NRT, SJC, LAX)',
  },
  {
    id: '7a19c53e-1082-4411-9a77-4402ebcf8821',
    name: 'web-demo',
    created: '2026-08-28 09:15:00',
    connections: '2x Connections (HKG, NRT)',
  },
];

type LogListener = (event: { payload: { message: string; level: 'info' | 'warn' | 'error' | 'success'; source: string } }) => void;
const mockLogListeners: LogListener[] = [];

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
        connections: 'Inactive',
      };
      mockTunnels.unshift(newTunnel);
      emitMockLog(`[SUCCESS] 隧道 [${name}] 创建成功！凭证已保存至 ~/.cloudflared/${newTunnel.id}.json`, 'success', 'server');
      return `隧道 [${name}] 创建成功 (ID: ${newTunnel.id})` as unknown as T;
    }

    case 'delete_tunnel': {
      const name = args?.name as string;
      mockTunnels = mockTunnels.filter(t => t.name !== name);
      emitMockLog(`[INFO] 隧道 [${name}] 已从 Cloudflare 网络中注销并删除本地配置文件`, 'info', 'server');
      return `已成功删除隧道 ${name}` as unknown as T;
    }

    case 'start_server_tunnel': {
      const name = (args?.name as string) || 'mc';
      const port = (args?.port as string) || '25565';
      emitMockLog(`[INFO] 正在与 Cloudflare 全球边缘节点建立多路复用连接 (QUIC/HTTP3)...`, 'info', 'server');
      setTimeout(() => {
        emitMockLog(`[INFO] 已在本地 127.0.0.1:${port} 建立入口代理服务`, 'info', 'server');
        emitMockLog(`[SUCCESS] 隧道 [${name}] 已成功连接至 4 个边缘路由节点 (HKG, NRT, SJC, LAX)`, 'success', 'server');
      }, 500);
      return `服务端隧道 [${name}] 启动成功 (端口: ${port})` as unknown as T;
    }

    case 'stop_server_tunnel':
      emitMockLog('[WARN] 服务端隧道连接已主动断开', 'warn', 'server');
      return '服务端隧道已停止' as unknown as T;

    case 'start_client_tunnel': {
      const domain = (args?.domain as string) || 'demo.domain.com';
      const port = (args?.port as string) || '25566';
      emitMockLog(`[INFO] 正在连接远程隧道服务: ${domain}...`, 'info', 'client');
      setTimeout(() => {
        emitMockLog(`[SUCCESS] 客户端反向代理建立成功！本地监听端口: 127.0.0.1:${port}`, 'success', 'client');
      }, 400);
      return `客户端隧道已连接至 ${domain} (本地端口: ${port})` as unknown as T;
    }

    case 'stop_client_tunnel':
      emitMockLog('[WARN] 客户端连接已关闭', 'warn', 'client');
      return '客户端连接已断开' as unknown as T;

    case 'is_server_running':
    case 'is_client_running':
      return false as unknown as T;

    case 'check_cloudflared_version':
      return 'cloudflared version 2024.8.3 (built 2024-08-15-1234 UTC)' as unknown as T;

    case 'update_cloudflared':
      emitMockLog('[INFO] 正在检测 cloudflared 官方最新 Release 版本...', 'info', 'misc');
      setTimeout(() => {
        emitMockLog('[SUCCESS] 当前 cloudflared 版本已是最新版 (2024.8.3)', 'success', 'misc');
      }, 600);
      return 'cloudflared 当前已为最新版本' as unknown as T;

    case 'download_and_install_cloudflared': {
      const filename = (args?.filename as string) || 'cloudflared.exe';
      emitMockLog(`[INFO] 正在从 GitHub 官方下载源拉取: ${filename}...`, 'info', 'misc');
      let progress = 10;
      const interval = setInterval(() => {
        progress += 25;
        if (progress <= 100) {
          emitMockLog(`[DOWNLOAD] 下载进度: ${progress}% / 100%`, 'info', 'misc');
        } else {
          clearInterval(interval);
          emitMockLog(`[SUCCESS] ${filename} 下载完成并已放置在应用目录，已具备执行权限！`, 'success', 'misc');
        }
      }, 350);
      return `已启动 ${filename} 下载任务` as unknown as T;
    }

    case 'login_cloudflared':
      window.open('https://dash.cloudflare.com', '_blank');
      emitMockLog('[INFO] 已在浏览器中打开 Cloudflare 授权页面', 'info', 'misc');
      return '已打开授权页面' as unknown as T;

    case 'open_external_url':
      if (args?.url) {
        window.open(args.url as string, '_blank');
      }
      return 'OK' as unknown as T;

    case 'open_cloudflared_config_dir':
      emitMockLog('[INFO] 模拟打开本地配置目录: C:\\Users\\当前用户\\.cloudflared', 'info', 'misc');
      return '~/.cloudflared' as unknown as T;

    case 'minimize_window':
    case 'toggle_maximize_window':
    case 'close_window':
    case 'is_window_maximized':
      return false as unknown as T;

    case 'exit_app':
      alert('【退出提示】在桌面版中，此操作将安全退出并终止后台隧道服务。');
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

  return () => {
    const idx = mockLogListeners.indexOf(handler as unknown as LogListener);
    if (idx !== -1) mockLogListeners.splice(idx, 1);
  };
}
