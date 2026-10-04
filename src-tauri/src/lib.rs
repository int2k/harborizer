mod events;
mod resolver;
mod safe_file;
mod summary;
mod watch;

use serde::Serialize;
use std::{env, path::PathBuf};

#[derive(Serialize)]
struct RuntimeDefaults {
    home: Option<String>,
    resolver_root: Option<String>,
}

#[derive(Serialize)]
struct TaskDetail {
    current_state: resolver::ResolverDetail,
    last_event: Option<events::LastWakeEvent>,
}

#[tauri::command]
fn runtime_defaults() -> RuntimeDefaults {
    RuntimeDefaults {
        home: env::var("FM_HOME").ok(),
        resolver_root: env::var("FM_ROOT")
            .or_else(|_| env::var("FM_ROOT_OVERRIDE"))
            .ok(),
    }
}

#[tauri::command]
fn read_summary(home: String) -> summary::SummaryReadResult {
    summary::read_summary_file(&PathBuf::from(home))
}

#[tauri::command]
async fn task_detail(home: String, resolver_root: Option<String>, task_id: String) -> TaskDetail {
    let home_path = PathBuf::from(home);
    let root_path = resolver_root
        .filter(|root| !root.trim().is_empty())
        .or_else(|| env::var("FM_ROOT").ok())
        .or_else(|| env::var("FM_ROOT_OVERRIDE").ok())
        .unwrap_or_else(|| home_path.to_string_lossy().into_owned());
    let current_state =
        resolver::resolve_task(&home_path, &PathBuf::from(root_path), &task_id).await;
    let last_event = events::read_last_wake_event(&home_path, &task_id);
    TaskDetail {
        current_state,
        last_event,
    }
}

pub fn run() {
    tauri::Builder::default()
        .manage(watch::SummaryWatcher::default())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            runtime_defaults,
            read_summary,
            task_detail,
            watch::watch_summary
        ])
        .run(tauri::generate_context!())
        .expect("error while running Harborizer");
}
