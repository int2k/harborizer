use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{path::Path, path::PathBuf};

use crate::safe_file::read_limited_regular_file;

pub const SUMMARY_SCHEMA: &str = "fm-secondmate-home-summary.v1";
pub const HOLD_SCHEMA: &str = "fm-captain-hold-buckets.v1";
const MAX_SUMMARY_BYTES: u64 = 2 * 1024 * 1024;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HomeSummary {
    pub schema: String,
    pub hold_classifier_schema: String,
    pub generated: String,
    pub generated_epoch: u64,
    pub home: String,
    pub valid: bool,
    pub reason: Option<String>,
    pub invalidity: Value,
    pub state: String,
    pub active_children: Vec<Value>,
    pub decisions_open: Vec<Value>,
    pub holds: Vec<Value>,
    pub queued: Vec<Value>,
    pub landed: Vec<Value>,
    pub endpoints: Vec<Value>,
    pub counts: Value,
    pub omitted: Vec<Value>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SummaryError {
    Invalid(String),
    Unsupported {
        schema: Option<String>,
        hold_schema: Option<String>,
    },
}

pub fn parse_summary(input: &str) -> Result<HomeSummary, SummaryError> {
    let value: Value = serde_json::from_str(input)
        .map_err(|error| SummaryError::Invalid(format!("summary JSON is malformed: {error}")))?;
    let schema = value
        .get("schema")
        .and_then(Value::as_str)
        .map(str::to_owned);
    let hold_schema = value
        .get("hold_classifier_schema")
        .and_then(Value::as_str)
        .map(str::to_owned);

    match schema.as_deref() {
        None => {
            return Err(SummaryError::Invalid(
                "summary is missing its schema declaration".into(),
            ))
        }
        Some(found) if found != SUMMARY_SCHEMA => {
            return Err(SummaryError::Unsupported {
                schema,
                hold_schema,
            })
        }
        _ => {}
    }
    match hold_schema.as_deref() {
        None => {
            return Err(SummaryError::Invalid(
                "summary is missing its hold classifier schema".into(),
            ))
        }
        Some(found) if found != HOLD_SCHEMA => {
            return Err(SummaryError::Unsupported {
                schema,
                hold_schema,
            })
        }
        _ => {}
    }

    let summary: HomeSummary = serde_json::from_value(value)
        .map_err(|error| SummaryError::Invalid(format!("summary fields are invalid: {error}")))?;
    if summary.generated.trim().is_empty()
        || summary.home.trim().is_empty()
        || summary.state.trim().is_empty()
    {
        return Err(SummaryError::Invalid(
            "generated, home, and state must be non-empty strings".into(),
        ));
    }
    if !summary.invalidity.is_object()
        || !summary.invalidity.get("ids").is_some_and(Value::is_array)
    {
        return Err(SummaryError::Invalid(
            "invalidity must be an object with an ids array".into(),
        ));
    }
    let counts = summary
        .counts
        .as_object()
        .ok_or_else(|| SummaryError::Invalid("counts must be an object".into()))?;
    for key in [
        "active_children",
        "decisions_open",
        "holds",
        "queued",
        "landed",
        "endpoints",
    ] {
        if !counts.get(key).and_then(Value::as_u64).is_some() {
            return Err(SummaryError::Invalid(format!(
                "counts.{key} must be a non-negative integer"
            )));
        }
    }
    for key in [
        "active_children",
        "decisions_open",
        "holds",
        "queued",
        "landed",
        "endpoints",
    ] {
        let rows = match key {
            "active_children" => &summary.active_children,
            "decisions_open" => &summary.decisions_open,
            "holds" => &summary.holds,
            "queued" => &summary.queued,
            "landed" => &summary.landed,
            _ => &summary.endpoints,
        };
        if rows
            .iter()
            .any(|row| !row.is_object() || !row.get("id").is_some_and(Value::is_string))
        {
            return Err(SummaryError::Invalid(format!(
                "{key} rows must be objects with string ids"
            )));
        }
    }
    for item in &summary.omitted {
        if !item.get("surface").is_some_and(Value::is_string)
            || item.get("count").and_then(Value::as_u64).is_none()
        {
            return Err(SummaryError::Invalid(
                "omitted entries require a surface and non-negative count".into(),
            ));
        }
    }
    Ok(summary)
}

pub fn read_summary_file(home: &Path) -> SummaryReadResult {
    let home = match home.canonicalize() {
        Ok(path) if path.is_dir() => path,
        Ok(_) => {
            return SummaryReadResult::Invalid {
                reason: "selected home is not a directory".into(),
            }
        }
        Err(error) => {
            return SummaryReadResult::Missing {
                reason: format!("selected home is unavailable: {error}"),
            }
        }
    };
    let state_dir = home.join("state");
    let canonical_state = match state_dir.canonicalize() {
        Ok(path) if path.starts_with(&home) && path.is_dir() => path,
        Ok(_) => {
            return SummaryReadResult::Invalid {
                reason: "state directory resolves outside the selected home".into(),
            }
        }
        Err(error) => {
            return SummaryReadResult::Missing {
                reason: format!("state directory is unavailable: {error}"),
            }
        }
    };
    let summary_path: PathBuf = canonical_state.join("home-summary.json");
    let input = match read_limited_regular_file(&summary_path, MAX_SUMMARY_BYTES) {
        Ok(input) => input,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return SummaryReadResult::Missing {
                reason: "state/home-summary.json was not found".into(),
            }
        }
        Err(error) => {
            return SummaryReadResult::Invalid {
                reason: format!("could not safely read summary: {error}"),
            }
        }
    };
    match parse_summary(&input) {
        Ok(summary) => SummaryReadResult::Ready { summary },
        Err(SummaryError::Invalid(reason)) => SummaryReadResult::Invalid { reason },
        Err(SummaryError::Unsupported {
            schema,
            hold_schema,
        }) => SummaryReadResult::Unsupported {
            schema,
            hold_schema,
        },
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum SummaryReadResult {
    Ready {
        summary: HomeSummary,
    },
    Invalid {
        reason: String,
    },
    Unsupported {
        schema: Option<String>,
        hold_schema: Option<String>,
    },
    Missing {
        reason: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::fs;

    fn valid_summary() -> String {
        json!({
            "schema": SUMMARY_SCHEMA,
            "hold_classifier_schema": HOLD_SCHEMA,
            "generated": "2026-09-30T12:00:00Z",
            "generated_epoch": 1790770,
            "home": "/example/firstmate",
            "valid": true,
            "reason": null,
            "invalidity": {"kind": null, "ids": []},
            "state": "active_child_work",
            "active_children": [{"id": "sail-1", "kind": "ship", "state": "working"}],
            "decisions_open": [],
            "holds": [],
            "queued": [],
            "landed": [],
            "endpoints": [],
            "counts": {"active_children": 1, "decisions_open": 0, "holds": 0, "queued": 0, "landed": 0, "endpoints": 1},
            "omitted": []
        }).to_string()
    }

    #[test]
    fn parses_a_complete_v1_summary() {
        let parsed = parse_summary(&valid_summary()).expect("valid v1 should parse");
        assert_eq!(parsed.schema, SUMMARY_SCHEMA);
        assert_eq!(parsed.active_children.len(), 1);
    }

    #[test]
    fn rejects_a_future_schema_as_unsupported() {
        let raw = valid_summary().replace(SUMMARY_SCHEMA, "fm-secondmate-home-summary.v9");
        assert!(matches!(
            parse_summary(&raw),
            Err(SummaryError::Unsupported { .. })
        ));
    }

    #[test]
    fn rejects_malformed_or_incomplete_json_as_invalid() {
        assert!(matches!(parse_summary("{"), Err(SummaryError::Invalid(_))));
        let incomplete = json!({"schema": SUMMARY_SCHEMA}).to_string();
        assert!(matches!(
            parse_summary(&incomplete),
            Err(SummaryError::Invalid(_))
        ));
    }

    #[test]
    fn reads_only_a_regular_summary_under_the_selected_home() {
        let directory = tempfile::tempdir().unwrap();
        let state = directory.path().join("state");
        fs::create_dir(&state).unwrap();
        fs::write(state.join("home-summary.json"), valid_summary()).unwrap();
        assert!(matches!(
            read_summary_file(directory.path()),
            SummaryReadResult::Ready { .. }
        ));
    }
}
