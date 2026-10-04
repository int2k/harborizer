# Harborizer v1 Implementation Plan

> **For agentic workers:** This task is executed inline in the isolated `fm/harborizer-v1` worktree. Steps use checkbox syntax for tracking.

**Goal:** Build a read-only Tauri desktop app that visualizes one local Firstmate home as both a nautical harbor chart and a sortable list.

**Architecture:** Rust validates `fm-secondmate-home-summary.v1`, watches its state directory, and runs the current-state resolver only for the selected task. A TypeScript adapter normalizes summary records and freshness; Svelte renders the chart/list/detail views and offers synthetic fixtures in browser development.

**Tech Stack:** Tauri 2, Rust, Svelte 5, TypeScript, Vite, Vitest, serde, notify.

**Spec:** `docs/superpowers/specs/2026-10-01-harborizer-v1-design.md`

## Global Constraints

- Read only the chosen Firstmate home; persist app preferences outside that home.
- Supported schema: `fm-secondmate-home-summary.v1`; classifier schema: `fm-captain-hold-buckets.v1`.
- Never infer current state from `state/<id>.status`; use the resolver for selected task detail.
- Do not create an HTTP server, touch worker terminals directly, or consume/acknowledge queues.
- Public fixtures and docs contain only synthetic names, paths, and `example.com` URLs.
- Respect keyboard navigation, light/dark themes, reduced motion, and explicit unknown states.

---

### Task 1: Tauri/Svelte foundation and schema adapter

**Files:** `package.json`, `package-lock.json`, `vite.config.js`, `svelte.config.js`, `tsconfig.json`, `src/app.html`, `src/routes/+layout.ts`, `src/routes/+layout.svelte`, `src/routes/+page.svelte`, `src/app.css`, `src/lib/model.ts`, `src/lib/stages.ts`, `src/lib/freshness.ts`, `src/lib/fixtures.ts`, `src/lib/stages.test.ts`, `src/lib/freshness.test.ts`, `src-tauri/Cargo.toml`, `src-tauri/src/lib.rs`, `src-tauri/src/summary.rs`, `src-tauri/tauri.conf.json`, `src-tauri/capabilities/default.json`.

**Interfaces:** `mapSummaryToVessels(summary): Vessel[]`; `classifyFreshness(summary, now, staleAfterSeconds): Freshness`; fixtures implement the documented summary v1 shape.

- [x] Scaffold Tauri 2 + Svelte 5 + TypeScript + Vite, with Vitest and Rust dependencies.
- [x] Add failing TS tests for known-stage mapping, unknown state preservation, held/queued overlap, and freshness states; verify red.
- [x] Implement typed normalized models, adapter, freshness classifier, and synthetic fixtures; verify green.
- [x] Add failing Rust tests for valid v1, unsupported schema, and malformed/missing required fields; verify red.
- [x] Implement strict summary parsing/validation and Tauri read-summary command.

### Task 2: Safe local-home connection and resolver/watch commands

**Files:** `src-tauri/src/resolver.rs`, `src-tauri/src/events.rs`, `src-tauri/src/watch.rs`, `src-tauri/src/lib.rs`, `src/lib/runtime.ts`.

**Interfaces:** Tauri exposes `read_summary(home)`, `runtime_defaults()`, `watch_summary(home)`, and `task_detail(home, resolver_root, task_id)`; resolver failures return unknown.

- [x] Add Rust tests for safe task-id validation, resolver output/PR parsing, timeout, nonzero exit, and oversized output; verify expected red.
- [x] Implement bounded resolver invocation with `FM_HOME` and `FM_ROOT_OVERRIDE`, and map all errors to unknown detail.
- [x] Read the final status line only as historical event detail; never infer current state from it.
- [x] Watch the state directory for `home-summary.json` replacement and poll every four seconds in the UI as fallback.
- [x] Add native folder picker and persist the selected home and optional code root in app local storage, never in the monitored home.

### Task 3: Harbor/list/detail UI and accessibility

**Files:** `src/routes/+page.svelte`, `src/lib/components/HarborChart.svelte`, `src/lib/components/VesselList.svelte`, `src/lib/components/DetailPanel.svelte`, `src/lib/components/ConnectionSettings.svelte`, `src/app.css`, `src/lib/fixtures.ts`.

**Interfaces:** Components consume normalized `Vessel[]`, `Freshness`, selection, and callbacks; all views expose the same selection and data.

- [x] Render busy and empty fixtures with a harbor chart, ports, vessels, decision buoys, scout report markers, and a sortable list toggle.
- [x] Add task detail with project/kind/stage/event time/decision, selected-task resolver result, and validated external PR link.
- [x] Add fixture selector, last-updated/stale/invalid/unsupported/omitted-row disclosures, light/dark theme, keyboard controls, and reduced-motion support.
- [ ] Visually inspect the browser preview and fix contrast/focus/layout issues. The Chrome bridge failed twice in this worker environment; no alternate browser was used.

### Task 4: Public documentation and final verification

**Files:** `README.md`, `LICENSE`, `.github/workflows/ci.yml`, all application files.

- [x] Document prerequisites, setup, browser/Tauri dev, fixture switcher, Linux dependencies, and build commands.
- [x] Add MIT license and confirm no real home paths or fleet data are present.
- [x] Add CI for Linux dependencies, Rust formatting/tests, frontend checks/tests/build, with pinned actions.
- [ ] Run `cargo fmt --check`, `cargo test`, `npm run check`, `npm test`, `npm run build`, dependency audit, and diff review; fix failures and repeat.
- [ ] Inspect final diff and commit the completed v1 branch.
