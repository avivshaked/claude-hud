import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { Bootstrap, FetchStatus, HudMode, Sample, UsageSnapshot } from "../types";

type State = {
  snapshot: UsageSnapshot | null;
  history: Sample[];
  status: FetchStatus;
  mode: HudMode;
};

const initial: State = {
  snapshot: null,
  history: [],
  status: { kind: "stale", at: new Date().toISOString(), reason: "starting" },
  mode: "minimal",
};

export function useUsage() {
  const [state, setState] = useState<State>(initial);

  useEffect(() => {
    let cancelled = false;

    (async () => {
      try {
        const boot = await invoke<Bootstrap>("bootstrap");
        if (cancelled) return;
        setState({
          snapshot: boot.snapshot,
          history: boot.history,
          status: boot.status,
          mode: boot.settings.mode,
        });
      } catch (e) {
        if (cancelled) return;
        setState((s) => ({
          ...s,
          status: { kind: "error", reason: String(e) },
        }));
      }
    })();

    const unlistenUsage = listen<{
      snapshot: UsageSnapshot;
      status: FetchStatus;
      sample: Sample;
    }>("usage://updated", (e) => {
      setState((s) => ({
        ...s,
        snapshot: e.payload.snapshot,
        status: e.payload.status,
        history: [...s.history.slice(-1439), e.payload.sample],
      }));
    });

    const unlistenStatus = listen<FetchStatus>("usage://status", (e) => {
      setState((s) => ({ ...s, status: e.payload }));
    });

    const unlistenMode = listen<HudMode>("settings://mode", (e) => {
      setState((s) => ({ ...s, mode: e.payload }));
    });

    return () => {
      cancelled = true;
      unlistenUsage.then((f) => f());
      unlistenStatus.then((f) => f());
      unlistenMode.then((f) => f());
    };
  }, []);

  return state;
}
