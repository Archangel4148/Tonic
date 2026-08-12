import { render, screen } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import { beforeEach, describe, expect, it, vi } from "vitest";
import App from "./App";

const mockedInvoke = vi.mocked(invoke);

describe("App", () => {
  beforeEach(() => {
    mockedInvoke.mockReset();
  });

  it("renders the Tonic shell after the engine responds", async () => {
    mockedInvoke.mockResolvedValue({
      name: "Tonic",
      version: "0.1.0",
      phase: 2,
      domainEngine: "tonic-domain",
      domainVersion: "0.1.0",
      persistenceHealthy: true,
    });

    render(<App />);

    expect(
      await screen.findByRole("heading", { name: "Tonic" }),
    ).toBeInTheDocument();
    expect(
      screen.getByText(/musician's digital songbook/i),
    ).toBeInTheDocument();
    expect(screen.getByText(/tonic-domain v0\.1\.0/i)).toBeInTheDocument();
    expect(screen.getByText(/in-memory stub healthy/i)).toBeInTheDocument();
  });

  it("surfaces an error when the engine is unavailable", async () => {
    mockedInvoke.mockRejectedValue(new Error("IPC unavailable"));

    render(<App />);

    expect(await screen.findByRole("alert")).toHaveTextContent(
      "IPC unavailable",
    );
  });
});
