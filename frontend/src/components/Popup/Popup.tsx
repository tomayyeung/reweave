import styles from "./Popup.module.css";

type PopupProps = {
  text: string;
  onClose: () => void;
  /** used for non-confirmation */
  closeText?: string;
  confirmText?: string;
  /** used for confirmation */
  cancelText?: string;
  onConfirm?: () => void;
};

export function Popup({
  text,
  onClose,
  closeText = "Close",
  confirmText = "Confirm",
  cancelText = "Cancel",
  onConfirm,
}: PopupProps) {
  const isConfirmation = onConfirm !== undefined;

  return (
    <div className={styles.overlay} role="presentation">
      <div className={styles.popup} role="dialog" aria-modal="true">
        <p>{text}</p>

        <div className={styles.actions}>
          {isConfirmation ? (
            <>
              <button type="button" onClick={onConfirm}>
                {confirmText}
              </button>
              <button type="button" onClick={onClose}>
                {cancelText}
              </button>
            </>
          ) : (
            <button type="button" onClick={onClose}>
              {closeText}
            </button>
          )}
        </div>
      </div>
    </div>
  );
}
