/**
 * 酒狐彩蛋音效
 * 点击 Cloudflared Logo 时随机播放一段
 *
 * 【为什么音频不写成 base64 常量】
 * 这里原先是 9 段 base64 字符串的数组，共 4.17 MB。它被 App.vue 静态 import，
 * 于是整块进了 main bundle（构建产物 index-*.js 实测 4.45 MB），
 * 并且以字符串形式常驻 V8 堆 —— 而这份数据只在用户点击 Logo 时才可能用到。
 *
 * 现在改为引用独立音频资源（同目录 ./easter/），由构建工具作为纯资源处理：
 *   - 主包不再携带这 4.17 MB
 *   - 音频在首次点 Logo 时由 webview 按需加载，不看彩蛋就不进内存
 *
 * 【音频本身也压过】
 * 原文件是 48kHz / 立体声 / 16bit。内容是语音，能量集中在 300-3400Hz，
 * 遂重采样为 24kHz / 单声道（仍保留到 12kHz 带宽），体积 4.17 MB -> 0.78 MB，
 * 听感无实质差异。
 */
import jiuhu1 from './easter/jiuhu-1.wav';
import jiuhu2 from './easter/jiuhu-2.wav';
import jiuhu3 from './easter/jiuhu-3.wav';
import jiuhu4 from './easter/jiuhu-4.wav';
import jiuhu5 from './easter/jiuhu-5.wav';
import jiuhu6 from './easter/jiuhu-6.wav';
import jiuhu7 from './easter/jiuhu-7.wav';
import jiuhu8 from './easter/jiuhu-8.wav';
import jiuhu9 from './easter/jiuhu-9.wav';

export const JIUHU_AUDIOS: string[] = [
  jiuhu1,
  jiuhu2,
  jiuhu3,
  jiuhu4,
  jiuhu5,
  jiuhu6,
  jiuhu7,
  jiuhu8,
  jiuhu9,
];

let currentAudio: HTMLAudioElement | null = null;

/**
 * 随机播放一段酒狐语音彩蛋
 */
export function playRandomEasterEggSound(enabled = true): void {
  if (!enabled || JIUHU_AUDIOS.length === 0) return;

  try {
    if (currentAudio) {
      currentAudio.pause();
      // 置空 src 并 load()，让上一段音频立刻释放已缓冲的媒体数据，
      // 否则它会一直挂在元素上等待被 GC，多听几段就多占几份。
      currentAudio.removeAttribute('src');
      currentAudio.load();
      currentAudio = null;
    }

    const randomIndex = Math.floor(Math.random() * JIUHU_AUDIOS.length);
    const audioSrc = JIUHU_AUDIOS[randomIndex];

    currentAudio = new Audio(audioSrc);
    currentAudio.volume = 0.55;
    currentAudio.play().catch(e => {
      console.warn('播放酒狐彩蛋音效失败:', e);
    });
  } catch (err) {
    console.warn('初始化酒狐彩蛋音效异常:', err);
  }
}
