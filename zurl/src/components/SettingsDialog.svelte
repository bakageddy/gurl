<script lang="ts">
  import { settings } from "../lib/stores/settings.svelte";
  import { ui } from "../lib/stores/ui.svelte";
  import { DEFAULT_RUN_OPTIONS } from "../lib/types";

  const o = $derived(settings.runOptions);
  let vars = $state("");

  $effect(() => {
    if (ui.settingsOpen) {
      vars = Object.entries(settings.runOptions.variables).map(([k, v]) => `${k}=${v}`).join("\n");
    }
  });

  function commitVars() {
    const out: Record<string, string> = {};
    for (const line of vars.split("\n")) {
      const i = line.indexOf("=");
      if (i > 0) out[line.slice(0, i).trim()] = line.slice(i + 1).trim();
    }
    settings.runOptions.variables = out;
    settings.save();
  }
  function reset() {
    settings.runOptions = { ...DEFAULT_RUN_OPTIONS };
    vars = "";
    settings.save();
  }
  const num = (v: string, d: number) => (Number.isFinite(+v) && v !== "" ? +v : d);
</script>

{#if ui.settingsOpen}
  <div class="scrim" role="presentation" onclick={() => (ui.settingsOpen = false)}>
    <div class="dialog" role="dialog" tabindex="-1" onclick={(e) => e.stopPropagation()} onkeydown={() => {}} onchange={() => settings.save()}>
      <header>
        <h3>Run settings</h3>
        <button class="btn sm ghost" onclick={reset}>Reset to defaults</button>
        <button class="btn sm" onclick={() => (ui.settingsOpen = false)}>Done</button>
      </header>
      <div class="scroll">
        <section>
          <h4 class="label">Workspace runs</h4>
          <label class="check"><input type="checkbox" bind:checked={settings.recursive} /> Include sub-folders when running a folder</label>
          <label class="row"><span>Parallel files</span><input class="field" type="number" min="1" max="16" bind:value={settings.jobs} /></label>
        </section>

        <section>
          <h4 class="label">Variable overrides</h4>
          <p class="hint muted">One <code>name=value</code> per line. These win over every <code>zurl.env</code>.</p>
          <textarea class="field mono" rows="3" bind:value={vars} onblur={commitVars} spellcheck="false"></textarea>
        </section>

        <section>
          <h4 class="label">Connection</h4>
          <label class="check"><input type="checkbox" bind:checked={o.follow_location} /> Follow redirects</label>
          <label class="check"><input type="checkbox" bind:checked={o.follow_location_trusted} /> …and forward credentials to other hosts</label>
          <label class="row"><span>Max redirects</span><input class="field" type="number" min="0" value={o.max_redirect} oninput={(e) => (o.max_redirect = num(e.currentTarget.value, 50))} /></label>
          <label class="check"><input type="checkbox" bind:checked={o.insecure} /> Skip TLS certificate verification</label>
          <label class="check"><input type="checkbox" bind:checked={o.compressed} /> Request compressed responses</label>
          <label class="check"><input type="checkbox" bind:checked={o.use_cookie_store} /> Keep cookies between requests</label>
          <label class="row"><span>HTTP version</span>
            <select class="field" bind:value={o.http_version}>
              <option value="">Default</option><option value="1.0">1.0</option><option value="1.1">1.1</option><option value="2">2</option><option value="3">3</option>
            </select></label>
          <label class="row"><span>IP version</span>
            <select class="field" bind:value={o.ip_resolve}>
              <option value="">Default</option><option value="v4">IPv4</option><option value="v6">IPv6</option>
            </select></label>
        </section>

        <section>
          <h4 class="label">Timing</h4>
          <label class="row"><span>Timeout (ms)</span><input class="field" type="number" min="0" value={o.timeout_ms} oninput={(e) => (o.timeout_ms = num(e.currentTarget.value, 300000))} /></label>
          <label class="row"><span>Connect timeout (ms)</span><input class="field" type="number" min="0" value={o.connect_timeout_ms} oninput={(e) => (o.connect_timeout_ms = num(e.currentTarget.value, 300000))} /></label>
          <label class="row"><span>Retries</span><input class="field" type="number" min="0" value={o.retry} oninput={(e) => (o.retry = num(e.currentTarget.value, 0))} /></label>
          <label class="row"><span>Retry interval (ms)</span><input class="field" type="number" min="0" value={o.retry_interval_ms} oninput={(e) => (o.retry_interval_ms = num(e.currentTarget.value, 1000))} /></label>
          <label class="row"><span>Delay between requests (ms)</span><input class="field" type="number" min="0" value={o.delay_ms} oninput={(e) => (o.delay_ms = num(e.currentTarget.value, 0))} /></label>
          <label class="row"><span>Repeat</span><input class="field" type="number" min="1" value={o.repeat} oninput={(e) => (o.repeat = num(e.currentTarget.value, 1))} /></label>
        </section>

        <section>
          <h4 class="label">Identity & network</h4>
          <label class="row"><span>User agent</span><input class="field mono" bind:value={o.user_agent} placeholder="default" /></label>
          <label class="row"><span>User (user:password)</span><input class="field mono" bind:value={o.user} /></label>
          <label class="row"><span>Proxy</span><input class="field mono" bind:value={o.proxy} placeholder="http://host:3128" /></label>
          <label class="row"><span>No proxy</span><input class="field mono" bind:value={o.no_proxy} placeholder="localhost,.internal" /></label>
          <label class="row"><span>CA certificate</span><input class="field mono" bind:value={o.cacert_file} placeholder="/path/ca.pem" /></label>
          <label class="row"><span>Client certificate</span><input class="field mono" bind:value={o.client_cert_file} /></label>
          <label class="row"><span>Client key</span><input class="field mono" bind:value={o.client_key_file} /></label>
        </section>

        <section>
          <h4 class="label">Behaviour</h4>
          <label class="check"><input type="checkbox" bind:checked={o.fail_fast} /> Stop a scenario at the first failing request</label>
          <label class="check"><input type="checkbox" bind:checked={o.no_assert} /> Ignore asserts (just send the requests)</label>
        </section>
      </div>
    </div>
  </div>
{/if}

<style>
  .scrim {
    position: fixed;
    inset: 0;
    z-index: 700;
    display: grid;
    place-items: center;
    background: rgba(1, 4, 9, 0.4);
  }
  .dialog {
    display: flex;
    flex-direction: column;
    width: min(560px, calc(100vw - 32px));
    max-height: calc(100vh - 64px);
    border: 1px solid var(--border);
    border-radius: 12px;
    background: var(--canvas-overlay);
    box-shadow: var(--shadow);
  }
  header {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 14px 18px;
    border-bottom: 1px solid var(--border);
  }
  h3 {
    margin: 0 auto 0 0;
    font-size: 14px;
  }
  .scroll {
    overflow: auto;
    padding: 6px 18px 18px;
  }
  section {
    display: grid;
    gap: 8px;
    margin-top: 14px;
  }
  h4 {
    margin: 0;
  }
  .hint {
    margin: 0;
    font-size: 12px;
  }
  .check {
    display: flex;
    gap: 8px;
    align-items: center;
  }
  .row {
    display: grid;
    grid-template-columns: 200px 1fr;
    gap: 10px;
    align-items: center;
  }
  .row span {
    color: var(--fg-muted);
  }
</style>
