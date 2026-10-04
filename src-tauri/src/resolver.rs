use serde::Serialize;
use std::{path::Path, process::Stdio, time::Duration};
use tokio::{io::AsyncReadExt, process::Command, time::timeout};

const MAX_RESOLVER_OUTPUT_BYTES: usize = 16 * 1024;

#[derive(Debug, Clone, Serialize)]
pub struct ResolverDetail {
    pub state: String,
    pub source: String,
    pub detail: Option<String>,
    pub pr_url: Option<String>,
    pub unknown: bool,
}

impl ResolverDetail {
    pub fn unknown(reason: impl Into<String>) -> Self {
        Self {
            state: "unknown".into(),
            source: "none".into(),
            detail: Some(reason.into()),
            pr_url: None,
            unknown: true,
        }
    }
}

pub fn parse_resolver_line(line: &str) -> Option<ResolverDetail> {
    let mut parts = line.trim().splitn(3, " · ");
    let state = parts.next()?.strip_prefix("state: ")?.trim();
    let source = parts.next()?.strip_prefix("source: ")?.trim();
    if state.is_empty() || source.is_empty() {
        return None;
    }
    let known = [
        "working", "parked", "done", "blocked", "paused", "failed", "unknown",
    ];
    if !known.contains(&state) {
        return None;
    }
    let detail = parts
        .next()
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .map(str::to_owned);
    let pr_url = detail.as_deref().and_then(extract_http_url);
    Some(ResolverDetail {
        state: state.into(),
        source: source.into(),
        detail,
        pr_url,
        unknown: state == "unknown",
    })
}

fn extract_http_url(detail: &str) -> Option<String> {
    detail.split_whitespace().find_map(|part| {
        let candidate = part.trim_end_matches(['.', ',', ';', ')', ']']);
        let url = url::Url::parse(candidate).ok()?;
        (matches!(url.scheme(), "http" | "https")
            && url.username().is_empty()
            && url.password().is_none())
        .then(|| url.to_string())
    })
}

#[cfg(test)]
pub async fn run_resolver_command(
    program: &Path,
    args: &[String],
    command_timeout: Duration,
) -> ResolverDetail {
    run_command(program, args, command_timeout, None).await
}

async fn run_command(
    program: &Path,
    args: &[String],
    command_timeout: Duration,
    context: Option<(&Path, &Path)>,
) -> ResolverDetail {
    let mut command = Command::new(program);
    command
        .args(args)
        .kill_on_drop(true)
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    if let Some((home, root)) = context {
        command.env("FM_HOME", home).env("FM_ROOT_OVERRIDE", root);
    }
    let output = match timeout(command_timeout, async {
        let mut child = command
            .spawn()
            .map_err(|error| format!("could not run resolver: {error}"))?;
        let mut stdout = child
            .stdout
            .take()
            .ok_or_else(|| "resolver stdout was unavailable".to_owned())?;
        let mut output = Vec::with_capacity(MAX_RESOLVER_OUTPUT_BYTES);
        (&mut stdout)
            .take((MAX_RESOLVER_OUTPUT_BYTES + 1) as u64)
            .read_to_end(&mut output)
            .await
            .map_err(|error| format!("could not read resolver output: {error}"))?;
        if output.len() > MAX_RESOLVER_OUTPUT_BYTES {
            let _ = child.kill().await;
            return Err(
                "current-state resolver output exceeded the 16 KiB safety limit".to_owned(),
            );
        }
        let status = child
            .wait()
            .await
            .map_err(|error| format!("could not wait for resolver: {error}"))?;
        if !status.success() {
            return Err("current-state resolver exited unsuccessfully".to_owned());
        }
        Ok::<Vec<u8>, String>(output)
    })
    .await
    {
        Ok(Ok(output)) => output,
        Ok(Err(reason)) => return ResolverDetail::unknown(reason),
        Err(_) => return ResolverDetail::unknown("current-state resolver timed out"),
    };
    let stdout = String::from_utf8_lossy(&output);
    stdout
        .lines()
        .find_map(parse_resolver_line)
        .unwrap_or_else(|| {
            ResolverDetail::unknown("current-state resolver returned an unreadable result")
        })
}

pub fn valid_task_id(task_id: &str) -> bool {
    let mut chars = task_id.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    task_id.len() <= 120
        && first.is_ascii_alphanumeric()
        && chars.all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '.' | '_' | '-'))
}

pub async fn resolve_task(home: &Path, root: &Path, task_id: &str) -> ResolverDetail {
    if !valid_task_id(task_id) {
        return ResolverDetail::unknown("task id has an invalid format");
    }
    let resolver = root.join("bin").join("fm-crew-state.sh");
    if !resolver.is_file() {
        return ResolverDetail::unknown(
            "Firstmate resolver was not found under the configured code root",
        );
    }
    run_command(
        &resolver,
        &[task_id.to_owned()],
        Duration::from_secs(12),
        Some((home, root)),
    )
    .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_the_resolvers_stable_current_state_line() {
        let detail =
            parse_resolver_line("state: working · source: pane · charting the bay").unwrap();
        assert_eq!(detail.state, "working");
        assert_eq!(detail.source, "pane");
        assert_eq!(detail.detail.as_deref(), Some("charting the bay"));
        assert_eq!(detail.pr_url, None);
        assert!(!detail.unknown);
    }

    #[test]
    fn exposes_a_validated_pull_request_url_from_resolver_detail() {
        let detail = parse_resolver_line("state: done · source: run-step · checks green: PR held for merge: https://example.com/demo/repo/pull/4").unwrap();
        assert_eq!(
            detail.pr_url.as_deref(),
            Some("https://example.com/demo/repo/pull/4")
        );
    }

    #[test]
    fn refuses_resolver_urls_with_embedded_credentials() {
        let detail = parse_resolver_line("state: done · source: run-step · run passed: PR open: https://captain:secret@example.com/demo/pull/1").unwrap();
        assert_eq!(detail.pr_url, None);
    }

    #[test]
    fn rejects_task_ids_that_could_escape_the_state_directory() {
        assert!(!valid_task_id("../secret"));
        assert!(!valid_task_id(""));
        assert!(valid_task_id("sail-17.a_b"));
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn timeout_becomes_unknown() {
        let detail = run_resolver_command(
            Path::new("/bin/sh"),
            &["-c".into(), "sleep 1".into()],
            Duration::from_millis(20),
        )
        .await;
        assert!(detail.unknown);
        assert_eq!(detail.state, "unknown");
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn oversized_resolver_output_becomes_unknown() {
        let detail = run_resolver_command(
            Path::new("/bin/sh"),
            &[
                "-c".into(),
                "yes 'state: working · source: pane' | head -c 20000".into(),
            ],
            Duration::from_secs(1),
        )
        .await;
        assert!(detail.unknown);
        assert!(detail
            .detail
            .as_deref()
            .unwrap_or_default()
            .contains("safety limit"));
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn nonzero_exit_becomes_unknown() {
        let detail = run_resolver_command(
            Path::new("/bin/sh"),
            &[
                "-c".into(),
                "echo 'state: working · source: pane'; exit 3".into(),
            ],
            Duration::from_secs(1),
        )
        .await;
        assert!(detail.unknown);
        assert_eq!(detail.state, "unknown");
    }
}
