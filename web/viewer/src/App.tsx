import { useQuery } from "@tanstack/react-query";
import { AlertTriangle } from "lucide-react";
import { startTransition, useEffect, useState } from "react";
import { getItem, getMeta, getRange } from "./api";
import Navigator from "./components/Navigator";
import RangeList from "./components/RangeList";
import RecordViewer from "./components/RecordViewer";
import TopBar from "./components/TopBar";

const RANGE_LIMIT = 24;

export default function App() {
  const [currentIdx, setCurrentIdx] = useState(0);
  const [inputIdx, setInputIdx] = useState("0");

  const metaQuery = useQuery({
    queryKey: ["meta"],
    queryFn: getMeta,
  });

  const meta = metaQuery.data;
  const hasRows = Boolean(meta && meta.num_lines > 0);
  const maxIdx = meta ? Math.max(0, meta.num_lines - 1) : 0;
  const rangeStart = hasRows
    ? Math.max(0, Math.min(currentIdx - Math.floor(RANGE_LIMIT / 2), maxIdx))
    : 0;

  const itemQuery = useQuery({
    queryKey: ["item", currentIdx],
    queryFn: () => getItem(currentIdx),
    enabled: hasRows,
  });

  const rangeQuery = useQuery({
    queryKey: ["range", rangeStart, RANGE_LIMIT],
    queryFn: () => getRange(rangeStart, RANGE_LIMIT),
    enabled: hasRows,
  });

  useEffect(() => {
    setInputIdx(String(currentIdx));
  }, [currentIdx]);

  useEffect(() => {
    if (meta && currentIdx > maxIdx) {
      setCurrentIdx(maxIdx);
    }
  }, [currentIdx, maxIdx, meta]);

  function setSelectedIdx(nextIdx: number) {
    const clamped = Math.max(0, Math.min(nextIdx, maxIdx));

    startTransition(() => {
      setCurrentIdx(clamped);
    });
  }

  function goToInputIdx() {
    const parsed = Number(inputIdx);

    if (!Number.isFinite(parsed)) {
      return;
    }

    setSelectedIdx(Math.trunc(parsed));
  }

  function goToRandomIdx() {
    if (!meta || meta.num_lines === 0) {
      return;
    }

    setSelectedIdx(Math.floor(Math.random() * meta.num_lines));
  }

  const appError = metaQuery.error;

  return (
    <div className="app-shell">
      <div className="ambient ambient-one" />
      <div className="ambient ambient-two" />

      <main className="app-frame">
        <TopBar
          meta={meta}
          isLoading={metaQuery.isLoading}
          onRefresh={() => void metaQuery.refetch()}
        />

        {appError ? (
          <section className="fatal-card">
            <AlertTriangle size={22} />
            <div>
              <h2>Unable to reach JSONL server</h2>
              <p>{appError instanceof Error ? appError.message : String(appError)}</p>
              <p className="hint-text">
                Start the backend with <code>jsonl-lab serve data.jsonl --port 7860</code>.
              </p>
            </div>
          </section>
        ) : (
          <>
            <Navigator
              currentIdx={currentIdx}
              inputIdx={inputIdx}
              maxIdx={maxIdx}
              disabled={!hasRows}
              isFetching={itemQuery.isFetching || rangeQuery.isFetching}
              onInputChange={setInputIdx}
              onGo={goToInputIdx}
              onPrev={() => setSelectedIdx(currentIdx - 1)}
              onNext={() => setSelectedIdx(currentIdx + 1)}
              onRandom={goToRandomIdx}
            />

            <section className="viewer-grid">
              <RangeList
                rows={rangeQuery.data?.rows ?? []}
                selectedIdx={currentIdx}
                isLoading={rangeQuery.isLoading}
                error={rangeQuery.error}
                onSelect={setSelectedIdx}
              />

              <RecordViewer
                idx={currentIdx}
                value={itemQuery.data}
                isLoading={itemQuery.isLoading || itemQuery.isFetching}
                error={itemQuery.error}
                onRefresh={() => void itemQuery.refetch()}
              />
            </section>
          </>
        )}
      </main>
    </div>
  );
}
