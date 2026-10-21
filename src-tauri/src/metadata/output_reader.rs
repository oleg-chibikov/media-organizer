use std::{
    io::{BufRead, BufReader},
    process::{ChildStderr, ChildStdout},
    sync::mpsc::{self, Receiver},
};

pub(super) enum ReaderEvent {
    Ready { request_id: u64, payload: String },
    Exited,
}

/// Collects stdout lines into one payload per `{readyN}` marker on a background thread.
pub(super) fn spawn_stdout_reader(stdout: ChildStdout) -> Receiver<ReaderEvent> {
    let (tx, rx) = mpsc::channel::<ReaderEvent>();
    std::thread::spawn(move || {
        let mut payload = String::new();
        for line in BufReader::new(stdout).lines() {
            let Ok(value) = line else {
                let _ = tx.send(ReaderEvent::Exited);
                return;
            };

            if let Some(request_id) = parse_ready_marker(&value) {
                let payload = std::mem::take(&mut payload);
                if tx
                    .send(ReaderEvent::Ready {
                        request_id,
                        payload,
                    })
                    .is_err()
                {
                    return;
                }
                continue;
            }

            if !payload.is_empty() {
                payload.push('\n');
            }
            payload.push_str(&value);
        }

        let _ = tx.send(ReaderEvent::Exited);
    });
    rx
}

/// Drains stderr so pipe backpressure cannot block stdout.
pub(super) fn spawn_stderr_drain(stderr: ChildStderr) {
    std::thread::spawn(move || for _ in BufReader::new(stderr).lines() {});
}

fn parse_ready_marker(line: &str) -> Option<u64> {
    line.strip_prefix("{ready")
        .and_then(|value| value.strip_suffix('}'))
        .and_then(|value| value.parse::<u64>().ok())
}
