export type WindowUsage = {
  utilization: number;
  resets_at: string | null;
} | null;

export type UsageSnapshot = {
  five_hour: WindowUsage;
  seven_day: WindowUsage;
  seven_day_opus: WindowUsage;
  seven_day_sonnet: WindowUsage;
  fetched_at: string;
};

export type FetchStatus =
  | { kind: "ok"; at: string }
  | { kind: "stale"; at: string; reason: string }
  | { kind: "error"; reason: string }
  | { kind: "auth_required" };

export type HudMode = "minimal" | "full";

export type Sample = {
  t: string;
  five_hour: number | null;
  seven_day: number | null;
};

export type Settings = {
  mode: HudMode;
  poll_interval_secs: number;
  hotkey: string;
};

export type Bootstrap = {
  snapshot: UsageSnapshot | null;
  history: Sample[];
  status: FetchStatus;
  settings: Settings;
};
