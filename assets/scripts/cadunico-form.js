const FOCUSABLE_SELECTOR =
  "input:not([type='hidden']):not([disabled]), select:not([disabled]), textarea:not([disabled]), button:not([disabled])";

export function digitsOnly(value) {
  return value.replace(/\D+/g, "");
}

export function normalizeCpfCnpj(value) {
  return digitsOnly(value).slice(0, 14);
}

export function normalizeCep(value) {
  return digitsOnly(value).slice(0, 8);
}

export function activateNextField(fields, currentId) {
  const index = fields.findIndex((field) => field.id === currentId);
  return index >= 0 && index < fields.length - 1 ? fields[index + 1].id : currentId;
}

export function activatePreviousField(fields, currentId) {
  const index = fields.findIndex((field) => field.id === currentId);
  return index > 0 ? fields[index - 1].id : currentId;
}

export function nextTabId(tabs, currentId, direction) {
  const index = tabs.indexOf(currentId);
  if (index === -1) return currentId;
  if (direction === "right") return tabs[Math.min(index + 1, tabs.length - 1)];
  return tabs[Math.max(index - 1, 0)];
}

function formatCpfCnpj(value) {
  const digits = normalizeCpfCnpj(value);
  if (digits.length <= 11) {
    return digits
      .replace(/^(\d{3})(\d)/, "$1.$2")
      .replace(/^(\d{3})\.(\d{3})(\d)/, "$1.$2.$3")
      .replace(/\.(\d{3})(\d)/, ".$1-$2");
  }

  return digits
    .replace(/^(\d{2})(\d)/, "$1.$2")
    .replace(/^(\d{2})\.(\d{3})(\d)/, "$1.$2.$3")
    .replace(/\.(\d{3})(\d)/, ".$1/$2")
    .replace(/(\d{4})(\d)/, "$1-$2");
}

function formatCep(value) {
  const digits = normalizeCep(value);
  return digits.replace(/^(\d{5})(\d)/, "$1-$2");
}

function formatPhone(value) {
  const digits = digitsOnly(value).slice(0, 11);
  if (digits.length <= 10) {
    return digits
      .replace(/^(\d{2})(\d)/, "($1) $2")
      .replace(/(\d{4})(\d)/, "$1-$2");
  }

  return digits
    .replace(/^(\d{2})(\d)/, "($1) $2")
    .replace(/(\d{5})(\d)/, "$1-$2");
}

function orderedTabs(root) {
  return Array.from(root.querySelectorAll("[data-tab-trigger]")).map((tab) =>
    tab.getAttribute("data-tab-id"),
  );
}

function fieldsForActivePanel(root) {
  const panel = root.querySelector(".tab-panel.is-active");
  if (!panel) return [];

  return Array.from(panel.querySelectorAll(FOCUSABLE_SELECTOR)).filter(
    (element) => element.offsetParent !== null || element === document.activeElement,
  );
}

function fieldIds(fields) {
  return fields.map((field, index) => {
    if (!field.id) {
      field.id = `${field.name || "cadunico-field"}-${index}`;
    }
    return { id: field.id, element: field };
  });
}

function activateTab(root, tabId) {
  const tabs = Array.from(root.querySelectorAll("[data-tab-trigger]"));
  const panels = Array.from(root.querySelectorAll("[data-tab-panel]"));

  tabs.forEach((tab) => {
    const isActive = tab.getAttribute("data-tab-id") === tabId;
    tab.classList.toggle("is-active", isActive);
    tab.setAttribute("aria-selected", String(isActive));
  });

  panels.forEach((panel) => {
    const isActive = panel.getAttribute("data-tab-panel") === tabId;
    panel.classList.toggle("is-active", isActive);
    panel.toggleAttribute("hidden", !isActive);
  });
}

function focusFieldById(root, targetId) {
  const fields = fieldIds(fieldsForActivePanel(root));
  const target = fields.find((field) => field.id === targetId);
  target?.element.focus();
}

function focusFirstField(root) {
  const [firstField] = fieldsForActivePanel(root);
  firstField?.focus();
}

function closeModal(modalRoot, restoreFocusTo) {
  modalRoot.innerHTML = "";
  restoreFocusTo?.focus();
}

function bindMasks(root) {
  root.addEventListener("input", (event) => {
    const target = event.target;
    if (!(target instanceof HTMLInputElement)) return;

    const mask = target.dataset.mask;
    if (mask === "cpf_cnpj") {
      target.value = formatCpfCnpj(target.value);
    }
    if (mask === "cep") {
      target.value = formatCep(target.value);
    }
    if (mask === "telefone") {
      target.value = formatPhone(target.value);
    }
  });
}

function bindTabs(root) {
  root.addEventListener("click", (event) => {
    const target = event.target;
    if (!(target instanceof HTMLElement)) return;

    const trigger = target.closest("[data-tab-trigger]");
    if (!(trigger instanceof HTMLElement)) return;

    const tabId = trigger.getAttribute("data-tab-id");
    if (!tabId) return;
    activateTab(root, tabId);
    focusFirstField(root);
  });
}

function bindKeyboard(root, form, modalRoot) {
  let restoreFocusTo = null;

  form.addEventListener("keydown", (event) => {
    const activeElement = document.activeElement;
    if (!(activeElement instanceof HTMLElement)) return;

    const fields = fieldIds(fieldsForActivePanel(root));
    const currentId = activeElement.id;

    if ((event.key === "Tab" && !event.shiftKey) || event.key === "ArrowDown") {
      event.preventDefault();
      focusFieldById(root, activateNextField(fields, currentId));
      return;
    }

    if ((event.key === "Tab" && event.shiftKey) || event.key === "ArrowUp") {
      event.preventDefault();
      focusFieldById(root, activatePreviousField(fields, currentId));
      return;
    }

    if (event.ctrlKey && (event.key === "ArrowRight" || event.key === "ArrowLeft")) {
      event.preventDefault();
      const tabs = orderedTabs(root);
      const activeTab = root.querySelector("[data-tab-trigger].is-active");
      const currentTabId = activeTab?.getAttribute("data-tab-id") || tabs[0];
      const targetTabId = nextTabId(
        tabs,
        currentTabId,
        event.key === "ArrowRight" ? "right" : "left",
      );
      activateTab(root, targetTabId);
      focusFirstField(root);
      return;
    }

    if (event.ctrlKey && event.key.toLowerCase() === "s") {
      event.preventDefault();
      form.requestSubmit();
      return;
    }

    if (event.key === "Escape" && modalRoot.firstElementChild) {
      event.preventDefault();
      closeModal(modalRoot, restoreFocusTo);
    }
  });

  root.addEventListener("click", (event) => {
    const target = event.target;
    if (!(target instanceof HTMLElement)) return;
    if (!target.closest("[data-close-modal]")) return;
    closeModal(modalRoot, restoreFocusTo);
  });

  document.body.addEventListener("htmx:afterSwap", (event) => {
    const target = event.detail?.target;
    if (target !== modalRoot) return;

    const modal = modalRoot.querySelector("#backend-error-modal");
    if (!(modal instanceof HTMLElement)) return;

    restoreFocusTo = document.activeElement instanceof HTMLElement ? document.activeElement : null;
    modal.focus();
  });
}

export function bootstrapCadUnicoForm(
  root = document.querySelector("[data-cadunico-root]"),
) {
  if (!(root instanceof HTMLElement)) return;

  const form = root.querySelector("[data-cadunico-form]");
  const modalRoot = root.querySelector("#cadunico-modal-root");
  if (!(form instanceof HTMLFormElement) || !(modalRoot instanceof HTMLElement)) return;

  const firstTabId = orderedTabs(root)[0];
  if (firstTabId) {
    activateTab(root, firstTabId);
  }

  bindTabs(root);
  bindMasks(root);
  bindKeyboard(root, form, modalRoot);
}

if (typeof document !== "undefined") {
  bootstrapCadUnicoForm();
}
