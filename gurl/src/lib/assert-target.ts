// How a run result view hands "add this assert" back to whatever owns the scenario text. Provided
// through Svelte context by DocView (the tab's own run) and ReportView (history / folder runs).

import type { AssertM, CaptureM } from "./types";

export const ASSERT_TARGET = Symbol("assert-target");

export interface AssertItems {
  asserts?: AssertM[];
  captures?: CaptureM[];
}

export interface AssertTarget {
  /** Whether adding is possible right now; `reason` explains when it isn't. */
  enabled: () => boolean;
  reason?: () => string;
  /** `entryIndex` is HURL's 1-based entry number in the file. */
  add: (entryIndex: number, items: AssertItems) => Promise<void>;
}
