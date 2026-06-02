export function formatNumber(value: number) {
  return new Intl.NumberFormat().format(value);
}

export function formatBytes(bytes: number) {
  if (!Number.isFinite(bytes)) {
    return "-";
  }

  const units = ["B", "KB", "MB", "GB", "TB", "PB"];
  let value = bytes;
  let unit = 0;

  while (value >= 1024 && unit < units.length - 1) {
    value /= 1024;
    unit += 1;
  }

  const digits = value >= 10 || unit === 0 ? 0 : 1;
  return `${value.toFixed(digits)} ${units[unit]}`;
}

export function shortPath(path: string) {
  const normalized = path.replaceAll("\\", "/");
  const parts = normalized.split("/").filter(Boolean);
  return parts.at(-1) ?? path;
}

export function previewValue(value: unknown) {
  let text: string;

  try {
    text = JSON.stringify(value);
  } catch {
    text = String(value);
  }

  if (!text) {
    return "empty";
  }

  return text.length > 120 ? `${text.slice(0, 120)}...` : text;
}
