// @vitest-environment jsdom
import { beforeEach, describe, expect, it, vi } from "vitest";
import { bootstrapCadUnicoList, nextRowIndex } from "./cadunico-list.js";

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
              <tr data-row data-row-id="10" tabindex="-1">
                <td>Registro 1</td>
                <td>
                  <div data-row-menu hidden>
                    <button type="button" disabled aria-disabled="true">Editar (em breve)</button>
                    <button type="button" data-row-delete>Excluir</button>
                  </div>
                </td>
              </tr>
              <tr data-row data-row-id="11" tabindex="-1">
                <td>Registro 2</td>
                <td>
                  <div data-row-menu hidden>
                    <button type="button" disabled aria-disabled="true">Editar (em breve)</button>
                    <button type="button" data-row-delete>Excluir</button>
                  </div>
                </td>
              </tr>
            </tbody>
          </table>
        </div>
        <footer>
          <button type="button" data-page-prev>Anterior</button>
          <button type="button" data-page-next>Proxima</button>
        </footer>
        <dialog data-delete-dialog>
          <form method="dialog">
            <button type="submit" value="cancel">Cancelar</button>
            <button type="submit" data-confirm-delete>Excluir</button>
          </form>
        </dialog>
      </section>
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

  it("bootstrapCadUnicoList should open the row menu on Enter and restore focus on Escape", () => {
    mountList();
    const row = document.querySelector("[data-row]");
    const menu = row.querySelector("[data-row-menu]");
    const firstAction = menu.querySelector("[data-row-delete]");

    document.dispatchEvent(new KeyboardEvent("keydown", { key: "ArrowDown", bubbles: true }));
    document.dispatchEvent(new KeyboardEvent("keydown", { key: "Enter", bubbles: true }));

    expect(menu.hidden).toBe(false);
    expect(document.activeElement).toBe(firstAction);

    document.dispatchEvent(new KeyboardEvent("keydown", { key: "Escape", bubbles: true }));

    expect(menu.hidden).toBe(true);
    expect(document.activeElement).toBe(row);
  });

  it("bootstrapCadUnicoList should navigate to /cadunico/criar on Ctrl+N", () => {
    mountList();
    const createLink = document.querySelector("[data-create-link]");
    const clickSpy = vi.fn((event) => event.preventDefault());
    createLink.addEventListener("click", clickSpy);

    document.dispatchEvent(new KeyboardEvent("keydown", { key: "n", ctrlKey: true, bubbles: true }));

    expect(clickSpy).toHaveBeenCalledTimes(1);
  });
});
