let activeTeardown = null;

export function nextRowIndex(total, current, direction) {
  if (total === 0) return -1;
  if (direction === "down") return Math.min(current + 1, total - 1);
  return Math.max(current - 1, 0);
}

function visibleRows(root) {
  return Array.from(root.querySelectorAll("[data-row]")).filter((row) => !row.hidden);
}

function activeRowIndex(rows) {
  return rows.findIndex((row) => row.dataset.rowActive === "true");
}

function pageButton(root, direction) {
  return root.querySelector(direction === "left" ? "[data-page-prev]" : "[data-page-next]");
}

function currentDialog(root) {
  return root.querySelector("[data-delete-dialog]");
}

function popoverRoot(root) {
  return root.querySelector("[data-row-menu-popover-root]");
}

function currentPopover(root) {
  return root.querySelector("[data-row-menu-popover]");
}

function openDialog(dialog) {
  if (!(dialog instanceof HTMLDialogElement)) return;
  if (typeof dialog.showModal === "function") {
    dialog.showModal();
    return;
  }

  dialog.setAttribute("open", "");
}

function closeDialog(dialog, value) {
  if (!(dialog instanceof HTMLDialogElement)) return;
  if (typeof dialog.close === "function") {
    dialog.close(value);
    return;
  }

  dialog.removeAttribute("open");
}

function currentRowFromPopover(root) {
  const menu = currentPopover(root);
  const rowId = menu?.getAttribute("data-row-id");
  if (!rowId) return null;
  return root.querySelector(`[data-row][data-row-id="${rowId}"]`);
}

function closeFloatingMenu(root, { restoreFocus = true } = {}) {
  const host = popoverRoot(root);
  if (!(host instanceof HTMLElement)) return false;

  const row = restoreFocus ? currentRowFromPopover(root) : null;
  host.replaceChildren();

  if (restoreFocus && row instanceof HTMLElement) {
    row.focus();
  }

  return true;
}

function setActiveRow(root, targetIndex) {
  const rows = visibleRows(root);
  rows.forEach((row, index) => {
    const isActive = index === targetIndex;
    row.tabIndex = isActive ? 0 : -1;
    row.dataset.rowActive = String(isActive);
  });

  const row = rows[targetIndex];
  row?.focus();
}

function positionPopover(menu, row) {
  if (!(menu instanceof HTMLElement) || !(row instanceof HTMLElement)) return;

  const anchor = row.querySelector(".cadunico-list-actions") ?? row.lastElementChild ?? row;
  if (!(anchor instanceof HTMLElement)) return;

  const rect = anchor.getBoundingClientRect();
  menu.style.top = `${rect.bottom + 8}px`;
  menu.style.left = `${Math.max(16, rect.right - 220)}px`;
  menu.style.minWidth = `${Math.max(180, rect.width || 180)}px`;
}

function openFloatingMenu(root, row) {
  const host = popoverRoot(root);
  if (!(host instanceof HTMLElement) || !(row instanceof HTMLElement)) return;

  host.innerHTML = `
    <div class="row-menu-popover" data-row-menu-popover data-row-id="${row.dataset.rowId ?? ""}" hidden>
      <button type="button" disabled aria-disabled="true">Editar (em breve)</button>
      <button
        type="button"
        data-row-delete
        data-row-id="${row.dataset.rowId ?? ""}"
        data-row-name="${row.dataset.rowName ?? ""}"
      >
        Excluir
      </button>
    </div>
  `;

  const menu = host.querySelector("[data-row-menu-popover]");
  if (!(menu instanceof HTMLElement)) return;

  menu.hidden = false;
  positionPopover(menu, row);
  const firstAction = menu.querySelector("[data-row-delete]");
  if (firstAction instanceof HTMLElement) {
    firstAction.focus();
  }
}

function debounceSearch(root) {
  const search = root.querySelector("#cadunico-search");
  if (!(search instanceof HTMLInputElement)) return () => {};

  let timerId = 0;
  const onInput = () => {
    window.clearTimeout(timerId);
    timerId = window.setTimeout(() => {
      const htmx = window.htmx;
      const region = root.querySelector("#cadunico-list-region");
      if (!htmx?.ajax || !(region instanceof HTMLElement)) return;

      const params = new URLSearchParams();
      const value = search.value.trim();
      if (value) {
        params.set("q", value);
      }

      closeFloatingMenu(root, { restoreFocus: false });
      const url = params.toString() ? `/cadunico/lista?${params}` : "/cadunico/lista";
      htmx.ajax("GET", url, { target: region, swap: "outerHTML" });
    }, 250);
  };

  search.addEventListener("input", onInput);
  return () => {
    window.clearTimeout(timerId);
    search.removeEventListener("input", onInput);
  };
}

function bindDeleteDialog(root) {
  let pendingRowId = null;
  let pendingRowName = "";

  const onClick = (event) => {
    const target = event.target;
    if (!(target instanceof HTMLElement)) return;

    const deleteButton = target.closest("[data-row-delete]");
    if (!(deleteButton instanceof HTMLElement)) return;

    const dialog = currentDialog(root);
    if (!(dialog instanceof HTMLDialogElement)) return;

    pendingRowId = deleteButton.getAttribute("data-row-id");
    pendingRowName = deleteButton.getAttribute("data-row-name") ?? "cadastro selecionado";

    const summary = dialog.querySelector("[data-delete-summary]");
    if (summary instanceof HTMLElement) {
      summary.textContent = `Confirme a exclusao de ${pendingRowName}.`;
    }

    openDialog(dialog);
  };

  const onConfirm = () => {
    if (!pendingRowId) return;

    const htmx = window.htmx;
    const region = root.querySelector("#cadunico-list-region");
    const search = root.querySelector("#cadunico-search");

    if (htmx?.ajax && region instanceof HTMLElement) {
      const params = new URLSearchParams();
      if (search instanceof HTMLInputElement && search.value.trim()) {
        params.set("q", search.value.trim());
      }

      const suffix = params.toString() ? `?${params}` : "";
      htmx.ajax("DELETE", `/cadunico/${pendingRowId}${suffix}`, {
        target: region,
        swap: "outerHTML",
      });
    }

    const dialog = currentDialog(root);
    closeFloatingMenu(root, { restoreFocus: false });
    closeDialog(dialog, "confirm");
    pendingRowId = null;
    pendingRowName = "";
  };

  const dialog = currentDialog(root);
  const confirm = dialog?.querySelector("[data-confirm-delete]");

  root.addEventListener("click", onClick);
  confirm?.addEventListener("click", onConfirm);

  return () => {
    root.removeEventListener("click", onClick);
    confirm?.removeEventListener("click", onConfirm);
  };
}

export function bootstrapCadUnicoList(
  root = document.querySelector("[data-cadunico-list-root]"),
) {
  if (!(root instanceof HTMLElement)) return;

  activeTeardown?.();

  const search = root.querySelector("#cadunico-search");
  const createLink = root.querySelector("[data-create-link]");

  const cleanupSearch = debounceSearch(root);
  const cleanupDelete = bindDeleteDialog(root);

  const onDocumentClick = (event) => {
    const target = event.target;
    if (!(target instanceof HTMLElement)) return;
    if (target.closest("[data-row-menu-popover]")) return;
    closeFloatingMenu(root, { restoreFocus: false });
  };

  const onHtmxAfterSwap = (event) => {
    const target = event.target;
    if (!(target instanceof HTMLElement)) return;
    if (!root.contains(target)) return;
    bootstrapCadUnicoList(root);
  };

  const onKeyDown = (event) => {
    const rows = visibleRows(root);
    const key = event.key.toLowerCase();
    const activeElement = document.activeElement;
    const inSearch = activeElement === search;

    if (event.ctrlKey && key === "k") {
      event.preventDefault();
      if (search instanceof HTMLElement) {
        search.focus();
        if ("select" in search) {
          search.select();
        }
      }
      return;
    }

    if (event.ctrlKey && key === "n") {
      event.preventDefault();
      createLink?.click();
      return;
    }

    if (event.key === "Escape") {
      if (currentPopover(root)) {
        event.preventDefault();
        closeFloatingMenu(root);
        return;
      }

      const dialog = currentDialog(root);
      if (dialog instanceof HTMLDialogElement && dialog.open) {
        closeDialog(dialog, "cancel");
        const row = currentRowFromPopover(root);
        row?.focus();
        event.preventDefault();
        return;
      }

      if (inSearch && search instanceof HTMLElement) {
        search.blur();
        event.preventDefault();
      }
      return;
    }

    if (inSearch) return;

    if (event.key === "ArrowDown" || event.key === "ArrowUp") {
      if (rows.length === 0) return;

      event.preventDefault();
      closeFloatingMenu(root, { restoreFocus: false });
      const direction = event.key === "ArrowDown" ? "down" : "up";
      const currentIndex = activeRowIndex(rows);
      const targetIndex = nextRowIndex(rows.length, currentIndex, direction);
      setActiveRow(root, targetIndex);
      return;
    }

    if (event.key === "ArrowRight" || event.key === "ArrowLeft") {
      const button = pageButton(root, event.key === "ArrowRight" ? "right" : "left");
      if (!(button instanceof HTMLButtonElement) || button.disabled) return;

      event.preventDefault();
      closeFloatingMenu(root, { restoreFocus: false });
      button.click();
      return;
    }

    if (event.key === "Enter") {
      const currentIndex = activeRowIndex(rows);
      const row = rows[currentIndex];
      if (!(row instanceof HTMLElement)) return;

      event.preventDefault();
      openFloatingMenu(root, row);
    }
  };

  document.addEventListener("click", onDocumentClick);
  document.addEventListener("keydown", onKeyDown);
  document.body.addEventListener("htmx:afterSwap", onHtmxAfterSwap);

  activeTeardown = () => {
    cleanupSearch();
    cleanupDelete();
    document.removeEventListener("click", onDocumentClick);
    document.removeEventListener("keydown", onKeyDown);
    document.body.removeEventListener("htmx:afterSwap", onHtmxAfterSwap);
  };
}

if (typeof document !== "undefined") {
  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", () => bootstrapCadUnicoList(), { once: true });
  } else {
    bootstrapCadUnicoList();
  }
}
