export function isTunnelNameValid(name: string): boolean {
  if (!name || name.trim() === '') return false;
  // 必须仅包含字母 a-z, A-Z，不允许特殊字符、数字、空格
  return /^[a-zA-Z]+$/.test(name.trim());
}

export function isPortValid(port: string): boolean {
  if (!port || port.trim() === '') return false;
  // 必须纯数字且在 1-65535 范围内
  if (!/^\d+$/.test(port.trim())) return false;
  const num = parseInt(port.trim(), 10);
  return num >= 1 && num <= 65535;
}

export function isDomainValid(domain: string): boolean {
  if (!domain || domain.trim() === '') return false;
  const trimmed = domain.trim();
  // 简单验证域名/主机名格式，不允许特殊特殊字符或空格
  const domainRegex = /^([a-zA-Z0-9]([a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?\.)+[a-zA-Z]{2,}$|^localhost$|^(\d{1,3}\.){3}\d{1,3}$/;
  return domainRegex.test(trimmed);
}
