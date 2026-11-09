import { useDocumentStore } from "../stores/document";
import type { MathRegion as MathRegionType } from "../types/document";
import { splitResultUnit, formatUnitDisplay } from "../lib/unitText";
import type { MathCompletion } from "../lib/mathCompletions";
import { MathField } from "./MathField";

type Props = {
  region: MathRegionType;
  onChange: (patch: Partial<MathRegionType>) => void;
  onEnter: () => void;
  completions: MathCompletion[];
  constantNames: string[];
};

export function MathRegion({
  region,
  onChange,
  onEnter,
  completions,
  constantNames
}: Props) {
  const result = useDocumentStore((state) => state.mathResults[region.id]);
  const hasTrailingEquals = region.source.trimEnd().endsWith("=");
  const formattedResult = result?.display
    ? splitResultUnit(result.display)
    : null;

  const handleChange = (latex: string, ascii: string) => {
    onChange({ source: latex, ascii });
  };

  return (
    <div className="math-region">
      <div
        className={`math-input-box ${hasTrailingEquals ? "with-result" : ""}`}
      >
        <MathField
          value={region.source}
          onChange={handleChange}
          onEnter={onEnter}
          completions={completions}
          constantNames={constantNames}
        />
        {hasTrailingEquals && formattedResult?.value && (
          <span
            className="region-result region-result-inline"
            title={`= ${result.display}`}
          >
            {formattedResult.value}
            {formattedResult.unit && (
              <span className="region-result-unit">
                {formatUnitDisplay(formattedResult.unit)}
              </span>
            )}
          </span>
        )}
      </div>

      {result?.error ? (
        <span className="region-error" title={result.error}>
          {result.error}
        </span>
      ) : !hasTrailingEquals && formattedResult?.value ? (
        <span className="region-result" title={`= ${result.display}`}>
          = {formattedResult.value}
          {formattedResult.unit && (
            <span className="region-result-unit">{formatUnitDisplay(formattedResult.unit)}</span>
          )}
        </span>
      ) : null}
    </div>
  );
}
