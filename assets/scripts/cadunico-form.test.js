// @vitest-environment jsdom
import { beforeEach, describe, expect, it } from "vitest";
import {
  activateNextField,
  activatePreviousField,
  bootstrapCadUnicoForm,
  clearInvalidFields,
  nextTabId,
  normalizeCep,
  normalizeCpfCnpj,
  parseInvalidFields,
} from "./cadunico-form.js";

beforeEach(() => {
  document.body.innerHTML = "";
});

describe("cadunico keyboard helpers", () => {
  it("activateNextField should move to the next focusable field", () => {
    const fields = [{ id: "a" }, { id: "b" }, { id: "c" }];
    expect(activateNextField(fields, "a")).toBe("b");
  });

  it("activatePreviousField should move to the previous focusable field", () => {
    const fields = [{ id: "a" }, { id: "b" }, { id: "c" }];
    expect(activatePreviousField(fields, "c")).toBe("b");
  });

  it("nextTabId should move left and right through ordered tabs", () => {
    const tabs = ["dados-principais", "endereco", "parametros", "cobranca"];
    expect(nextTabId(tabs, "endereco", "right")).toBe("parametros");
    expect(nextTabId(tabs, "endereco", "left")).toBe("dados-principais");
  });

  it("normalizeCpfCnpj should strip formatting", () => {
    expect(normalizeCpfCnpj("123.456.789-01")).toBe("12345678901");
  });

  it("normalizeCep should strip formatting", () => {
    expect(normalizeCep("01001-000")).toBe("01001000");
  });

  it("bootstrapCadUnicoForm should opt 422 htmx responses into swapping for the modal root", () => {
    document.body.innerHTML = `
      <main data-cadunico-root>
        <form data-cadunico-form></form>
        <div id="cadunico-modal-root"></div>
      </main>
    `;

    const root = document.querySelector("[data-cadunico-root]");
    const modalRoot = document.querySelector("#cadunico-modal-root");

    bootstrapCadUnicoForm(root);

    const detail = {
      xhr: { status: 422 },
      target: modalRoot,
      shouldSwap: false,
      isError: true,
    };

    document.body.dispatchEvent(new CustomEvent("htmx:beforeSwap", { detail }));

    expect(detail.shouldSwap).toBe(true);
    expect(detail.isError).toBe(false);
  });

  it("create form should keep first tab active on bootstrap", () => {
    document.body.innerHTML = `
      <main data-cadunico-root>
        <form data-cadunico-form>
          <button data-tab-trigger data-tab-id="dados-principais"></button>
          <button data-tab-trigger data-tab-id="endereco"></button>
          <section class="tab-panel" data-tab-panel="dados-principais"></section>
          <section class="tab-panel" data-tab-panel="endereco"></section>
        </form>
        <div id="cadunico-modal-root"></div>
      </main>
    `;

    bootstrapCadUnicoForm(document.querySelector("[data-cadunico-root]"));
    expect(
      document
        .querySelector('[data-tab-trigger][data-tab-id="dados-principais"]')
        .classList
        .contains("is-active"),
    ).toBe(true);
  });

  it("parseInvalidFields should read comma separated field names", () => {
    expect(parseInvalidFields("cpf_cnpj, fantasia ,cep")).toEqual([
      "cpf_cnpj",
      "fantasia",
      "cep",
    ]);
  });

  it("bootstrapCadUnicoForm should mark invalid fields after modal swap", () => {
    document.body.innerHTML = `
      <main data-cadunico-root>
        <form data-cadunico-form>
          <button data-tab-trigger data-tab-id="dados-principais" class="is-active"></button>
          <button data-tab-trigger data-tab-id="endereco"></button>
          <section class="tab-panel is-active" data-tab-panel="dados-principais">
            <div class="field"><input id="cpf_cnpj" name="cpf_cnpj" /></div>
          </section>
          <section class="tab-panel" data-tab-panel="endereco" hidden>
            <div class="field"><input id="fantasia" name="fantasia" /></div>
          </section>
        </form>
        <div id="cadunico-modal-root">
          <div id="backend-error-modal" data-invalid-fields="fantasia"></div>
        </div>
      </main>
    `;

    const root = document.querySelector("[data-cadunico-root]");
    const modalRoot = document.querySelector("#cadunico-modal-root");

    bootstrapCadUnicoForm(root);

    modalRoot.dispatchEvent(new CustomEvent("htmx:afterSwap", { bubbles: true, detail: { target: modalRoot } }));

    expect(document.querySelector("#fantasia").getAttribute("aria-invalid")).toBe("true");
    expect(document.querySelector("#fantasia").closest(".field").dataset.invalid).toBe("true");
    expect(document.querySelector('[data-tab-trigger][data-tab-id="endereco"]').classList.contains("is-active")).toBe(true);
  });

  it("bootstrapCadUnicoForm should clear invalid state when the user edits a field", () => {
    document.body.innerHTML = `
      <main data-cadunico-root>
        <form data-cadunico-form>
          <section class="tab-panel is-active">
            <div class="field"><input id="cpf_cnpj" name="cpf_cnpj" aria-invalid="true" /></div>
          </section>
        </form>
        <div id="cadunico-modal-root"></div>
      </main>
    `;

    const root = document.querySelector("[data-cadunico-root]");
    const input = document.querySelector("#cpf_cnpj");
    input.closest(".field").dataset.invalid = "true";

    bootstrapCadUnicoForm(root);
    input.dispatchEvent(new Event("input", { bubbles: true }));

    expect(input.hasAttribute("aria-invalid")).toBe(false);
    expect(input.closest(".field").hasAttribute("data-invalid")).toBe(false);
  });

  it("clearInvalidFields should remove invalid markers from the form", () => {
    document.body.innerHTML = `
      <form>
        <div class="field" data-invalid="true"><input id="cpf_cnpj" aria-invalid="true" /></div>
      </form>
    `;

    clearInvalidFields(document.querySelector("form"));

    expect(document.querySelector(".field").hasAttribute("data-invalid")).toBe(false);
    expect(document.querySelector("#cpf_cnpj").hasAttribute("aria-invalid")).toBe(false);
  });

  it("bootstrapCadUnicoForm should close the modal on Escape when focus is inside the dialog", () => {
    document.body.innerHTML = `
      <main data-cadunico-root>
        <form data-cadunico-form>
          <section class="tab-panel is-active" data-tab-panel="dados-principais">
            <div class="field"><input id="cpf_cnpj" name="cpf_cnpj" /></div>
          </section>
        </form>
        <div id="cadunico-modal-root">
          <div id="backend-error-modal" tabindex="-1">
            <button type="button" data-close-modal>Fechar</button>
          </div>
        </div>
      </main>
    `;

    const root = document.querySelector("[data-cadunico-root]");
    const modalRoot = document.querySelector("#cadunico-modal-root");
    const modal = document.querySelector("#backend-error-modal");

    bootstrapCadUnicoForm(root);
    modal.focus();
    modalRoot.dispatchEvent(new CustomEvent("htmx:afterSwap", { bubbles: true, detail: { target: modalRoot } }));

    document.dispatchEvent(new KeyboardEvent("keydown", { key: "Escape", bubbles: true }));

    expect(modalRoot.innerHTML).toBe("");
  });
});
