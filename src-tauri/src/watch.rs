use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::{path::PathBuf, sync::Mutex};
use tauri::{AppHandle, Emitter, State};

#[derive(Default)]
pub struct SummaryWatcher(pub Mutex<Option<RecommendedWatcher>>);

#[tauri::command]
pub fn watch_summary(
    app: AppHandle,
    watcher_state: State<'_, SummaryWatcher>,
    home: String,
) -> Result<(), String> {
    let home = PathBuf::from(home)
        .canonicalize()
        .map_err(|error| format!("home is unavailable: {error}"))?;
    let state_dir = home
        .join("state")
        .canonicalize()
        .map_err(|error| format!("state directory is unavailable: {error}"))?;
    if !state_dir.starts_with(&home) || !state_dir.is_dir() {
        return Err("state directory does not resolve under the selected home".into());
    }
    let app = app.clone();
    let mut watcher = notify::recommended_watcher(move |event: notify::Result<Event>| {
        if let Ok(event) = event {
            if event.paths.iter().any(|path| {
                path.file_name()
                    .is_some_and(|name| name == "home-summary.json")
            }) {
                let _ = app.emit("summary-changed", ());
            }
        }
    })
    .map_err(|error| format!("could not create summary watcher: {error}"))?;
    watcher
        .watch(&state_dir, RecursiveMode::NonRecursive)
        .map_err(|error| format!("could not watch state directory: {error}"))?;
    let mut active = watcher_state
        .0
        .lock()
        .map_err(|_| "summary watcher lock is unavailable")?;
    *active = Some(watcher);
    Ok(())
}
