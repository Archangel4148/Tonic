# Tonic documentation

Tonic is a local-first musician's songbook: import, edit, transpose, organize, and perform chord charts offline.

This folder is the project documentation. The product requirements live in [`spec.md`](./spec.md).

## Start here

| Document                                      | What it covers                                       |
| --------------------------------------------- | ---------------------------------------------------- |
| [Development](./development.md)               | Install, run, test, lint, and build                  |
| [Technology choices](./technology-choices.md) | Why Tauri, Rust, React, and the supporting tools     |
| [Architecture](./architecture.md)             | Layer boundaries and dependency rules                |
| [Project structure](./project-structure.md)   | Where code and docs live                             |
| [State management](./state-management.md)     | What owns authoritative vs derived state             |
| [Testing](./testing.md)                       | Rust and UI test setup                               |
| [Assumptions](./assumptions.md)               | Decisions made where the spec was open               |
| [Phase 1 report](./phases/phase-1.md)         | Foundation checkpoint, limitations, and review notes |

## Source of truth

[`spec.md`](./spec.md) is the current product and engineering specification. Implement only the requested phase, then stop at that phase's review checkpoint.
