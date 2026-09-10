/**
 * 基于 Web Audio API 纯代码合成的 UI 音效管理器
 * 包含：playHover (悬浮), playClick (点击), playTab (切换Tab)
 * 支持全局事件代理 (Event Delegation) 和 pointerenter / pointerover 多端手势输入
 */

export class SoundManager {
  private ctx: AudioContext | null = null;
  public enabled: boolean = true;

  /**
   * 初始化 AudioContext (支持浏览器自动播放安全策略恢复)
   */
  public init() {
    try {
      if (!this.ctx) {
        const AudioCtx = window.AudioContext || (window as unknown as { webkitAudioContext: typeof AudioContext }).webkitAudioContext;
        if (AudioCtx) {
          this.ctx = new AudioCtx();
        }
      }
      if (this.ctx && this.ctx.state === 'suspended') {
        this.ctx.resume().catch(() => {});
      }
    } catch (e) {
      console.warn('Web Audio API 不可用:', e);
    }
  }

  /**
   * 合成单音阶音频
   * @param freq 频率 (Hz)
   * @param duration 持续时长 (秒)
   * @param type 波形类型 ('sine' | 'square' | 'sawtooth' | 'triangle')
   * @param vol 音量 (0 ~ 1)
   */
  public playTone(freq: number, duration: number, type: OscillatorType = 'sine', vol = 0.15) {
    if (!this.enabled) return;
    this.init();
    if (!this.ctx) return;

    try {
      const osc = this.ctx.createOscillator();
      const gain = this.ctx.createGain();

      osc.type = type;
      osc.connect(gain);
      gain.connect(this.ctx.destination);

      const now = this.ctx.currentTime;
      osc.frequency.setValueAtTime(freq, now);
      gain.gain.setValueAtTime(vol, now);
      // 平滑衰减至静音，消除杂音爆破
      gain.gain.exponentialRampToValueAtTime(0.0001, now + duration);

      osc.start(now);
      osc.stop(now + duration);
    } catch {
      // 忽略音频异常
    }
  }

  /**
   * 悬浮音效 (1320Hz 轻快短促)
   */
  public playHover() {
    this.playTone(1320, 0.05, 'sine', 0.05);
  }

  /**
   * 普通按钮/链接点击音效 (880Hz 饱满清脆)
   */
  public playClick() {
    this.playTone(880, 0.08, 'sine', 0.1);
  }

  /**
   * 标签页/导航切换音效 (660Hz + 880Hz 双音阶复合)
   */
  public playTab() {
    this.playTone(660, 0.12, 'sine', 0.12);
    setTimeout(() => {
      this.playTone(880, 0.12, 'sine', 0.12);
    }, 45);
  }

  /**
   * 成功/启动音效 (四音阶阶梯上扬)
   */
  public playSuccess() {
    if (!this.enabled) return;
    this.init();
    [523.25, 659.25, 783.99, 1046.50].forEach((f, i) => {
      setTimeout(() => this.playTone(f, 0.3, 'sine', 0.12), i * 80);
    });
  }
}

export const soundManager = new SoundManager();

/**
 * 全局事件代理 (Event Delegation) 自动绑定音效
 * - 拦截 pointerover 事件 (支持鼠标、数位笔悬浮、触控板手势)
 * - 拦截 click 事件 (自动区分 tab 切换与普通按钮点击)
 * 不需要给每个按钮手动书写 @click / @pointerenter
 */
export function initSoundDelegation(root: Document | HTMLElement = document) {
  let lastHoveredElement: Element | null = null;

  const INTERACTIVE_SELECTOR = [
    'button',
    'a',
    '[role="button"]',
    '.fluent-btn',
    '.tile-btn',
    '.fluent-icon-btn',
    '.fluent-dropdown-btn',
    '.fluent-dropdown-item',
    '.nav-tab',
    '.tab-btn',
    '.nav-item',
    '.zoomable',
    '.usdt-address-row',
    '.about-link-item',
    '.console-btn',
    '[data-sound]'
  ].join(',');

  const TAB_SELECTOR = [
    '.nav-tab',
    '.tab-btn',
    '.nav-item',
    '[data-sound="tab"]'
  ].join(',');

  // 1. 悬浮音效监听 (使用 pointerover 代理 pointerenter 行为，完美支持数位笔/触控板/鼠标)
  root.addEventListener(
    'pointerover',
    (e: Event) => {
      const target = (e.target as HTMLElement)?.closest?.(INTERACTIVE_SELECTOR);
      if (target && target !== lastHoveredElement) {
        // 排除已禁用的按钮
        if ((target as HTMLButtonElement).disabled || target.getAttribute('aria-disabled') === 'true') {
          return;
        }
        lastHoveredElement = target;
        soundManager.playHover();
      } else if (!target) {
        lastHoveredElement = null;
      }
    },
    { passive: true }
  );

  root.addEventListener(
    'pointerout',
    (e: Event) => {
      const target = (e.target as HTMLElement)?.closest?.(INTERACTIVE_SELECTOR);
      const related = (e as PointerEvent).relatedTarget as HTMLElement | null;
      if (target && (!related || !target.contains(related))) {
        if (lastHoveredElement === target) {
          lastHoveredElement = null;
        }
      }
    },
    { passive: true }
  );

  // 2. 点击音效监听 (自动判定普通点击与 Tab 点击)
  root.addEventListener(
    'click',
    (e: Event) => {
      // 用户首次交互时激活 AudioContext
      soundManager.init();

      const tabTarget = (e.target as HTMLElement)?.closest?.(TAB_SELECTOR);
      if (tabTarget) {
        if ((tabTarget as HTMLButtonElement).disabled || tabTarget.getAttribute('aria-disabled') === 'true') {
          return;
        }
        soundManager.playTab();
        return;
      }

      const clickTarget = (e.target as HTMLElement)?.closest?.(INTERACTIVE_SELECTOR);
      if (clickTarget) {
        if ((clickTarget as HTMLButtonElement).disabled || clickTarget.getAttribute('aria-disabled') === 'true') {
          return;
        }
        soundManager.playClick();
      }
    },
    { capture: true, passive: true }
  );
}
