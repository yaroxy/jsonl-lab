import { AlertTriangle, List } from "lucide-react";
import type { PreviewRow } from "../api";

type RangeListProps = {
  rows: PreviewRow[];
  selectedIdx: number;
  isLoading: boolean;
  error: unknown;
  onSelect: (idx: number) => void;
};

export default function RangeList({ rows, selectedIdx, isLoading, error, onSelect }: RangeListProps) {
  return (
    <aside className="range-panel">
      <div className="panel-heading">
        <div>
          <p className="eyebrow">Nearby rows</p>
          <h2>Range</h2>
        </div>
        <List size={20} />
      </div>

      {error ? (
        <div className="inline-error">
          <AlertTriangle size={17} />
          <span>{error instanceof Error ? error.message : String(error)}</span>
        </div>
      ) : null}

      <div className="range-list">
        {isLoading && rows.length === 0
          ? Array.from({ length: 10 }).map((_, index) => <div className="row-skeleton" key={index} />)
          : null}

        {!isLoading && rows.length === 0 && !error ? (
          <div className="empty-state">No rows available.</div>
        ) : null}

        {rows.map((row) => (
          <button
            className={row.idx === selectedIdx ? "range-row selected" : "range-row"}
            key={row.idx}
            onClick={() => onSelect(row.idx)}
            type="button"
          >
            <span className="range-idx">#{row.idx}</span>
            <span className="range-preview">{row.preview}</span>
          </button>
        ))}
      </div>
    </aside>
  );
}
