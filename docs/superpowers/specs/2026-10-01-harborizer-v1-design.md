# Harborizer v1 Design

## Goal and boundaries

Harborizer is a local, read-only desktop view of one Firstmate home. It presents the same fleet as a nautical SVG harbor chart and as a conventional sortable list, with a detail pane for the selected work item. It does not control workers, alter Firstmate files, read event logs as current state, consume queues, expose an HTTP server, or add remote-home, history, notification, or telemetry features.

## Architecture

The desktop shell is Tauri 2. Rust owns filesystem access, strict summary validation, directory watching, and bounded calls to Firstmate's `bin/fm-crew-state.sh`. The Svelte 5 + TypeScript frontend consumes an adapter model, not raw JSON; the adapter maps only known states and keeps unknown states explicit. A development-only fixture adapter provides busy, empty, stale, invalid, and decision-waiting scenes in browser mode.

The app reads `<home>/state/home-summary.json`, whose only supported schema is `fm-secondmate-home-summary.v1` with `hold_classifier_schema` equal to `fm-captain-hold-buckets.v1`. It watches the containing state directory so atomic replacement is observed, and polls every few seconds as a fallback. Unsupported versions, invalid summaries, stale data, and producer-omitted rows are disclosed rather than hidden. `state/<id>.status` is historical wake-event data and is never used as current state.

For a selected task, Rust invokes the Firstmate resolver with a validated task id, a bounded timeout, and the selected home environment. Resolver failures become an explicit unknown detail. PR links are opened only through the system browser after validating an HTTP(S) URL. Home selection is persisted locally by the app; no preference is written into the monitored home.

## Interface

The primary chart gives each project a port, places work at a known stage (setting out, under way, inspection, quay, arrived), shows scouts with a report marker, and makes captain decisions conspicuous as flagged buoys. Unmappable states remain visibly unknown. The list view exposes the same normalized records without relying on chart artwork. The detail pane shows project, kind, stage, last event/time, decision text, and an external PR link when present.

The visual direction is a restrained Swiss-style operations console translated into nautical language: paper/ink surfaces, deep-water blues, sea-glass teal, and high-contrast buoy amber. SVG art is authored in-repo. Light/dark themes, visible focus, keyboard selection, text alternatives, and reduced-motion behavior are required.

## Verification

Rust tests cover the schema contract, invalid/unsupported input, and resolver success/failure/timeout handling. TypeScript tests cover row-to-stage mapping and freshness classification. CI runs Rust formatting/tests and frontend typecheck/tests/build on pushes to `main` and pull requests, installing Linux WebKitGTK development dependencies. Synthetic fixtures use only fictional identifiers, paths, and `example.com` links.
