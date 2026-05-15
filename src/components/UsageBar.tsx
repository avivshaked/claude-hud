import { useEffect, useState } from "react";

type Props = {
  label: string;
  utilization: number | null;
  resetsAt: string | null;
};

function color(u: number): "green" | "amber" | "red" {
  if (u >= 90) return "red";
  if (u >= 75) return "amber";
  return "green";
}

function formatCountdown(resetsAt: string | null, now: number): { text: string; stale: boolean } {
  if (!resetsAt) return { text: "", stale: false };
  const target = Date.parse(resetsAt);
  if (Number.isNaN(target)) return { text: "", stale: false };
  const diff = target - now;
  if (diff <= 0) return { text: "stale — waiting for next poll", stale: true };
  const total = Math.floor(diff / 1000);
  const h = Math.floor(total / 3600);
  const m = Math.floor((total % 3600) / 60);
  const s = total % 60;
  const text = h > 0
    ? `resets in ${h}h ${m.toString().padStart(2, "0")}m`
    : `resets in ${m}:${s.toString().padStart(2, "0")}`;
  return { text, stale: false };
}

export function UsageBar({ label, utilization, resetsAt }: Props) {
  const [now, setNow] = useState(() => Date.now());
  useEffect(() => {
    const id = window.setInterval(() => setNow(Date.now()), 1000);
    return () => window.clearInterval(id);
  }, []);

  const u = utilization ?? 0;
  const c = color(u);
  const { text: countdownText, stale } = formatCountdown(resetsAt, now);

  return (
    <div className="hud__row">
      <div className="hud__label">
        <span className="name">{label}</span>
        <span className="pct">{utilization === null ? "—" : `${u.toFixed(0)}%`}</span>
      </div>
      <div className="bar">
        <div
          className={`bar__fill bar__fill--${c}`}
          style={{
            width: `${Math.min(100, Math.max(0, u))}%`,
            opacity: stale ? 0.4 : 1,
          }}
        />
      </div>
      {countdownText && (
        <div className="countdown" style={stale ? { color: "var(--amber)" } : undefined}>
          {countdownText}
        </div>
      )}
    </div>
  );
}
