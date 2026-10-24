// Small in-app dialogs, menus and toasts. WKWebView doesn't implement window.prompt, so the
// app draws its own.

export interface PromptSpec {
  title: string;
  label?: string;
  value?: string;
  confirmLabel?: string;
}
export interface ConfirmSpec {
  title: string;
  message: string;
  confirmLabel?: string;
  danger?: boolean;
}
export interface MenuItem {
  label: string;
  action: () => void;
  danger?: boolean;
  separator?: boolean;
  disabled?: boolean;
}

class UI {
  prompt = $state<(PromptSpec & { resolve: (v: string | null) => void }) | null>(null);
  confirmation = $state<(ConfirmSpec & { resolve: (v: boolean) => void }) | null>(null);
  menu = $state<{ x: number; y: number; items: MenuItem[] } | null>(null);
  toasts = $state<Array<{ id: number; text: string; kind: "info" | "error" }>>([]);
  settingsOpen = $state(false);
  private seq = 0;

  ask(spec: PromptSpec): Promise<string | null> {
    return new Promise((resolve) => (this.prompt = { ...spec, resolve }));
  }
  confirm(spec: ConfirmSpec): Promise<boolean> {
    return new Promise((resolve) => (this.confirmation = { ...spec, resolve }));
  }
  openMenu(e: MouseEvent, items: MenuItem[]) {
    e.preventDefault();
    e.stopPropagation();
    this.menu = { x: e.clientX, y: e.clientY, items };
  }
  toast(text: string, kind: "info" | "error" = "info") {
    const id = ++this.seq;
    this.toasts.push({ id, text, kind });
    setTimeout(() => (this.toasts = this.toasts.filter((t) => t.id !== id)), kind === "error" ? 6000 : 2800);
  }
  error(e: unknown) {
    this.toast(typeof e === "string" ? e : e instanceof Error ? e.message : JSON.stringify(e), "error");
  }
}

export const ui = new UI();
