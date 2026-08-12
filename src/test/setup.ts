import "@testing-library/jest-dom/vitest";
import { cleanup } from "@testing-library/react";
import { afterEach, vi } from "vitest";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

vi.mock("opensheetmusicdisplay", () => ({
  OpenSheetMusicDisplay: class {
    constructor(container: HTMLElement) {
      container.dataset.osmd = "true";
    }
    load() {
      return Promise.resolve();
    }
    render() {}
  },
}));

afterEach(() => {
  cleanup();
});
