import { useUsage } from "./hooks/useUsage";
import { Minimal } from "./modes/Minimal";
import { Full } from "./modes/Full";

export default function App() {
  const { snapshot, history, status, mode } = useUsage();

  if (mode === "full") {
    return <Full snapshot={snapshot} history={history} status={status} />;
  }
  return <Minimal snapshot={snapshot} status={status} />;
}
