import { useState } from "react";
import { kindLabel, statusLabel, type Row } from "../rows";

interface Props {
  row: Row;
  onClose: () => void;
}

type CopyState = "idle" | "ok" | "manual";

interface CommandRowProps {
  label: string;
  command: string;
  state: CopyState;
  onCopy: (text: string) => void;
}

function CommandRow({ label, command, state, onCopy }: CommandRowProps) {
  return (
    <div className="field">
      <dt>{label}</dt>
      <dd>
        <div className="cmd">
          <code>{command}</code>
          <button type="button" className="btn btn--tiny" onClick={() => onCopy(command)}>
            {state === "ok" ? "已複製" : state === "manual" ? "請手動複製" : "複製"}
          </button>
        </div>
      </dd>
    </div>
  );
}

export default function Inspector({ row, onClose }: Props) {
  const [copiedText, setCopiedText] = useState<string | null>(null);
  const [failedText, setFailedText] = useState<string | null>(null);

  const copy = async (text: string) => {
    try {
      await navigator.clipboard.writeText(text);
      setCopiedText(text);
      setFailedText(null);
    } catch {
      setFailedText(text);
      setCopiedText(null);
    }
    window.setTimeout(() => {
      setCopiedText(null);
      setFailedText(null);
    }, 2200);
  };

  const stateOf = (command: string | null): CopyState => {
    if (command === null) return "idle";
    if (copiedText === command) return "ok";
    if (failedText === command) return "manual";
    return "idle";
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
        {row.description && <p className="inspector-desc">{row.description}</p>}
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

        {row.updateCommand && (
          <CommandRow
            label="更新指令"
            command={row.updateCommand}
            state={stateOf(row.updateCommand)}
            onCopy={(text) => void copy(text)}
          />
        )}

        {row.removeCommand && (
          <CommandRow
            label="移除指令"
            command={row.removeCommand}
            state={stateOf(row.removeCommand)}
            onCopy={(text) => void copy(text)}
          />
        )}

        {row.note && (
          <div className="field">
            <dt>備註</dt>
            <dd>{row.note}</dd>
          </div>
        )}
      </dl>

      <p className="foot-note">
        只讀工具：上面的指令不會被自動執行。移除指令貼進終端機後會真的刪掉這個套件，動手前先確認。
      </p>
    </aside>
  );
}
