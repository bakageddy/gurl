import { getSettings, setSettings } from "../api";
import { DEFAULT_RUN_OPTIONS, type RunOptions } from "../types";
import { debounce } from "../format";

export type ThemePref = "system" | "light" | "dark";

class Settings {
  theme = $state<ThemePref>("system");
  sidebarWidth = $state(264);
  resultWidth = $state(46); // percent of the main area
  jobs = $state(4);
  recursive = $state(true);
  runOptions = $state<RunOptions>({ ...DEFAULT_RUN_OPTIONS });
  lastWorkspace = $state<string | null>(null);
  loaded = $state(false);

  private persist = debounce(() => {
    setSettings($state.snapshot({
      theme: this.theme,
      sidebarWidth: this.sidebarWidth,
      resultWidth: this.resultWidth,
      jobs: this.jobs,
      recursive: this.recursive,
      runOptions: this.runOptions,
      lastWorkspace: this.lastWorkspace,
    }) as Record<string, unknown>).catch(() => {});
  }, 400);

  async load() {
    try {
      const s = (await getSettings()) as Record<string, any> | null;
      if (s) {
        this.theme = s.theme ?? this.theme;
        this.sidebarWidth = s.sidebarWidth ?? this.sidebarWidth;
        this.resultWidth = s.resultWidth ?? this.resultWidth;
        this.jobs = s.jobs ?? this.jobs;
        this.recursive = s.recursive ?? this.recursive;
        this.runOptions = { ...DEFAULT_RUN_OPTIONS, ...(s.runOptions ?? {}) };
        this.lastWorkspace = s.lastWorkspace ?? null;
      }
    } catch {
      /* first run */
    }
    this.apply();
    this.loaded = true;
  }

  apply() {
    const el = document.documentElement;
    if (this.theme === "system") el.removeAttribute("data-theme");
    else el.setAttribute("data-theme", this.theme);
  }

  save() {
    this.apply();
    this.persist();
  }
}

export const settings = new Settings();
