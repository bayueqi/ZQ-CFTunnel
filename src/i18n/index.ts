import { LangKey, LangPack } from '../types';
import zh_CN from '../locales/zh_CN.json';
import zh_TW from '../locales/zh_TW.json';
import pt_BR from '../locales/pt_BR.json';
import es_ES from '../locales/es_ES.json';
import en_US from '../locales/en_US.json';
import ja_JP from '../locales/ja_JP.json';

export const LANG_ORDER: { key: LangKey; label: string; flag: string }[] = [
  { key: 'zh_CN', label: '简体中文', flag: '🇨🇳' },
  { key: 'zh_TW', label: '繁體中文', flag: '🇭🇰' },
  { key: 'pt_BR', label: 'Português', flag: '🇧🇷' },
  { key: 'es_ES', label: 'Español', flag: '🇪🇸' },
  { key: 'en_US', label: 'English', flag: '🇺🇸' },
  { key: 'ja_JP', label: '日本語', flag: '🇯🇵' },
];

export const LANG_DATA: Record<LangKey, LangPack> = {
  zh_CN: zh_CN as LangPack,
  zh_TW: zh_TW as LangPack,
  pt_BR: pt_BR as LangPack,
  es_ES: es_ES as LangPack,
  en_US: en_US as LangPack,
  ja_JP: ja_JP as LangPack,
};

/**
 * 当前语言的键。给「没有 Vue 上下文」的模块取文案用
 * —— 目前只有网页演示模式（utils/tauriBridge.ts）需要，
 * 它要在 mock 里给出与界面同语言的日志与吐司。
 * App.vue 在初始化与切换语言时会调用 setActiveLang() 同步。
 */
let activeLangKey: LangKey = 'zh_CN';

/** 同步当前语言；缺省仍是 zh_CN，所以漏调不会崩，只是演示文案回落到中文 */
export const setActiveLang = (key: LangKey): void => {
  if (LANG_DATA[key]) activeLangKey = key;
};

/** 当前语言的文案包（非 Vue 上下文取文案用） */
export const activePack = (): LangPack => LANG_DATA[activeLangKey] || LANG_DATA.zh_CN;

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
