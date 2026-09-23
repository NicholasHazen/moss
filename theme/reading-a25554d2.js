/* Native disclosures work without JavaScript. Preserve discoverability when
 * mdBook links a search result into a long answer, and include answers in print. */
(() => {
    "use strict";
    const references = [...document.querySelectorAll("details.worked-reference")];
    if (!references.length) return;
    if (new URLSearchParams(location.search).has("highlight") || location.pathname.endsWith("/print.html")) {
        references.forEach(reference => { reference.open = true; });
    }
    function revealAnchor() {
        let id;
        try { id = decodeURIComponent(location.hash.slice(1)); } catch { return; }
        const target = document.getElementById(id);
        if (!target) return;
        let ancestor = target.parentElement;
        let opened = false;
        while (ancestor) {
            if (ancestor instanceof HTMLDetailsElement && !ancestor.open) {
                ancestor.open = true;
                opened = true;
            }
            ancestor = ancestor.parentElement;
        }
        if (opened) target.scrollIntoView();
    }
    revealAnchor();
    window.addEventListener("hashchange", revealAnchor);
    let closedBeforePrint = [];
    window.addEventListener("beforeprint", () => {
        closedBeforePrint = references.filter(reference => !reference.open);
        closedBeforePrint.forEach(reference => { reference.open = true; });
    });
    window.addEventListener("afterprint", () => {
        closedBeforePrint.forEach(reference => { reference.open = false; });
        closedBeforePrint = [];
    });
})();
