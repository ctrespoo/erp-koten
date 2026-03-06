// @vitest-environment jsdom
import { beforeEach, describe, expect, it, vi } from "vitest";
import {
  bootstrapCadUnicoList,
  clampPopoverPosition,
  nextRowIndex,
} from "./cadunico-list.js";

beforeEach(() => {
  document.body.innerHTML = "";
});

function mountList() {
  document.body.innerHTML = `
    <main data-cadunico-list-root>
      <section class="cadunico-list-toolbar">
        <input id="cadunico-search" type="search" />
        <a href="/cadunico/criar" data-create-link>Novo cadastro</a>
      </section>
      <section id="cadunico-list-region" data-list-region tabindex="0">
        <div class="cadunico-list-table">
          <table>
            <tbody>
              <tr data-row data-row-id="10" data-row-name="Registro 1" tabindex="-1">
                <td>Registro 1</td>
                <td class="cadunico-list-actions"></td>
              </tr>
              <tr data-row data-row-id="11" data-row-name="Registro 2" tabindex="-1">
                <td>Registro 2</td>
                <td class="cadunico-list-actions"></td>
              </tr>
            </tbody>
          </table>
        </div>
        <footer>
          <button type="button" data-page-prev>Anterior</button>
          <button type="button" data-page-next>Proxima</button>
        </footer>
      </section>
      <div data-row-menu-popover-root></div>
      <dialog data-delete-dialog>
        <form method="dialog">
          <p data-delete-summary>Confirme a exclusao do cadastro selecionado.</p>
          <button type="submit" value="cancel">Cancelar</button>
          <button type="submit" data-confirm-delete>Excluir</button>
        </form>
      </dialog>
    </main>
  `;

  const root = document.querySelector("[data-cadunico-list-root]");
  bootstrapCadUnicoList(root);
  return root;
}

describe("cadunico list keyboard helpers", () => {
  it("nextRowIndex should move within the row bounds", () => {
    expect(nextRowIndex(2, -1, "down")).toBe(0);
    expect(nextRowIndex(2, 0, "down")).toBe(1);
    expect(nextRowIndex(2, 1, "down")).toBe(1);
    expect(nextRowIndex(2, 1, "up")).toBe(0);
  });

  it("bootstrapCadUnicoList should focus search on Ctrl+K and blur it on Escape", () => {
    mountList();
    const search = document.querySelector("#cadunico-search");

    document.dispatchEvent(new KeyboardEvent("keydown", { key: "k", ctrlKey: true, bubbles: true }));
    expect(document.activeElement).toBe(search);

    document.dispatchEvent(new KeyboardEvent("keydown", { key: "Escape", bubbles: true }));
    expect(document.activeElement).not.toBe(search);
  });

  it("bootstrapCadUnicoList should navigate rows with ArrowUp and ArrowDown", () => {
    mountList();
    const rows = Array.from(document.querySelectorAll("[data-row]"));

    document.dispatchEvent(new KeyboardEvent("keydown", { key: "ArrowDown", bubbles: true }));
    expect(document.activeElement).toBe(rows[0]);

    document.dispatchEvent(new KeyboardEvent("keydown", { key: "ArrowDown", bubbles: true }));
    expect(document.activeElement).toBe(rows[1]);

    document.dispatchEvent(new KeyboardEvent("keydown", { key: "ArrowUp", bubbles: true }));
    expect(document.activeElement).toBe(rows[0]);
  });

  it("menu navigation should consume ArrowUp/ArrowDown when row menu is open", () => {
    mountList();
    const rows = Array.from(document.querySelectorAll("[data-row]"));

    document.dispatchEvent(new KeyboardEvent("keydown", { key: "ArrowDown", bubbles: true }));
    document.dispatchEvent(new KeyboardEvent("keydown", { key: "Enter", bubbles: true }));

    const menuButton = document.querySelector("[data-row-delete]");
    expect(document.activeElement).toBe(menuButton);

    document.dispatchEvent(new KeyboardEvent("keydown", { key: "ArrowDown", bubbles: true }));

    expect(document.activeElement).toBe(menuButton);
    expect(rows[0].dataset.rowActive).toBe("true");
  });

  it("bootstrapCadUnicoList should change pages with ArrowLeft and ArrowRight outside the search field", () => {
    mountList();
    const nextButton = document.querySelector("[data-page-next]");
    const prevButton = document.querySelector("[data-page-prev]");
    const nextSpy = vi.fn();
    const prevSpy = vi.fn();

    nextButton.addEventListener("click", nextSpy);
    prevButton.addEventListener("click", prevSpy);

    document.dispatchEvent(new KeyboardEvent("keydown", { key: "ArrowRight", bubbles: true }));
    document.dispatchEvent(new KeyboardEvent("keydown", { key: "ArrowLeft", bubbles: true }));

    expect(nextSpy).toHaveBeenCalledTimes(1);
    expect(prevSpy).toHaveBeenCalledTimes(1);
  });

  it("bootstrapCadUnicoList should open a floating menu for the active row", () => {
    const root = mountList();
    const popoverRoot = root.querySelector("[data-row-menu-popover-root]");

    document.dispatchEvent(new KeyboardEvent("keydown", { key: "ArrowDown", bubbles: true }));
    document.dispatchEvent(new KeyboardEvent("keydown", { key: "Enter", bubbles: true }));

    const menu = popoverRoot.querySelector("[data-row-menu-popover]");
    expect(menu.hidden).toBe(false);
    expect(menu.getAttribute("data-row-id")).toBe("10");
  });

  it("Enter on focused menu item should open delete dialog", () => {
    mountList();
    document.dispatchEvent(new KeyboardEvent("keydown", { key: "ArrowDown", bubbles: true }));
    document.dispatchEvent(new KeyboardEvent("keydown", { key: "Enter", bubbles: true }));
    document.dispatchEvent(new KeyboardEvent("keydown", { key: "Enter", bubbles: true }));

    expect(document.querySelector("[data-delete-dialog]").open).toBe(true);
  });

  it("active row should remain the same when opening menu", () => {
    mountList();
    const rows = Array.from(document.querySelectorAll("[data-row]"));

    document.dispatchEvent(new KeyboardEvent("keydown", { key: "ArrowDown", bubbles: true }));
    document.dispatchEvent(new KeyboardEvent("keydown", { key: "Enter", bubbles: true }));

    expect(rows[0].dataset.rowActive).toBe("true");
  });

  it("bootstrapCadUnicoList should close the floating menu on Escape and restore focus", () => {
    mountList();
    const row = document.querySelector("[data-row]");

    document.dispatchEvent(new KeyboardEvent("keydown", { key: "ArrowDown", bubbles: true }));
    document.dispatchEvent(new KeyboardEvent("keydown", { key: "Enter", bubbles: true }));
    document.dispatchEvent(new KeyboardEvent("keydown", { key: "Escape", bubbles: true }));

    expect(document.activeElement).toBe(row);
    expect(document.querySelector("[data-row-menu-popover]")).toBeNull();
  });

  it("popover should clamp position inside viewport bounds", () => {
    const result = clampPopoverPosition(
      { top: 900, left: 1400 },
      { width: 220, height: 120 },
      { width: 1280, height: 720 },
    );

    expect(result.left).toBeLessThanOrEqual(1280 - 220 - 16);
    expect(result.top).toBeLessThanOrEqual(720 - 120 - 16);
  });

  it("bootstrapCadUnicoList should use the persistent delete dialog after a list swap", () => {
    const root = mountList();
    window.htmx = { ajax: vi.fn() };

    document.dispatchEvent(new KeyboardEvent("keydown", { key: "ArrowDown", bubbles: true }));
    document.dispatchEvent(new KeyboardEvent("keydown", { key: "Enter", bubbles: true }));
    root.querySelector("[data-row-delete]").click();

    const dialog = root.querySelector("[data-delete-dialog]");
    expect(dialog.open).toBe(true);
  });

  it("bootstrapCadUnicoList should navigate to /cadunico/criar on Ctrl+N", () => {
    const root = mountList();
    const createLink = root.querySelector("[data-create-link]");
    const clickSpy = vi.fn((event) => event.preventDefault());
    createLink.addEventListener("click", clickSpy);

    document.dispatchEvent(new KeyboardEvent("keydown", { key: "n", ctrlKey: true, bubbles: true }));

    expect(clickSpy).toHaveBeenCalledTimes(1);
  });
});
