import { LangPack } from '../types';
import zh_CN from '../locales/zh_CN.json';

/**
 * 文案包。曾支持 6 种语言（简中 / 繁中 / 英 / 日 / 西 / 葡），2026-09-22 起只保留简体中文
 * —— 语言下拉菜单、LangKey 联合类型与另外 5 份语言包都已一并移除。
 * 仍保留 LANG_DATA 这层壳：App.vue 里 130 多处日志与吐司写的是 t.value.<tab>.<key>，
 * 演示 mock 写的是 activePack()，有这层壳就一行都不用改。
 */
export const LANG_DATA = { zh_CN: zh_CN as LangPack };

/**
 * 文案包（非 Vue 上下文取文案用）。
 * 目前只有网页演示模式（utils/tauriBridge.ts）需要 —— 它没有 Vue 上下文、取不到 App.vue 的 t，
 * 所以在 mock 里直接调这个函数拿同一份文案。
 */
export const activePack = (): LangPack => LANG_DATA.zh_CN;

/**
 * 填充文案里的 {name} 占位符。
 * 模板里的文案直接插值即可；脚本里拼的日志 / 吐司走这个函数。
 * 缺键时返回空串而不是 "undefined"，避免把 undefined 拼进日志。
 */
export const fmt = (
  tpl: string | undefined,
  vars: Record<string, string | number | undefined> = {}
): string => {
  let s = tpl == null ? '' : String(tpl);
  for (const [k, v] of Object.entries(vars)) {
    s = s.split(`{${k}}`).join(v == null ? '' : String(v));
  }
  return s;
};
