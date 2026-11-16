export type Status = "unknown" | "current" | "outdated" | "error";

export interface Item {
  id: string;
  key: string;
  ecosystem: string;
  name: string;
  version: string;
  source: string;
  kind: string;
  location: string | null;
  updateCommand: string | null;
  removeCommand: string | null;
  note: string | null;
  noteKey: string | null;
  noteArgs: string[];
  description: string | null;
  status: Status;
  latest: string | null;
}

export interface Ecosystem {
  id: string;
  label: string;
  blurb: string;
  items: Item[];
  checkable: boolean;
  headline: string | null;
  error: string | null;
  warnings: string[];
  durationMs: number;
}

export interface Inventory {
  scannedAtMs: number;
  host: string;
  os: string;
  arch: string;
  model: string;
  ecosystems: Ecosystem[];
}

export interface Patch {
  key: string;
  latest: string;
}

export interface Updates {
  checkedAtMs: number;
  checkedEcosystems: string[];
  patches: Patch[];
  warnings: string[];
}

export interface ExportResult {
  folder: string;
  files: string[];
  itemCount: number;
}
