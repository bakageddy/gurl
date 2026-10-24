export type Platform = "mac" | "windows" | "linux";

export function detectPlatform(): Platform {
  const ua = navigator.userAgent;
  if (/Mac/i.test(ua)) return "mac";
  if (/Windows/i.test(ua)) return "windows";
  return "linux";
}

export const platform: Platform = detectPlatform();
/** macOS gets the real native chrome; everything else gets the drawn macOS-style chrome. */
export const nativeChrome = platform === "mac";

export function applyPlatformClass() {
  document.documentElement.classList.add(`platform-${platform}`);
}
