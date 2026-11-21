import { useState } from "react";
import { localDescription } from "../i18nDesc";
import { useT } from "../i18nContext";
import { kindLabel, statusLabel, type Row } from "../rows";
import { useSettings } from "../settings";

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
  copyLabel: string;
  copiedLabel: string;
  manualLabel: string;
}

function CommandRow({
  label,
  command,
  state,
  onCopy,
  copyLabel,
  copiedLabel,
  manualLabel,
}: CommandRowProps) {
  return (
    <div className="field">
      <dt>{label}</dt>
      <dd>
        <div className="cmd">
          <code>{command}</code>
          <button type="button" className="btn btn--tiny" onClick={() => onCopy(command)}>
            {state === "ok" ? copiedLabel : state === "manual" ? manualLabel : copyLabel}
          </button>
        </div>
      </dd>
    </div>
  );
}

export default function Inspector({ row, onClose }: Props) {
  const t = useT();
  const statusText = statusLabel(row.statusResolved, row.latestResolved, row.checkable, t);
  const { lang } = useSettings();
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

  // 我們自己撰稿的說明有翻譯；上游 metadata（多為英文）保持原文
  const description =
    localDescription(lang, row.ecosystem, row.name, row.kind) ?? row.description;
  const note = row.noteKey
    ? t(row.noteKey, Object.fromEntries(row.noteArgs.map((value, index) => [String(index), value])))
    : row.note;

  return (
    <aside className="inspector" aria-label={row.name}>
      <div className="inspector-head">
        <div className="inspector-bar">
          <span className="inspector-kind">{kindLabel(row.kind, t)}</span>
          <button type="button" className="inspector-close" onClick={onClose}>
            {t("insp.close")}
          </button>
        </div>
        <h2 className="inspector-name">{row.name}</h2>
        <p className="inspector-version">{row.version}</p>
        {statusText && (
          <p className={`inspector-status is-${row.statusResolved}`}>{statusText}</p>
        )}
        {description && <p className="inspector-desc">{description}</p>}
      </div>

      <dl className="fields">
        <div className="field">
          <dt>{t("insp.source")}</dt>
          <dd>{row.source}</dd>
        </div>

        {row.location && (
          <div className="field">
            <dt>{t("insp.location")}</dt>
            <dd className="mono">{row.location}</dd>
          </div>
        )}

        {row.updateCommand && (
          <CommandRow
            label={t("insp.update")}
            command={row.updateCommand}
            state={stateOf(row.updateCommand)}
            onCopy={(text) => void copy(text)}
            copyLabel={t("insp.copy")}
            copiedLabel={t("insp.copied")}
            manualLabel={t("insp.manual")}
          />
        )}

        {row.removeCommand && (
          <CommandRow
            label={t("insp.remove")}
            command={row.removeCommand}
            state={stateOf(row.removeCommand)}
            onCopy={(text) => void copy(text)}
            copyLabel={t("insp.copy")}
            copiedLabel={t("insp.copied")}
            manualLabel={t("insp.manual")}
          />
        )}

        {note && (
          <div className="field">
            <dt>{t("insp.note")}</dt>
            <dd>{note}</dd>
          </div>
        )}
      </dl>

    </aside>
  );
}
