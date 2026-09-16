import fs from 'fs';
import path from 'path';
import https from 'https';
import { execSync } from 'child_process';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const rootDir = path.resolve(__dirname, '..');

const CLOUDFLARED_RELEASE_BASE = 'https://github.com/cloudflare/cloudflared/releases/latest/download';

/**
 * 8大目标架构与 Cloudflare 官方 Release 文件及 Tauri Target Triple 映射规范
 */
const TARGET_MAP = {
  // Windows
  'x86_64-pc-windows-msvc': {
    remoteFile: 'cloudflared-windows-amd64.exe',
    destFile: 'cloudflared-x86_64-pc-windows-msvc.exe',
    isTgz: false,
  },
  'i686-pc-windows-msvc': {
    remoteFile: 'cloudflared-windows-386.exe',
    destFile: 'cloudflared-i686-pc-windows-msvc.exe',
    isTgz: false,
  },
  // macOS
  'x86_64-apple-darwin': {
    remoteFile: 'cloudflared-darwin-amd64.tgz',
    destFile: 'cloudflared-x86_64-apple-darwin',
    isTgz: true,
  },
  'aarch64-apple-darwin': {
    remoteFile: 'cloudflared-darwin-arm64.tgz',
    destFile: 'cloudflared-aarch64-apple-darwin',
    isTgz: true,
  },
  // Linux
  'x86_64-unknown-linux-gnu': {
    remoteFile: 'cloudflared-linux-amd64',
    destFile: 'cloudflared-x86_64-unknown-linux-gnu',
    isTgz: false,
  },
  'i686-unknown-linux-gnu': {
    remoteFile: 'cloudflared-linux-386',
    destFile: 'cloudflared-i686-unknown-linux-gnu',
    isTgz: false,
  },
  'aarch64-unknown-linux-gnu': {
    remoteFile: 'cloudflared-linux-arm64',
    destFile: 'cloudflared-aarch64-unknown-linux-gnu',
    isTgz: false,
  },
  'armv7-unknown-linux-gnueabihf': {
    remoteFile: 'cloudflared-linux-arm',
    destFile: 'cloudflared-armv7-unknown-linux-gnueabihf',
    isTgz: false,
  },
};

/**
 * 跨平台下载文件并支持跟随 GitHub 302 重定向
 */
function downloadFile(url, destPath) {
  return new Promise((resolve, reject) => {
    function get(currentUrl, redirectCount = 0) {
      if (redirectCount > 10) {
        return reject(new Error(`重定向次数过多: ${url}`));
      }
      https.get(currentUrl, (res) => {
        if (res.statusCode >= 300 && res.statusCode < 400 && res.headers.location) {
          return get(res.headers.location, redirectCount + 1);
        }
        if (res.statusCode !== 200) {
          return reject(new Error(`下载失败 [HTTP ${res.statusCode}]: ${currentUrl}`));
        }
        const fileStream = fs.createWriteStream(destPath);
        res.pipe(fileStream);
        fileStream.on('finish', () => {
          fileStream.close();
          resolve();
        });
        fileStream.on('error', (err) => {
          fs.unlink(destPath, () => {});
          reject(err);
        });
      }).on('error', reject);
    }
    get(url);
  });
}

/**
 * 确保目录存在
 */
function ensureDir(dir) {
  if (!fs.existsSync(dir)) {
    fs.mkdirSync(dir, { recursive: true });
  }
}

async function processTarget(triple, info, binDir, tauriBinDir) {
  const downloadUrl = `${CLOUDFLARED_RELEASE_BASE}/${info.remoteFile}`;
  const tempFilePath = path.join(binDir, info.remoteFile);
  const targetBinaryPath = path.join(binDir, info.destFile);
  const tauriTargetBinaryPath = path.join(tauriBinDir, info.destFile);

  console.log(`\n========================================`);
  console.log(`[🚀 开始处理] Target: ${triple}`);
  console.log(`- 远端地址: ${downloadUrl}`);
  console.log(`- 目标文件名: ${info.destFile}`);

  if (fs.existsSync(targetBinaryPath)) {
    console.log(`[✔ 存在] 已存在缓存文件，跳过下载: ${info.destFile}`);
    fs.copyFileSync(targetBinaryPath, tauriTargetBinaryPath);
    return;
  }

  console.log(`[⬇ 下载中] 正在从 GitHub Releases 拉取: ${info.remoteFile}...`);
  await downloadFile(downloadUrl, tempFilePath);

  if (info.isTgz) {
    console.log(`[📦 解压中] 解压 macOS tgz 归档: ${info.remoteFile}...`);
    const extractTempDir = path.join(binDir, `temp_${triple}`);
    ensureDir(extractTempDir);
    execSync(`tar -xzf "${tempFilePath}" -C "${extractTempDir}"`, { stdio: 'inherit' });
    
    const extractedBinary = path.join(extractTempDir, 'cloudflared');
    if (!fs.existsSync(extractedBinary)) {
      throw new Error(`在解压目录中未找到 cloudflared 二进制文件: ${extractTempDir}`);
    }
    fs.renameSync(extractedBinary, targetBinaryPath);
    
    fs.rmSync(extractTempDir, { recursive: true, force: true });
    fs.unlinkSync(tempFilePath);
  } else {
    fs.renameSync(tempFilePath, targetBinaryPath);
  }

  // 设置可执行权限 (Unix)
  if (!info.destFile.endsWith('.exe')) {
    try {
      fs.chmodSync(targetBinaryPath, 0o755);
    } catch {}
  }

  // 复制到 src-tauri/binaries/ 保证双目录完备
  fs.copyFileSync(targetBinaryPath, tauriTargetBinaryPath);
  if (!info.destFile.endsWith('.exe')) {
    try {
      fs.chmodSync(tauriTargetBinaryPath, 0o755);
    } catch {}
  }

  console.log(`[✔ 完成] 已成功生成并重命名: ${info.destFile}`);
}

async function main() {
  const args = process.argv.slice(2);
  let requestedTarget = null;

  for (const arg of args) {
    if (arg.startsWith('--target=')) {
      requestedTarget = arg.split('=')[1].trim();
    }
  }

  const binDir = path.join(rootDir, 'binaries');
  const tauriBinDir = path.join(rootDir, 'src-tauri', 'binaries');
  ensureDir(binDir);
  ensureDir(tauriBinDir);

  if (requestedTarget) {
    const info = TARGET_MAP[requestedTarget];
    if (!info) {
      console.error(`[❌ 错误] 不支持的目标架构: ${requestedTarget}`);
      console.error(`支持的列表: ${Object.keys(TARGET_MAP).join(', ')}`);
      process.exit(1);
    }
    await processTarget(requestedTarget, info, binDir, tauriBinDir);
  } else {
    console.log(`[ℹ 模式] 未指定单一 target，开始批量下载并准备全部 8 种架构二进制...`);
    for (const [triple, info] of Object.entries(TARGET_MAP)) {
      await processTarget(triple, info, binDir, tauriBinDir);
    }
  }

  console.log(`\n🎉 [ALL DONE] 全部 Sidecar 二进制已准备就绪！`);
}

main().catch((err) => {
  console.error(`\n❌ [Fatal Error]`, err);
  process.exit(1);
});
