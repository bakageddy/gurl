import { createEffect, createSignal } from "solid-js";
import type { HurlRunOptions } from "../types";

interface Props {
  options: HurlRunOptions;
  onChange: (options: HurlRunOptions) => void;
}

function Field(props: {
  label: string;
  children: any;
}) {
  return (
    <label class="flex flex-col gap-1">
      <span class="text-xs font-medium text-text-2">{props.label}</span>
      {props.children}
    </label>
  );
}

function Checkbox(props: {
  label: string;
  checked: boolean;
  onChange: (checked: boolean) => void;
}) {
  return (
    <label class="inline-flex cursor-pointer items-center gap-2 rounded-lg border border-border bg-surface-2 px-3 py-2 hover:bg-white/5 transition">
      <input
        type="checkbox"
        checked={props.checked}
        onChange={(e) => props.onChange(e.currentTarget.checked)}
        class="h-4 w-4 rounded border-border bg-surface text-accent focus:ring-accent/40"
      />
      <span class="text-sm text-text">{props.label}</span>
    </label>
  );
}

export function OptionsPanel(props: Props) {
  const [vars, setVars] = createSignal<string>("");

  createEffect(() => {
    const lines = Object.entries(props.options.variables || {})
      .map(([k, v]) => `${k}=${v}`)
      .join("\n");
    setVars(lines);
  });

  function applyVars(text: string) {
    const variables: Record<string, string> = {};
    for (const line of text.split("\n")) {
      const idx = line.indexOf("=");
      if (idx > 0) {
        variables[line.slice(0, idx).trim()] = line.slice(idx + 1);
      }
    }
    props.onChange({ ...props.options, variables });
  }

  const patch = (partial: Partial<HurlRunOptions>) =>
    props.onChange({ ...props.options, ...partial });

  return (
    <div class="grid h-full grid-cols-1 gap-6 overflow-auto md:grid-cols-2">
      <div class="space-y-4">
        <div class="rounded-lg border border-border bg-surface p-4">
          <h3 class="mb-3 text-sm font-semibold text-text">Variables</h3>
          <textarea
            value={vars()}
            onInput={(e) => applyVars(e.currentTarget.value)}
            rows={8}
            placeholder={`base_url=https://api.example.com\nuser_id=42`}
            class="w-full rounded-lg border border-border bg-bg p-3 font-mono text-sm text-text outline-none transition placeholder:text-text-2/50 focus:border-accent/50 focus:ring-1 focus:ring-accent/30"
          />
        </div>

        <div class="rounded-lg border border-border bg-surface p-4">
          <h3 class="mb-3 text-sm font-semibold text-text">Network & TLS</h3>
          <div class="grid grid-cols-2 gap-3">
            <Checkbox
              label="Insecure TLS"
              checked={props.options.insecure}
              onChange={(v) => patch({ insecure: v })}
            />
            <Checkbox
              label="Follow redirects"
              checked={props.options.followLocation}
              onChange={(v) => patch({ followLocation: v })}
            />
            <Checkbox
              label="Trust redirect credentials"
              checked={props.options.followLocationTrusted}
              onChange={(v) => patch({ followLocationTrusted: v })}
            />
            <Checkbox
              label="Accept compressed"
              checked={props.options.compressed}
              onChange={(v) => patch({ compressed: v })}
            />
          </div>
          <div class="mt-3 grid grid-cols-2 gap-3">
            <Field label="Max redirects (-1 = infinite)">
              <input
                type="number"
                value={props.options.maxRedirect}
                onInput={(e) => patch({ maxRedirect: Number(e.currentTarget.value) })}
                class="rounded-lg border border-border bg-bg px-3 py-2 text-sm text-text outline-none focus:border-accent/50 focus:ring-1 focus:ring-accent/30"
              />
            </Field>
            <Field label="IP resolve">
              <select
                value={props.options.ipResolve}
                onChange={(e) => patch({ ipResolve: e.currentTarget.value })}
                class="rounded-lg border border-border bg-bg px-3 py-2 text-sm text-text outline-none focus:border-accent/50 focus:ring-1 focus:ring-accent/30"
              >
                <option value="default">Default</option>
                <option value="v4">IPv4</option>
                <option value="v6">IPv6</option>
              </select>
            </Field>
            <Field label="HTTP version">
              <select
                value={props.options.httpVersion}
                onChange={(e) => patch({ httpVersion: e.currentTarget.value })}
                class="rounded-lg border border-border bg-bg px-3 py-2 text-sm text-text outline-none focus:border-accent/50 focus:ring-1 focus:ring-accent/30"
              >
                <option value="default">Default</option>
                <option value="1.0">HTTP/1.0</option>
                <option value="1.1">HTTP/1.1</option>
                <option value="2">HTTP/2</option>
                <option value="3">HTTP/3</option>
              </select>
            </Field>
            <Field label="Proxy">
              <input
                type="text"
                value={props.options.proxy}
                onInput={(e) => patch({ proxy: e.currentTarget.value })}
                placeholder="http://proxy:8080"
                class="rounded-lg border border-border bg-bg px-3 py-2 text-sm text-text outline-none placeholder:text-text-2/50 focus:border-accent/50 focus:ring-1 focus:ring-accent/30"
              />
            </Field>
            <Field label="No proxy">
              <input
                type="text"
                value={props.options.noProxy}
                onInput={(e) => patch({ noProxy: e.currentTarget.value })}
                placeholder="localhost,.local"
                class="rounded-lg border border-border bg-bg px-3 py-2 text-sm text-text outline-none placeholder:text-text-2/50 focus:border-accent/50 focus:ring-1 focus:ring-accent/30"
              />
            </Field>
          </div>
        </div>
      </div>

      <div class="space-y-4">
        <div class="rounded-lg border border-border bg-surface p-4">
          <h3 class="mb-3 text-sm font-semibold text-text">Timing & Retries</h3>
          <div class="grid grid-cols-2 gap-3">
            <Field label="Timeout (ms)">
              <input
                type="number"
                value={props.options.timeoutMs}
                onInput={(e) => patch({ timeoutMs: Number(e.currentTarget.value) })}
                class="rounded-lg border border-border bg-bg px-3 py-2 text-sm text-text outline-none focus:border-accent/50 focus:ring-1 focus:ring-accent/30"
              />
            </Field>
            <Field label="Connect timeout (ms)">
              <input
                type="number"
                value={props.options.connectTimeoutMs}
                onInput={(e) => patch({ connectTimeoutMs: Number(e.currentTarget.value) })}
                class="rounded-lg border border-border bg-bg px-3 py-2 text-sm text-text outline-none focus:border-accent/50 focus:ring-1 focus:ring-accent/30"
              />
            </Field>
            <Field label="Retry count">
              <input
                type="number"
                value={props.options.retry}
                onInput={(e) => patch({ retry: Number(e.currentTarget.value) })}
                class="rounded-lg border border-border bg-bg px-3 py-2 text-sm text-text outline-none focus:border-accent/50 focus:ring-1 focus:ring-accent/30"
              />
            </Field>
            <Field label="Retry interval (ms)">
              <input
                type="number"
                value={props.options.retryIntervalMs}
                onInput={(e) => patch({ retryIntervalMs: Number(e.currentTarget.value) })}
                class="rounded-lg border border-border bg-bg px-3 py-2 text-sm text-text outline-none focus:border-accent/50 focus:ring-1 focus:ring-accent/30"
              />
            </Field>
            <Field label="Delay before request (ms)">
              <input
                type="number"
                value={props.options.delayMs}
                onInput={(e) => patch({ delayMs: Number(e.currentTarget.value) })}
                class="rounded-lg border border-border bg-bg px-3 py-2 text-sm text-text outline-none focus:border-accent/50 focus:ring-1 focus:ring-accent/30"
              />
            </Field>
          </div>
        </div>

        <div class="rounded-lg border border-border bg-surface p-4">
          <h3 class="mb-3 text-sm font-semibold text-text">Execution range</h3>
          <div class="grid grid-cols-2 gap-3">
            <Field label="From entry (1-based)">
              <input
                type="number"
                min={1}
                value={props.options.fromEntry ?? ""}
                onInput={(e) => {
                  const v = e.currentTarget.value;
                  patch({ fromEntry: v ? Number(v) : null });
                }}
                class="rounded-lg border border-border bg-bg px-3 py-2 text-sm text-text outline-none focus:border-accent/50 focus:ring-1 focus:ring-accent/30"
              />
            </Field>
            <Field label="To entry (1-based)">
              <input
                type="number"
                min={1}
                value={props.options.toEntry ?? ""}
                onInput={(e) => {
                  const v = e.currentTarget.value;
                  patch({ toEntry: v ? Number(v) : null });
                }}
                class="rounded-lg border border-border bg-bg px-3 py-2 text-sm text-text outline-none focus:border-accent/50 focus:ring-1 focus:ring-accent/30"
              />
            </Field>
          </div>
        </div>

        <div class="rounded-lg border border-border bg-surface p-4">
          <h3 class="mb-3 text-sm font-semibold text-text">Authentication & Identity</h3>
          <div class="grid grid-cols-1 gap-3">
            <Field label="User (basic auth)">
              <input
                type="text"
                value={props.options.user}
                onInput={(e) => patch({ user: e.currentTarget.value })}
                placeholder="username:password"
                class="rounded-lg border border-border bg-bg px-3 py-2 text-sm text-text outline-none placeholder:text-text-2/50 focus:border-accent/50 focus:ring-1 focus:ring-accent/30"
              />
            </Field>
            <Field label="User-Agent">
              <input
                type="text"
                value={props.options.userAgent}
                onInput={(e) => patch({ userAgent: e.currentTarget.value })}
                class="rounded-lg border border-border bg-bg px-3 py-2 text-sm text-text outline-none placeholder:text-text-2/50 focus:border-accent/50 focus:ring-1 focus:ring-accent/30"
              />
            </Field>
          </div>
        </div>

        <div class="rounded-lg border border-border bg-surface p-4">
          <h3 class="mb-3 text-sm font-semibold text-text">TLS certificates</h3>
          <div class="grid grid-cols-1 gap-3">
            <Field label="CA cert file">
              <input
                type="text"
                value={props.options.cacertFile}
                onInput={(e) => patch({ cacertFile: e.currentTarget.value })}
                class="rounded-lg border border-border bg-bg px-3 py-2 text-sm text-text outline-none placeholder:text-text-2/50 focus:border-accent/50 focus:ring-1 focus:ring-accent/30"
              />
            </Field>
            <Field label="Client cert file">
              <input
                type="text"
                value={props.options.clientCertFile}
                onInput={(e) => patch({ clientCertFile: e.currentTarget.value })}
                class="rounded-lg border border-border bg-bg px-3 py-2 text-sm text-text outline-none placeholder:text-text-2/50 focus:border-accent/50 focus:ring-1 focus:ring-accent/30"
              />
            </Field>
            <Field label="Client key file">
              <input
                type="text"
                value={props.options.clientKeyFile}
                onInput={(e) => patch({ clientKeyFile: e.currentTarget.value })}
                class="rounded-lg border border-border bg-bg px-3 py-2 text-sm text-text outline-none placeholder:text-text-2/50 focus:border-accent/50 focus:ring-1 focus:ring-accent/30"
              />
            </Field>
          </div>
        </div>
      </div>

      <div class="space-y-4">
        <div class="rounded-lg border border-border bg-surface p-4">
          <h3 class="mb-3 text-sm font-semibold text-text">Host mappings</h3>
          <div class="space-y-3">
            <Field label="Resolves (one per line)">
              <textarea
                value={props.options.resolves}
                onInput={(e) => patch({ resolves: e.currentTarget.value })}
                rows={3}
                placeholder="example.com:443:127.0.0.1"
                class="w-full rounded-lg border border-border bg-bg p-2 font-mono text-xs text-text outline-none placeholder:text-text-2/50 focus:border-accent/50 focus:ring-1 focus:ring-accent/30"
              />
            </Field>
            <Field label="Connects-to (one per line)">
              <textarea
                value={props.options.connectsTo}
                onInput={(e) => patch({ connectsTo: e.currentTarget.value })}
                rows={3}
                placeholder="host1:port1:host2:port2"
                class="w-full rounded-lg border border-border bg-bg p-2 font-mono text-xs text-text outline-none placeholder:text-text-2/50 focus:border-accent/50 focus:ring-1 focus:ring-accent/30"
              />
            </Field>
          </div>
        </div>

        <div class="rounded-lg border border-border bg-surface p-4">
          <h3 class="mb-3 text-sm font-semibold text-text">Authentication methods</h3>
          <div class="grid grid-cols-2 gap-3">
            <Checkbox
              label="HTTP Digest"
              checked={props.options.digest}
              onChange={(v) => patch({ digest: v })}
            />
            <Checkbox
              label="Negotiate (SPNEGO)"
              checked={props.options.negotiate}
              onChange={(v) => patch({ negotiate: v })}
            />
            <Checkbox
              label="NTLM"
              checked={props.options.ntlm}
              onChange={(v) => patch({ ntlm: v })}
            />
            <Checkbox
              label="Use .netrc"
              checked={props.options.netrc}
              onChange={(v) => patch({ netrc: v })}
            />
            <Checkbox
              label="Netrc optional"
              checked={props.options.netrcOptional}
              onChange={(v) => patch({ netrcOptional: v })}
            />
          </div>
          <div class="mt-3 grid grid-cols-1 gap-3">
            <Field label="Netrc file">
              <input
                type="text"
                value={props.options.netrcFile}
                onInput={(e) => patch({ netrcFile: e.currentTarget.value })}
                class="rounded-lg border border-border bg-bg px-3 py-2 text-sm text-text outline-none placeholder:text-text-2/50 focus:border-accent/50 focus:ring-1 focus:ring-accent/30"
              />
            </Field>
            <Field label="AWS SigV4">
              <input
                type="text"
                value={props.options.awsSigv4}
                onInput={(e) => patch({ awsSigv4: e.currentTarget.value })}
                placeholder="region/service"
                class="rounded-lg border border-border bg-bg px-3 py-2 text-sm text-text outline-none placeholder:text-text-2/50 focus:border-accent/50 focus:ring-1 focus:ring-accent/30"
              />
            </Field>
          </div>
        </div>
      </div>

      <div class="space-y-4">
        <div class="rounded-lg border border-border bg-surface p-4">
          <h3 class="mb-3 text-sm font-semibold text-text">TLS & Connection</h3>
          <div class="grid grid-cols-2 gap-3">
            <Checkbox
              label="SSL no revoke"
              checked={props.options.sslNoRevoke}
              onChange={(v) => patch({ sslNoRevoke: v })}
            />
            <Checkbox
              label="Path as-is"
              checked={props.options.pathAsIs}
              onChange={(v) => patch({ pathAsIs: v })}
            />
            <Checkbox
              label="Allow connection reuse"
              checked={props.options.allowReuse}
              onChange={(v) => patch({ allowReuse: v })}
            />
          </div>
          <div class="mt-3 grid grid-cols-1 gap-3">
            <Field label="Pinned public key">
              <input
                type="text"
                value={props.options.pinnedPubKey}
                onInput={(e) => patch({ pinnedPubKey: e.currentTarget.value })}
                class="rounded-lg border border-border bg-bg px-3 py-2 text-sm text-text outline-none placeholder:text-text-2/50 focus:border-accent/50 focus:ring-1 focus:ring-accent/30"
              />
            </Field>
            <Field label="Unix socket">
              <input
                type="text"
                value={props.options.unixSocket}
                onInput={(e) => patch({ unixSocket: e.currentTarget.value })}
                placeholder="/path/to/socket"
                class="rounded-lg border border-border bg-bg px-3 py-2 text-sm text-text outline-none placeholder:text-text-2/50 focus:border-accent/50 focus:ring-1 focus:ring-accent/30"
              />
            </Field>
          </div>
        </div>

        <div class="rounded-lg border border-border bg-surface p-4">
          <h3 class="mb-3 text-sm font-semibold text-text">Transfer speed & Repetition</h3>
          <div class="grid grid-cols-2 gap-3">
            <Field label="Repeat count">
              <input
                type="number"
                value={props.options.repeat}
                onInput={(e) => patch({ repeat: Number(e.currentTarget.value) })}
                class="rounded-lg border border-border bg-bg px-3 py-2 text-sm text-text outline-none focus:border-accent/50 focus:ring-1 focus:ring-accent/30"
              />
            </Field>
            <Field label="Max send speed (B/s)">
              <input
                type="number"
                value={props.options.maxSendSpeed ?? ""}
                onInput={(e) => {
                  const v = e.currentTarget.value;
                  patch({ maxSendSpeed: v ? Number(v) : null });
                }}
                class="rounded-lg border border-border bg-bg px-3 py-2 text-sm text-text outline-none focus:border-accent/50 focus:ring-1 focus:ring-accent/30"
              />
            </Field>
            <Field label="Max recv speed (B/s)">
              <input
                type="number"
                value={props.options.maxRecvSpeed ?? ""}
                onInput={(e) => {
                  const v = e.currentTarget.value;
                  patch({ maxRecvSpeed: v ? Number(v) : null });
                }}
                class="rounded-lg border border-border bg-bg px-3 py-2 text-sm text-text outline-none focus:border-accent/50 focus:ring-1 focus:ring-accent/30"
              />
            </Field>
          </div>
        </div>

        <div class="rounded-lg border border-border bg-surface p-4">
          <h3 class="mb-3 text-sm font-semibold text-text">Miscellaneous</h3>
          <div class="grid grid-cols-2 gap-3">
            <Checkbox
              label="Fail fast"
              checked={props.options.failFast}
              onChange={(v) => patch({ failFast: v })}
            />
            <Checkbox
              label="Ignore asserts"
              checked={props.options.noAssert}
              onChange={(v) => patch({ noAssert: v })}
            />
            <Checkbox
              label="Use cookie store"
              checked={props.options.useCookieStore}
              onChange={(v) => patch({ useCookieStore: v })}
            />
            <Field label="Verbosity">
              <select
                value={props.options.verbosity}
                onChange={(e) => patch({ verbosity: e.currentTarget.value })}
                class="rounded-lg border border-border bg-bg px-3 py-2 text-sm text-text outline-none focus:border-accent/50 focus:ring-1 focus:ring-accent/30"
              >
                <option value="none">None</option>
                <option value="verbose">Verbose</option>
                <option value="very-verbose">Very verbose</option>
              </select>
            </Field>
          </div>
        </div>
      </div>
    </div>
  );
}
