import { t } from "../i18n";
import { OrbitMark, PlusIcon } from "./icons";
import "./empty-state.css";

export function EmptyState({ disabled, onAdd }: { disabled: boolean; onAdd: () => void }) {
  return (
    <div className="empty">
      <OrbitMark width={44} height={44} className="empty-mark" />
      <h2>{t("empty.title")}</h2>
      <p>{t("empty.body")}</p>
      <button type="button" className="btn btn-primary" disabled={disabled} onClick={onAdd}>
        <PlusIcon />
        {t("footer.add")}
      </button>
    </div>
  );
}
