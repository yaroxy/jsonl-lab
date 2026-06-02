import { JsonView, darkStyles } from "react-json-view-lite";
import { AlertTriangle, Braces, RefreshCw } from "lucide-react";

type RecordViewerProps = {
  idx: number;
  value: unknown;
  isLoading: boolean;
  error: unknown;
  onRefresh: () => void;
};

export default function RecordViewer({ idx, value, isLoading, error, onRefresh }: RecordViewerProps) {
  const jsonViewData = toJsonViewData(value);

  return (
    <section className="record-panel">
      <div className="panel-heading record-heading">
        <div>
          <p className="eyebrow">Pretty JSON</p>
          <h2>Record #{idx}</h2>
        </div>
        <button className="icon-button compact" disabled={isLoading} onClick={onRefresh} type="button">
          <RefreshCw className={isLoading ? "spin" : ""} size={17} />
          Reload
        </button>
      </div>

      <div className="record-body">
        {error ? (
          <div className="record-error">
            <AlertTriangle size={22} />
            <div>
              <h3>Unable to load record</h3>
              <p>{error instanceof Error ? error.message : String(error)}</p>
            </div>
          </div>
        ) : null}

        {isLoading && !value && !error ? (
          <div className="json-placeholder">
            <Braces size={28} />
            <p>Loading record...</p>
          </div>
        ) : null}

        {!isLoading && !error && value === undefined ? (
          <div className="json-placeholder">
            <Braces size={28} />
            <p>No record selected.</p>
          </div>
        ) : null}

        {value !== undefined && !error ? (
          <div className="json-card">
            <JsonView data={jsonViewData} shouldExpandNode={shallowExpand} style={darkStyles} />
          </div>
        ) : null}
      </div>
    </section>
  );
}

function toJsonViewData(value: unknown): object | unknown[] {
  if (Array.isArray(value)) {
    return value;
  }

  if (value !== null && typeof value === "object") {
    return value;
  }

  return { value };
}

function shallowExpand(level: number) {
  return level <= 2;
}
