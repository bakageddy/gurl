import { mount } from "svelte";
import "./app.css";
import App from "./App.svelte";
import { applyPlatformClass } from "./lib/platform";

applyPlatformClass();

// The app is a desktop shell: no browser context menu or text-selection drags on chrome.
document.addEventListener("contextmenu", (e) => {
  if (!(e.target instanceof HTMLElement && e.target.closest(".cm-editor, input, textarea"))) e.preventDefault();
});

mount(App, { target: document.getElementById("app")! });
