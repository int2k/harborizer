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

**Files:** `package.json`, `package-lock.json`, `vite.config.ts`, `svelte.config.js`, `tsconfig.json`, `index.html`, `src/main.ts`, `src/App.svelte`, `src/app.css`, `src/lib/model.ts`, `src/lib/stages.ts`, `src/lib/freshness.ts`, `src/lib/fixtures.ts`, `src/lib/stages.test.ts`, `src/lib/freshness.test.ts`, `src-tauri/Cargo.toml`, `src-tauri/src/lib.rs`, `src-tauri/tauri.conf.json`, `.github/workflows/ci.yml`.

**Interfaces:** `mapSummaryToVessels(summary): Vessel[]`; `classifyFreshness(summary, now, staleAfterSeconds): Freshness`; fixtures implement the documented summary v1 shape.

- [ ] Scaffold Tauri 2 + Svelte 5 + TypeScript + Vite, with Vitest, Rust tests, format/typecheck/build scripts, and GitHub Actions.
- [ ] Add failing TS tests for each recognized stage, unknown state preservation, and fresh/stale/invalid/unsupported freshness states; verify red.
- [ ] Implement typed normalized models, adapter, freshness classifier, and hand-authored synthetic fixtures; verify green.
- [ ] Add failing Rust tests for valid v1, wrong schema, and malformed/missing required fields; verify red.
- [ ] Implement strict summary parsing/validation and Tauri read-summary command; verify Rust tests green.

### Task 2: Safe local-home connection and resolver/watch commands

**Files:** `src-tauri/src/home.rs`, `src-tauri/src/resolver.rs`, `src-tauri/src/watch.rs`, `src-tauri/src/lib.rs`, `src-tauri/Cargo.toml`, `src/lib/runtime.ts`, `src/lib/runtime.test.ts`.

**Interfaces:** Rust exposes `read_summary(home)`, `choose_home()`, `start_summary_watch(home)`, and `resolve_task(home, root, task_id)`; resolver failures return unknown. Frontend runtime selects Tauri commands or fixtures.

- [ ] Add Rust tests for safe task-id validation, resolver output parsing, timeout, nonzero exit, and missing executable; verify expected red.
- [ ] Implement bounded resolver invocation using `FM_ROOT` or the selected home as resolver root, set `FM_HOME`, and map all errors to unknown detail.
- [ ] Watch the state directory for `home-summary.json` replacement and emit a change event; poll every four seconds in the UI as fallback.
- [ ] Add folder picker and persist the selected path in app local storage, never in the monitored home.

### Task 3: Harbor/list/detail UI and accessibility

**Files:** `src/App.svelte`, `src/lib/components/HarborChart.svelte`, `src/lib/components/VesselList.svelte`, `src/lib/components/DetailPanel.svelte`, `src/lib/components/ConnectionSettings.svelte`, `src/app.css`, `src/lib/fixtures.ts`.

**Interfaces:** Components consume normalized `Vessel[]`, `Freshness`, selection, and callbacks; all views expose the same selection and data.

- [ ] Render the busy and empty fixtures with a harbor chart, ports, vessels, decision buoys, scout report markers, and a sortable list toggle.
- [ ] Add task detail with project/kind/stage/event time/decision, selected-task resolver result, and validated external PR link.
- [ ] Add fixture selector, last-updated/stale/invalid/unsupported/omitted-row disclosures, light/dark theme, keyboard controls, and reduced-motion support.
- [ ] Run the app in browser dev mode with every fixture and fix layout/contrast/focus issues.

### Task 4: Public documentation and final verification

**Files:** `README.md`, `LICENSE`, `.github/workflows/ci.yml`, all application files.

- [ ] Document prerequisites, setup, browser/Tauri dev, fixture switcher, Linux dependencies, and build commands.
- [ ] Add MIT license and confirm no real home paths or fleet data are present.
- [ ] Run `cargo fmt --check`, `cargo test`, `npm run check`, `npm test`, `npm run build`, and CI-equivalent checks; fix failures and repeat.
- [ ] Inspect final diff and commit the completed v1 branch.
