import {
  closeWorkspace,
  createDir,
  createFile,
  deletePath,
  duplicatePath,
  listDir,
  openWorkspace,
  renamePath,
} from "../api";
import type { TreeNode } from "../types";
import { dirname } from "../format";

export interface Row {
  node: TreeNode;
  depth: number;
}

class Workspace {
  root = $state<string | null>(null);
  maxDepth = $state(255);
  /** Loaded children per directory path. */
  children = $state<Record<string, TreeNode[]>>({});
  expanded = $state<Record<string, boolean>>({});
  selected = $state<string | null>(null);

  /** Flattened, visible tree rows (the sidebar virtualises over this). */
  rows = $derived.by<Row[]>(() => {
    const out: Row[] = [];
    if (!this.root) return out;
    const walk = (dir: string, depth: number) => {
      for (const node of this.children[dir] ?? []) {
        out.push({ node, depth });
        if (node.kind === "dir" && this.expanded[node.path]) walk(node.path, depth + 1);
      }
    };
    walk(this.root, 0);
    return out;
  });

  async open(path: string) {
    const info = await openWorkspace(path);
    this.root = info.root;
    this.maxDepth = info.maxDepth;
    this.children = { [info.root]: info.nodes };
    this.expanded = {};
    this.selected = info.root;
  }

  async close() {
    await closeWorkspace();
    this.root = null;
    this.children = {};
    this.expanded = {};
    this.selected = null;
  }

  async load(dir: string) {
    this.children[dir] = await listDir(dir);
  }

  async toggle(dir: string) {
    if (this.expanded[dir]) {
      this.expanded[dir] = false;
      return;
    }
    if (!this.children[dir]) await this.load(dir);
    this.expanded[dir] = true;
  }

  async reveal(path: string) {
    if (!this.root) return;
    const rel = path.slice(this.root.length).split(/[\\/]/).filter(Boolean);
    rel.pop();
    let cur = this.root;
    for (const part of rel) {
      cur = cur.replace(/[\\/]$/, "") + (cur.includes("\\") ? "\\" : "/") + part;
      if (!this.children[cur]) await this.load(cur);
      this.expanded[cur] = true;
    }
  }

  /** Re-lists the directories that are loaded and were touched on disk. */
  async refresh(dirs: string[]) {
    await Promise.all(
      dirs.filter((d) => d in this.children).map((d) => this.load(d).catch(() => delete this.children[d])),
    );
  }

  /** Directory a "new file/folder" action should target for the current selection. */
  targetDir(path: string | null = this.selected): string | null {
    if (!this.root) return null;
    if (!path) return this.root;
    for (const list of Object.values(this.children)) {
      const n = list.find((x) => x.path === path);
      if (n) return n.kind === "dir" ? n.path : dirname(n.path);
    }
    return path === this.root ? this.root : this.root;
  }

  async newFile(dir: string, name: string, content?: string) {
    const fname = /\.(hurl|hur)$/.test(name) ? name : `${name}.hurl`;
    const path = await createFile(dir, fname, content);
    await this.load(dir);
    this.expanded[dir] = true;
    return path;
  }
  async newDir(parent: string, name: string) {
    const path = await createDir(parent, name);
    await this.load(parent);
    this.expanded[parent] = true;
    this.expanded[path] = true;
    this.children[path] = [];
    return path;
  }
  async rename(path: string, newName: string) {
    const to = await renamePath(path, newName);
    await this.load(dirname(path));
    delete this.children[path];
    return to;
  }
  async remove(path: string) {
    await deletePath(path);
    await this.load(dirname(path));
  }
  async duplicate(path: string) {
    const to = await duplicatePath(path);
    await this.load(dirname(path));
    return to;
  }
}

export const workspace = new Workspace();
