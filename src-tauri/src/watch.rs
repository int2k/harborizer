use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::{path::PathBuf, sync::Mutex};
use tauri::{AppHandle, Emitter, State};

#[derive(Default)]
pub struct SummaryWatcher(pub Mutex<Option<RecommendedWatcher>>);

/// Reads open the summary file and raise Access events; only content changes count.
fn is_summary_change(event: &Event) -> bool {
    matches!(
        event.kind,
        EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_)
    ) && event.paths.iter().any(|path| {
        path.file_name()
            .is_some_and(|name| name == "home-summary.json")
    })
}

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
            if is_summary_change(&event) {
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

#[cfg(test)]
mod tests {
    use super::*;
    use notify::event::{AccessKind, AccessMode, CreateKind, ModifyKind, RemoveKind};
    use std::{fs, sync::mpsc, time::Duration};

    fn event(kind: EventKind, name: &str) -> Event {
        Event::new(kind).add_path(PathBuf::from("/home/state").join(name))
    }

    #[test]
    fn content_changes_to_the_summary_count() {
        for kind in [
            EventKind::Create(CreateKind::File),
            EventKind::Modify(ModifyKind::Any),
            EventKind::Remove(RemoveKind::File),
        ] {
            assert!(is_summary_change(&event(kind, "home-summary.json")));
        }
    }

    #[test]
    fn access_events_and_other_files_are_ignored() {
        let open = EventKind::Access(AccessKind::Open(AccessMode::Any));
        assert!(!is_summary_change(&event(open, "home-summary.json")));
        let modify = EventKind::Modify(ModifyKind::Any);
        assert!(!is_summary_change(&event(modify, "other.json")));
    }

    #[test]
    fn reading_the_summary_does_not_retrigger_the_watcher() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("home-summary.json");
        fs::write(&path, "{}").unwrap();
        let (tx, rx) = mpsc::channel();
        let mut watcher = notify::recommended_watcher(move |event: notify::Result<Event>| {
            if event.is_ok_and(|event| is_summary_change(&event)) {
                let _ = tx.send(());
            }
        })
        .unwrap();
        watcher
            .watch(dir.path(), RecursiveMode::NonRecursive)
            .unwrap();
        std::thread::sleep(Duration::from_millis(200));

        for _ in 0..3 {
            fs::read(&path).unwrap();
        }
        assert!(rx.recv_timeout(Duration::from_millis(500)).is_err());

        fs::write(&path, "{\"a\":1}").unwrap();
        assert!(rx.recv_timeout(Duration::from_secs(3)).is_ok());
    }
}
