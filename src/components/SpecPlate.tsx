import { formatTime } from "../rows";
import type { Inventory } from "../types";

interface Props {
  inventory: Inventory | null;
  total: number;
  outdated: number;
}

export default function SpecPlate({ inventory, total, outdated }: Props) {
  return (
    <section className="plate" aria-label="這台機器">
      <div className="plate-id">
        <h1 className="plate-host">{inventory?.host ?? "正在辨識這台 Mac…"}</h1>
        <dl className="plate-specs">
          <div className="plate-spec">
            <dt>機型</dt>
            <dd>{inventory?.model ?? "—"}</dd>
          </div>
          <div className="plate-spec">
            <dt>系統</dt>
            <dd>macOS {inventory?.os ?? "—"}</dd>
          </div>
          <div className="plate-spec">
            <dt>架構</dt>
            <dd>{inventory?.arch ?? "—"}</dd>
          </div>
        </dl>
      </div>

      <dl className="plate-ledger">
        <div className="ledger-cell">
          <dt>已盤點</dt>
          <dd>{total}</dd>
        </div>
        <div className="ledger-cell">
          <dt>需更新</dt>
          <dd className={outdated > 0 ? "is-warn" : ""}>{outdated}</dd>
        </div>
        <div className="ledger-cell">
          <dt>掃描</dt>
          <dd className="ledger-time">{inventory ? formatTime(inventory.scannedAtMs) : "—"}</dd>
        </div>
      </dl>
    </section>
  );
}
