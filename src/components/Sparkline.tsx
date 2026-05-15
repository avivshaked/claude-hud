import { useEffect, useRef } from "react";
import type { Sample } from "../types";

type Props = {
  samples: Sample[];
  field: "five_hour" | "seven_day";
  height?: number;
};

export function Sparkline({ samples, field, height = 48 }: Props) {
  const ref = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    const canvas = ref.current;
    if (!canvas) return;
    const dpr = window.devicePixelRatio || 1;
    const w = canvas.clientWidth;
    const h = height;
    canvas.width = w * dpr;
    canvas.height = h * dpr;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;
    ctx.scale(dpr, dpr);
    ctx.clearRect(0, 0, w, h);

    if (samples.length < 2) return;

    const pad = 2;
    const usable = h - pad * 2;
    const points = samples.map((s, i) => {
      const v = s[field] ?? 0;
      const x = (i / (samples.length - 1)) * w;
      const y = pad + usable - (Math.min(100, Math.max(0, v)) / 100) * usable;
      return [x, y] as const;
    });

    ctx.lineWidth = 1.5;
    ctx.strokeStyle = "#a78bfa";
    ctx.beginPath();
    points.forEach(([x, y], i) => (i === 0 ? ctx.moveTo(x, y) : ctx.lineTo(x, y)));
    ctx.stroke();

    const grad = ctx.createLinearGradient(0, 0, 0, h);
    grad.addColorStop(0, "rgba(167,139,250,0.35)");
    grad.addColorStop(1, "rgba(167,139,250,0)");
    ctx.fillStyle = grad;
    ctx.lineTo(w, h);
    ctx.lineTo(0, h);
    ctx.closePath();
    ctx.fill();
  }, [samples, field, height]);

  return <canvas ref={ref} className="sparkline" style={{ height }} />;
}
