import { UsageBar } from "../components/UsageBar";
import type { FetchStatus, UsageSnapshot } from "../types";

type Props = {
  snapshot: UsageSnapshot | null;
  status: FetchStatus;
};

function StatusFooter({ status }: { status: FetchStatus }) {
  if (status.kind === "auth_required") {
    return (
      <div className="hud__footer">
        <span><span className="status-dot status-dot--err" />sign in</span>
        <span>run <code>claude</code></span>
      </div>
    );
  }
  if (status.kind === "error") {
    return (
      <div className="hud__footer" title={status.reason}>
        <span><span className="status-dot status-dot--err" />offline</span>
        <span style={{ maxWidth: "60%", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
          {status.reason}
        </span>
      </div>
    );
  }
  if (status.kind === "stale") {
    return (
      <div className="hud__footer">
        <span><span className="status-dot status-dot--stale" />stale</span>
      </div>
    );
  }
  return (
    <div className="hud__footer">
      <span><span className="status-dot status-dot--ok" />live</span>
    </div>
  );
}

export function Minimal({ snapshot, status }: Props) {
  return (
    <div className="hud">
      <UsageBar
        label="5h"
        utilization={snapshot?.five_hour?.utilization ?? null}
        resetsAt={snapshot?.five_hour?.resets_at ?? null}
      />
      <UsageBar
        label="Week"
        utilization={snapshot?.seven_day?.utilization ?? null}
        resetsAt={snapshot?.seven_day?.resets_at ?? null}
      />
      {snapshot?.seven_day_opus && (
        <UsageBar
          label="Opus / wk"
          utilization={snapshot.seven_day_opus.utilization}
          resetsAt={snapshot.seven_day_opus.resets_at}
        />
      )}
      <StatusFooter status={status} />
    </div>
  );
}
