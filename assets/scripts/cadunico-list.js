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

function closeAllMenus(root) {
  root.querySelectorAll("[data-row-menu]").forEach((menu) => {
    menu.hidden = true;
  });
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

function openMenuForRow(row) {
  const root = row.closest("[data-cadunico-list-root]");
  if (!(root instanceof HTMLElement)) return;

  closeAllMenus(root);
  const menu = row.querySelector("[data-row-menu]");
  if (!(menu instanceof HTMLElement)) return;

  menu.hidden = false;
  const firstAction = menu.querySelector("button:not([disabled]):not([aria-disabled='true'])");
  if (firstAction instanceof HTMLElement) {
    firstAction.focus();
  }
}

function closeFocusedMenu(root) {
  const activeElement = document.activeElement;
  if (!(activeElement instanceof HTMLElement)) return false;

  const menu = activeElement.closest("[data-row-menu]");
  if (!(menu instanceof HTMLElement)) return false;

  menu.hidden = true;
  const row = menu.closest("[data-row]");
  if (row instanceof HTMLElement) {
    row.focus();
  }
  return true;
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
  const dialog = root.querySelector("[data-delete-dialog]");
  if (!(dialog instanceof HTMLDialogElement)) return () => {};

  let pendingRowId = null;
  const summary = dialog.querySelector("[data-delete-summary]");
  const confirm = dialog.querySelector("[data-confirm-delete]");

  const onClick = (event) => {
    const target = event.target;
    if (!(target instanceof HTMLElement)) return;

    const deleteButton = target.closest("[data-row-delete]");
    if (!(deleteButton instanceof HTMLElement)) return;

    const row = deleteButton.closest("[data-row]");
    if (!(row instanceof HTMLElement)) return;

    pendingRowId = row.dataset.rowId ?? null;
    if (summary instanceof HTMLElement) {
      const rowName = deleteButton.getAttribute("data-row-name") ?? "cadastro selecionado";
      summary.textContent = `Confirme a exclusao de ${rowName}.`;
    }
    dialog.showModal();
  };

  const onClose = (event) => {
    const target = event.target;
    if (target !== confirm || !pendingRowId) return;

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
    pendingRowId = null;
  };

  root.addEventListener("click", onClick);
  dialog.addEventListener("close", onClose);

  return () => {
    root.removeEventListener("click", onClick);
    dialog.removeEventListener("close", onClose);
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
      if (createLink instanceof HTMLElement) {
        createLink.click();
      }
      return;
    }

    if (event.key === "Escape") {
      if (closeFocusedMenu(root)) {
        event.preventDefault();
        return;
      }

      const dialog = root.querySelector("[data-delete-dialog]");
      if (dialog instanceof HTMLDialogElement && dialog.open) {
        dialog.close("cancel");
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
      button.click();
      return;
    }

    if (event.key === "Enter") {
      const currentIndex = activeRowIndex(rows);
      const row = rows[currentIndex];
      if (!(row instanceof HTMLElement)) return;

      event.preventDefault();
      openMenuForRow(row);
    }
  };

  document.addEventListener("keydown", onKeyDown);

  activeTeardown = () => {
    cleanupSearch();
    cleanupDelete();
    document.removeEventListener("keydown", onKeyDown);
  };
}

if (typeof document !== "undefined") {
  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", () => bootstrapCadUnicoList(), { once: true });
  } else {
    bootstrapCadUnicoList();
  }
}
