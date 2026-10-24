<script lang="ts">
  // Frameless windows lose their native resize borders; draw thin hit areas that start a
  // native resize drag.
  import { getCurrentWindow } from "@tauri-apps/api/window";
  const win = getCurrentWindow();
  type Dir = "North" | "South" | "East" | "West" | "NorthEast" | "NorthWest" | "SouthEast" | "SouthWest";
  const edges: Array<{ dir: Dir; cls: string }> = [
    { dir: "North", cls: "n" },
    { dir: "South", cls: "s" },
    { dir: "East", cls: "e" },
    { dir: "West", cls: "w" },
    { dir: "NorthEast", cls: "ne" },
    { dir: "NorthWest", cls: "nw" },
    { dir: "SouthEast", cls: "se" },
    { dir: "SouthWest", cls: "sw" },
  ];
</script>

{#each edges as e (e.dir)}
  <div class="edge {e.cls}" role="presentation" onmousedown={(ev) => { if (ev.button === 0) win.startResizeDragging(e.dir); }}></div>
{/each}

<style>
  .edge {
    position: fixed;
    z-index: 1000;
  }
  .n, .s { left: 8px; right: 8px; height: 4px; cursor: ns-resize; }
  .e, .w { top: 8px; bottom: 8px; width: 4px; cursor: ew-resize; }
  .n { top: 0; } .s { bottom: 0; } .e { right: 0; } .w { left: 0; }
  .ne, .nw, .se, .sw { width: 8px; height: 8px; }
  .ne { top: 0; right: 0; cursor: nesw-resize; }
  .nw { top: 0; left: 0; cursor: nwse-resize; }
  .se { bottom: 0; right: 0; cursor: nwse-resize; }
  .sw { bottom: 0; left: 0; cursor: nesw-resize; }
</style>
