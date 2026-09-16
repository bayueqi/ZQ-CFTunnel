import { type as getOsType, arch as getCpuArch } from '@tauri-apps/plugin-os';

export interface CloudflaredDownloadTarget {
  os: string;
  arch: string;
  fileName: string;
  downloadUrl: string;
  isArchive: boolean;
  displayName: string;
}

/**
 * 调用 @tauri-apps/plugin-os 插件的 type() 和 arch() API
 * 准确获取当前用户的操作系统 (OS) 和 CPU架构 (Arch)
 * 严格按照 64位 / 32位 架构规范动态返回对应的 cloudflared 官方直链及文件信息
 */
export function getCloudflaredTarget(): CloudflaredDownloadTarget {
  let osType = 'windows';
  let cpuArch = 'x86_64';

  try {
    osType = getOsType();
    cpuArch = getCpuArch();
  } catch (err) {
    console.warn('获取系统/架构信息异常，使用默认 Windows x86_64:', err);
  }

  const os = osType.toLowerCase().trim();
  const arch = cpuArch.toLowerCase().trim();

  // 1. 严格判断 64位 x86 架构 (x86_64, amd64, x64)
  const isX86_64 = arch === 'x86_64' || arch === 'amd64' || arch === 'x64';

  // 2. 严格判断 64位 ARM 架构 (aarch64, arm64)
  const isArm64 = arch === 'aarch64' || arch === 'arm64';

  // 3. 严格判断 32位 x86 架构 (x86, i386, i686, ia32) —— 严禁包含 x86_64
  const isX86_32 = !isX86_64 && (arch === 'x86' || arch === 'i386' || arch === 'i686' || arch === 'ia32');

  // 4. 严格判断 32位 ARM 架构 (arm, armv7, armv7l)
  const isArm32 = !isArm64 && (arch === 'arm' || arch === 'armv7' || arch === 'armv7l');

  let fileName = 'cloudflared-windows-amd64.exe';
  let isArchive = false;
  let displayName = 'Windows x86_64 (64位)';

  // 1. Windows 系统 (OS: windows)
  if (os.includes('windows') || os === 'win32' || os === 'windows_nt') {
    if (isX86_64) {
      fileName = 'cloudflared-windows-amd64.exe';
      displayName = 'Windows x86_64 (64位)';
    } else if (isArm64) {
      fileName = 'cloudflared-windows-arm64.exe';
      displayName = 'Windows ARM64 (64位)';
    } else if (isX86_32) {
      fileName = 'cloudflared-windows-386.exe';
      displayName = 'Windows x86 (32位)';
    } else {
      // 现代 PC 默认回退 64位
      fileName = 'cloudflared-windows-amd64.exe';
      displayName = 'Windows x86_64 (64位)';
    }
  }
  // 2. macOS 系统 (OS: darwin 或 macos)
  else if (os.includes('darwin') || os.includes('macos') || os.includes('ios')) {
    isArchive = true;
    if (isArm64) {
      fileName = 'cloudflared-darwin-arm64.tgz';
      displayName = 'macOS Apple Silicon (M芯片 ARM64)';
    } else if (isX86_64) {
      fileName = 'cloudflared-darwin-amd64.tgz';
      displayName = 'macOS Intel (x86_64 64位)';
    } else {
      fileName = 'cloudflared-darwin-amd64.tgz';
      displayName = 'macOS (x86_64)';
    }
  }
  // 3. Linux 系统 (OS: linux)
  else if (os.includes('linux') || os.includes('android')) {
    if (isX86_64) {
      fileName = 'cloudflared-linux-amd64';
      displayName = 'Linux x86_64 (64位)';
    } else if (isArm64) {
      fileName = 'cloudflared-linux-arm64';
      displayName = 'Linux ARM64 (64位)';
    } else if (isX86_32) {
      fileName = 'cloudflared-linux-386';
      displayName = 'Linux x86 (32位)';
    } else if (isArm32) {
      fileName = 'cloudflared-linux-arm';
      displayName = 'Linux ARMv7 (32位)';
    } else {
      fileName = 'cloudflared-linux-amd64';
      displayName = 'Linux x86_64 (64位)';
    }
  }

  const downloadUrl = `https://github.com/cloudflare/cloudflared/releases/latest/download/${fileName}`;

  return {
    os: osType,
    arch: cpuArch,
    fileName,
    downloadUrl,
    isArchive,
    displayName,
  };
}
