import { useT } from "../i18nContext";
import { formatTime } from "../rows";
import { useSettings } from "../settings";
import type { Inventory } from "../types";

interface Props {
  inventory: Inventory | null;
  total: number;
  outdated: number;
}

export default function SpecPlate({ inventory, total, outdated }: Props) {
  const t = useT();
  const { lang } = useSettings();

  return (
    <section className="plate" aria-label="machine">
      <div className="plate-id">
        <h1 className="plate-host">{inventory?.host ?? t("plate.identifying")}</h1>
        <dl className="plate-specs">
          <div className="plate-spec">
            <dt>{t("plate.model")}</dt>
            <dd>{inventory?.model ?? "—"}</dd>
          </div>
          <div className="plate-spec">
            <dt>{t("plate.os")}</dt>
            <dd>macOS {inventory?.os ?? "—"}</dd>
          </div>
          <div className="plate-spec">
            <dt>{t("plate.arch")}</dt>
            <dd>{inventory?.arch ?? "—"}</dd>
          </div>
        </dl>
      </div>

      <dl className="plate-ledger">
        <div className="ledger-cell">
          <dt>{t("plate.items")}</dt>
          <dd>{total}</dd>
        </div>
        <div className="ledger-cell">
          <dt>{t("plate.outdated")}</dt>
          <dd className={outdated > 0 ? "is-warn" : ""}>{outdated}</dd>
        </div>
        <div className="ledger-cell">
          <dt>{t("plate.scan")}</dt>
          <dd className="ledger-time">
            {inventory ? formatTime(inventory.scannedAtMs, lang) : "—"}
          </dd>
        </div>
      </dl>
    </section>
  );
}
