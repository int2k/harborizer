use serde::Serialize;
use std::path::Path;

use crate::safe_file::read_limited_regular_file;

use crate::resolver::valid_task_id;

const MAX_EVENT_LOG_BYTES: u64 = 256 * 1024;

#[derive(Debug, Clone, Serialize)]
pub struct LastWakeEvent {
    pub line: String,
    pub epoch: Option<u64>,
}

pub fn read_last_wake_event(home: &Path, task_id: &str) -> Option<LastWakeEvent> {
    if !valid_task_id(task_id) {
        return None;
    }
    let state_dir = home.join("state");
    let state = state_dir.canonicalize().ok()?;
    let home = home.canonicalize().ok()?;
    if !state.starts_with(home) {
        return None;
    }
    let path = state.join(format!("{task_id}.status"));
    let contents = read_limited_regular_file(&path, MAX_EVENT_LOG_BYTES).ok()?;
    let line = contents
        .lines()
        .rev()
        .find(|line| !line.trim().is_empty())?
        .trim()
        .to_owned();
    let epoch = line.find("[at=").and_then(|start| {
        let rest = &line[start + 4..];
        let end = rest.find(']')?;
        rest[..end].parse::<u64>().ok()
    });
    Some(LastWakeEvent { line, epoch })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn exposes_the_last_wake_line_as_history_without_mapping_its_verb_to_state() {
        let directory = tempfile::tempdir().unwrap();
        let state = directory.path().join("state");
        fs::create_dir(&state).unwrap();
        fs::write(
            state.join("sail-1.status"),
            "working [at=10]: old\ndone [at=20]: completed\n",
        )
        .unwrap();
        let event = read_last_wake_event(directory.path(), "sail-1").unwrap();
        assert_eq!(event.line, "done [at=20]: completed");
        assert_eq!(event.epoch, Some(20));
    }

    #[test]
    fn refuses_task_ids_that_are_not_safe_file_names() {
        assert!(read_last_wake_event(Path::new("/"), "../etc/passwd").is_none());
    }
}
