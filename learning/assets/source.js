/* A standalone enhancement for exact, selectable source-answer text. */
(() => {
  "use strict";
  const button = document.querySelector("[data-copy-source]");
  const source = document.getElementById("source-code");
  const status = document.getElementById("copy-status");
  if (!button || !source || !status) return;
  button.hidden = false;
  button.addEventListener("click", async () => {
    button.disabled = true;
    status.textContent = "Copying…";
    try {
      if (typeof navigator.clipboard?.writeText !== "function") {
        throw new Error("Clipboard unavailable");
      }
      await navigator.clipboard.writeText(source.textContent);
      status.textContent = "Copied the complete file.";
    } catch {
      // Selection helps manual copying but is never described as clipboard success.
      status.textContent = "Automatic copy was unavailable. Select and copy the source manually, or use Download raw file.";
      try {
        const selection = window.getSelection();
        if (selection) {
          const range = document.createRange();
          range.selectNodeContents(source);
          selection.removeAllRanges();
          selection.addRange(range);
        }
      } catch {
        // The raw-download route remains available if selection is unsupported.
      }
    } finally {
      button.disabled = false;
    }
  });
})();
