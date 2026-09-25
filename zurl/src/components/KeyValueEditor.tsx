import { Trash2, GripVertical } from "lucide-solid";
import type { KeyValue } from "../types";

interface Props {
  items: KeyValue[];
  onChange: (items: KeyValue[]) => void;
}

export function KeyValueEditor(props: Props) {
  function update(id: string, patch: Partial<KeyValue>) {
    props.onChange(
      props.items.map((item) => (item.id === id ? { ...item, ...patch } : item))
    );
  }

  function add() {
    props.onChange([
      ...props.items,
      {
        id: Math.random().toString(36).slice(2),
        name: "",
        value: "",
        enabled: true,
      },
    ]);
  }

  function remove(id: string) {
    props.onChange(props.items.filter((item) => item.id !== id));
  }

  return (
    <div class="flex h-full flex-col">
      <div class="flex-1 overflow-auto rounded-lg border border-border">
        <table class="w-full text-left text-sm">
          <thead class="sticky top-0 bg-surface-2 text-xs uppercase text-text-2">
            <tr>
              <th class="w-10 px-3 py-2 text-center"></th>
              <th class="px-3 py-2 font-medium">Key</th>
              <th class="px-3 py-2 font-medium">Value</th>
              <th class="w-12 px-3 py-2 text-center"></th>
            </tr>
          </thead>
          <tbody class="divide-y divide-border">
            {props.items.map((item) => (
              <tr class="group transition hover:bg-white/[0.02]">
                <td class="px-3 py-1.5 text-center">
                  <input
                    type="checkbox"
                    checked={item.enabled}
                    onChange={(e) =>
                      update(item.id, { enabled: e.currentTarget.checked })
                    }
                    class="h-4 w-4 rounded border-border bg-surface-2 text-accent focus:ring-accent/40"
                  />
                </td>
                <td class="px-1 py-1">
                  <input
                    type="text"
                    value={item.name}
                    onInput={(e) => update(item.id, { name: e.currentTarget.value })}
                    placeholder="name"
                    class="w-full rounded bg-transparent px-2 py-1 text-text outline-none transition placeholder:text-text-2/50 focus:bg-surface-2 focus:ring-1 focus:ring-accent/30"
                  />
                </td>
                <td class="px-1 py-1">
                  <input
                    type="text"
                    value={item.value}
                    onInput={(e) => update(item.id, { value: e.currentTarget.value })}
                    placeholder="value"
                    class="w-full rounded bg-transparent px-2 py-1 text-text outline-none transition placeholder:text-text-2/50 focus:bg-surface-2 focus:ring-1 focus:ring-accent/30"
                  />
                </td>
                <td class="px-3 py-1 text-center">
                  <button
                    onClick={() => remove(item.id)}
                    class="rounded p-1 text-text-2 opacity-0 transition hover:bg-danger/20 hover:text-danger group-hover:opacity-100"
                  >
                    <Trash2 size={14} />
                  </button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
      <button
        onClick={add}
        class="mt-2 self-start rounded-lg border border-border bg-surface-2 px-3 py-1.5 text-xs font-medium text-text-2 hover:bg-white/5 transition"
      >
        + Add row
      </button>
    </div>
  );
}
