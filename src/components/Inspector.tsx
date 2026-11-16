import { useState } from "react";
import { kindLabel, statusLabel, type Row } from "../rows";

interface Props {
  row: Row;
  onClose: () => void;
}

export default function Inspector({ row, onClose }: Props) {
  const [copyState, setCopyState] = useState<"idle" | "ok" | "manual">("idle");
  const command = row.updateCommand;

  const copyCommand = async (text: string) => {
    try {
      await navigator.clipboard.writeText(text);
      setCopyState("ok");
    } catch {
      setCopyState("manual");
    }
    window.setTimeout(() => setCopyState("idle"), 2200);
  };

  return (
    <aside className="inspector" aria-label="項目細節">
      <div className="inspector-head">
        <div className="inspector-bar">
          <span className="inspector-kind">{kindLabel(row.kind)}</span>
          <button type="button" className="inspector-close" onClick={onClose}>
            關閉
          </button>
        </div>
        <h2 className="inspector-name">{row.name}</h2>
        <p className="inspector-version">{row.version}</p>
        <p className={`inspector-status is-${row.statusResolved}`}>
          {statusLabel(row.statusResolved, row.latestResolved)}
        </p>
      </div>

      <dl className="fields">
        <div className="field">
          <dt>來源</dt>
          <dd>{row.source}</dd>
        </div>

        {row.location && (
          <div className="field">
            <dt>位置</dt>
            <dd className="mono">{row.location}</dd>
          </div>
        )}

        {command && (
          <div className="field">
            <dt>更新指令</dt>
            <dd>
              <div className="cmd">
                <code>{command}</code>
                <button type="button" className="btn btn--tiny" onClick={() => void copyCommand(command)}>
                  {copyState === "ok" ? "已複製" : copyState === "manual" ? "請手動複製" : "複製"}
                </button>
              </div>
            </dd>
          </div>
        )}

        {row.note && (
          <div className="field">
            <dt>備註</dt>
            <dd>{row.note}</dd>
          </div>
        )}
      </dl>

      <p className="foot-note">只讀工具：不會替你執行上面的指令，要更新請自己貼進終端機。</p>
    </aside>
  );
}
