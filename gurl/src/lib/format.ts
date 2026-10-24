export function bytes(n: number): string {
  if (n < 1024) return `${n} B`;
  if (n < 1024 * 1024) return `${(n / 1024).toFixed(1)} KB`;
  return `${(n / 1024 / 1024).toFixed(1)} MB`;
}

export function ms(n: number): string {
  if (n < 1) return "<1 ms";
  if (n < 1000) return `${Math.round(n)} ms`;
  return `${(n / 1000).toFixed(2)} s`;
}

export function basename(path: string): string {
  const p = path.replace(/\\/g, "/");
  return p.slice(p.lastIndexOf("/") + 1) || path;
}

export function dirname(path: string): string {
  const p = path.replace(/\\/g, "/");
  const i = p.lastIndexOf("/");
  if (i < 0) return "";
  // keep the original separators
  return path.slice(0, i === 0 ? 1 : i);
}

export function relative(root: string, path: string): string {
  const r = root.replace(/\\/g, "/").replace(/\/$/, "");
  const p = path.replace(/\\/g, "/");
  return p.startsWith(r + "/") ? p.slice(r.length + 1) : p;
}

export function joinPath(dir: string, name: string): string {
  const sep = dir.includes("\\") && !dir.includes("/") ? "\\" : "/";
  return dir.replace(/[\\/]$/, "") + sep + name;
}

export function prettyJson(text: string): string | null {
  try {
    return JSON.stringify(JSON.parse(text), null, 2);
  } catch {
    return null;
  }
}

export function timeAgo(epochSecs: number): string {
  const d = Date.now() / 1000 - epochSecs;
  if (d < 60) return "just now";
  if (d < 3600) return `${Math.floor(d / 60)} min ago`;
  if (d < 86400) return `${Math.floor(d / 3600)} h ago`;
  return new Date(epochSecs * 1000).toLocaleDateString();
}

export function debounce<A extends unknown[]>(fn: (...a: A) => void, wait: number) {
  let t: ReturnType<typeof setTimeout> | undefined;
  const run = (...a: A) => {
    clearTimeout(t);
    t = setTimeout(() => fn(...a), wait);
  };
  run.cancel = () => clearTimeout(t);
  return run;
}
