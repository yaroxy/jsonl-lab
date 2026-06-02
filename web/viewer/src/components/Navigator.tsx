import { ChevronLeft, ChevronRight, Loader2, Search, Shuffle } from "lucide-react";
import { formatNumber } from "../utils";

type NavigatorProps = {
  currentIdx: number;
  inputIdx: string;
  maxIdx: number;
  disabled: boolean;
  isFetching: boolean;
  onInputChange: (value: string) => void;
  onGo: () => void;
  onPrev: () => void;
  onNext: () => void;
  onRandom: () => void;
};

export default function Navigator({
  currentIdx,
  inputIdx,
  maxIdx,
  disabled,
  isFetching,
  onInputChange,
  onGo,
  onPrev,
  onNext,
  onRandom,
}: NavigatorProps) {
  return (
    <section className="nav-card">
      <div className="nav-copy">
        <p className="eyebrow">Current index</p>
        <h2>{formatNumber(currentIdx)}</h2>
        <p>Valid range: 0 to {formatNumber(maxIdx)}</p>
      </div>

      <form
        className="nav-controls"
        onSubmit={(event) => {
          event.preventDefault();
          onGo();
        }}
      >
        <label className="idx-input-wrap">
          <span>Jump to</span>
          <input
            disabled={disabled}
            inputMode="numeric"
            min={0}
            max={maxIdx}
            type="number"
            value={inputIdx}
            onChange={(event) => onInputChange(event.target.value)}
          />
        </label>

        <button className="primary-button" disabled={disabled} type="submit">
          <Search size={18} />
          Go
        </button>
        <button className="secondary-button" disabled={disabled || currentIdx <= 0} onClick={onPrev} type="button">
          <ChevronLeft size={18} />
          Prev
        </button>
        <button className="secondary-button" disabled={disabled || currentIdx >= maxIdx} onClick={onNext} type="button">
          Next
          <ChevronRight size={18} />
        </button>
        <button className="secondary-button" disabled={disabled} onClick={onRandom} type="button">
          <Shuffle size={18} />
          Random
        </button>

        {isFetching ? (
          <span className="fetch-badge">
            <Loader2 className="spin" size={16} />
            Loading
          </span>
        ) : null}
      </form>
    </section>
  );
}
