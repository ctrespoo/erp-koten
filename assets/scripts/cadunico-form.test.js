// @vitest-environment jsdom
import { describe, expect, it } from "vitest";
import {
  activateNextField,
  activatePreviousField,
  nextTabId,
  normalizeCep,
  normalizeCpfCnpj,
} from "./cadunico-form.js";

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
});
