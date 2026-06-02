import { Database, FileJson, HardDrive, RefreshCw } from "lucide-react";
import type { Meta } from "../api";
import { formatBytes, formatNumber, shortPath } from "../utils";

type TopBarProps = {
  meta: Meta | undefined;
  isLoading: boolean;
  onRefresh: () => void;
};

export default function TopBar({ meta, isLoading, onRefresh }: TopBarProps) {
  return (
    <header className="top-bar">
      <div className="brand-block">
        <div className="brand-mark">
          <FileJson size={24} />
        </div>
        <div>
          <p className="eyebrow">JSONL Lab</p>
          <h1>{meta ? shortPath(meta.path) : "Viewer"}</h1>
          <p className="path-line" title={meta?.path}>
            {meta?.path ?? "Waiting for dataset metadata"}
          </p>
        </div>
      </div>

      <div className="meta-strip">
        <div className="meta-pill">
          <Database size={17} />
          <span>{meta ? formatNumber(meta.num_lines) : "-"} rows</span>
        </div>
        <div className="meta-pill">
          <HardDrive size={17} />
          <span>{meta ? formatBytes(meta.file_size) : "-"}</span>
        </div>
        <button className="icon-button" disabled={isLoading} onClick={onRefresh} type="button">
          <RefreshCw className={isLoading ? "spin" : ""} size={18} />
          <span>Refresh</span>
        </button>
      </div>
    </header>
  );
}
