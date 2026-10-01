# Harborizer

Harborizer is a read-only desktop harbor chart for one local Firstmate home. Projects become ports; work becomes vessels arranged by stage. A sortable list and detail panel keep the fleet understandable without relying on the illustration.

## Safety and data

- Reads only `<Firstmate home>/state/home-summary.json`, and validates the `fm-secondmate-home-summary.v1` and `fm-captain-hold-buckets.v1` contracts before displaying it.
- Watches the `state` directory for atomic summary replacement and polls every four seconds as a fallback. Snapshots older than 90 seconds are marked stale.
- Runs `bin/fm-crew-state.sh <task-id>` only for the selected task, with a 12-second timeout. Set `FM_ROOT` if the Firstmate code root is separate from the home folder. The configured code root must be trusted: Harborizer executes its resolver script as the current user only when a vessel is selected.
- Shows the most recent `<task-id>.status` line as historical wake-event context only; it is never interpreted as current state.
- Writes no fleet state, never acknowledges wake events, and has no terminal controls, queue actions, local HTTP server, telemetry, or remote-home support. Packaged Tauri uses the native webview; the Vite server is for development only.
- Unrecognized schema versions and current states remain unsupported/unknown rather than being guessed.

## Prerequisites

- Node.js 22 or newer and npm
- Rust stable with Cargo
- Tauri 2 Linux development libraries on Linux

On Ubuntu 24.04 / Debian derivatives, install the Linux libraries with:

```sh
sudo apt-get update
sudo apt-get install build-essential curl file libayatana-appindicator3-dev libssl-dev \
  libwebkit2gtk-4.1-dev libxdo-dev librsvg2-dev patchelf
```

## Setup

```sh
npm ci
```

If Firstmate's home and code root are known before launch, configure them in the environment:

```sh
export FM_HOME="/path/to/firstmate-home"
export FM_ROOT="/path/to/firstmate-code"
```

The app also lets you choose the home folder. The resolver root is optional when the resolver script is inside the home folder. Harborizer remembers these paths in its own app storage, outside the monitored home.

## Development

Run the UI in a browser with synthetic, hand-authored fixtures:

```sh
npm run dev
```

The preview switcher includes **Busy fleet**, **Empty harbor**, **Stale summary**, **Invalid summary**, and **Captain’s call**. Fixture names, paths, and pull-request URLs are synthetic; example links use `example.com`.

Run the native desktop app against the same fixtures, or select **Live local home** in the preview switcher:

```sh
npm run tauri dev
```

The live mode reads one selected home. Selecting a vessel runs Firstmate's read-only current-state resolver for that vessel and displays any available PR link for opening in the system browser.

## Checks and build

```sh
npm run check
npm test
cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check
cargo test --manifest-path src-tauri/Cargo.toml
npm run build
```

Build a native Tauri bundle with:

```sh
npm run tauri build
```

GitHub Actions runs the frontend checks, Rust formatting/tests, and frontend production build for pushes to `main` and pull requests.

## License

MIT. See [LICENSE](LICENSE).
