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
