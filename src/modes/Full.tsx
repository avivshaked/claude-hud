import { UsageBar } from "../components/UsageBar";
import { Sparkline } from "../components/Sparkline";
import type { FetchStatus, Sample, UsageSnapshot } from "../types";

type Props = {
  snapshot: UsageSnapshot | null;
  history: Sample[];
  status: FetchStatus;
};

export function Full({ snapshot, history, status }: Props) {
  return (
    <div className="hud full">
      <UsageBar
        label="5h"
        utilization={snapshot?.five_hour?.utilization ?? null}
        resetsAt={snapshot?.five_hour?.resets_at ?? null}
      />
      <Sparkline samples={history} field="five_hour" height={40} />
      <UsageBar
        label="Week"
        utilization={snapshot?.seven_day?.utilization ?? null}
        resetsAt={snapshot?.seven_day?.resets_at ?? null}
      />
      <Sparkline samples={history} field="seven_day" height={40} />
      {snapshot?.seven_day_opus && (
        <UsageBar
          label="Opus / wk"
          utilization={snapshot.seven_day_opus.utilization}
          resetsAt={snapshot.seven_day_opus.resets_at}
        />
      )}
      {snapshot?.seven_day_sonnet && (
        <UsageBar
          label="Sonnet / wk"
          utilization={snapshot.seven_day_sonnet.utilization}
          resetsAt={snapshot.seven_day_sonnet.resets_at}
        />
      )}
      <div className="hud__footer">
        <span>
          {status.kind === "ok" && <><span className="status-dot status-dot--ok" />live</>}
          {status.kind === "stale" && <><span className="status-dot status-dot--stale" />stale</>}
          {status.kind === "error" && <><span className="status-dot status-dot--err" />offline</>}
          {status.kind === "auth_required" && <><span className="status-dot status-dot--err" />sign in</>}
        </span>
        <span>{history.length} samples</span>
      </div>
    </div>
  );
}
