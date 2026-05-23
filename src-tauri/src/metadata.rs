use std::{
    io::{BufRead, BufReader, Write},
    path::{Path, PathBuf},
    process::{Child, ChildStdin, Command, Stdio},
    sync::mpsc::{self, Receiver},
    time::{Duration, Instant},
};

use chrono::{DateTime, Local, NaiveDateTime, TimeZone};
use tauri::Manager;

use crate::models::{DateKind, FileKind};

pub const DEFAULT_REQUEST_TIMEOUT_MS: u64 = 20_000;

const EXIFTOOL_TAGS: [&str; 7] = [
    "-SubSecDateTimeOriginal",
    "-DateTimeOriginal",
    "-CreateDate",
    "-ModifyDate",
    "-MediaCreateDate",
    "-TrackCreateDate",
    "-FileModifyDate",
];

#[derive(Debug, Clone)]
pub struct RawMetadataDates {
    pub sub_sec_date_time_original: Option<String>,
    pub date_time_original: Option<String>,
    pub create_date: Option<String>,
    pub modify_date: Option<String>,
    pub media_create_date: Option<String>,
    pub track_create_date: Option<String>,
    pub file_modify_date: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SelectedMetadataDate {
    pub chosen_date: String,
    pub chosen_date_source: String,
    pub date_kind: DateKind,
    pub raw_metadata_date: String,
}

#[derive(Debug, Clone)]
pub struct MetadataReadOutcome {
    pub raw_dates: RawMetadataDates,
    pub did_restart: bool,
}

#[derive(Default)]
pub struct ExifToolWorkerManager {
    worker: Option<ExifToolWorker>,
}

impl ExifToolWorkerManager {
    pub fn read_metadata(
        &mut self,
        app: &tauri::AppHandle,
        source_path: &Path,
        timeout: Duration,
    ) -> Result<MetadataReadOutcome, String> {
        let mut did_restart = false;

        for attempt in 0..=1 {
            let had_worker = self.worker.is_some();
            if self.worker_needs_restart() {
                self.shutdown_worker();
                self.worker = Some(ExifToolWorker::start(app)?);
                if had_worker {
                    did_restart = true;
                }
            }

            let Some(worker) = self.worker.as_mut() else {
                return Err("ExifTool worker is unavailable.".to_string());
            };

            match worker.read_metadata(source_path, timeout) {
                Ok(raw_dates) => {
                    return Ok(MetadataReadOutcome {
                        raw_dates,
                        did_restart,
                    });
                }
                Err(WorkerError::TimedOut(message)) => {
                    self.shutdown_worker();
                    return Err(message);
                }
                Err(WorkerError::Crashed(message)) => {
                    self.shutdown_worker();
                    if attempt == 0 {
                        did_restart = true;
                        continue;
                    }
                    return Err(message);
                }
                Err(WorkerError::Protocol(message)) => {
                    self.shutdown_worker();
                    return Err(message);
                }
            }
        }

        Err("ExifTool worker failed after restart attempt.".to_string())
    }

    fn worker_needs_restart(&mut self) -> bool {
        match self.worker.as_mut() {
            Some(worker) => !worker.is_alive(),
            None => true,
        }
    }

    fn shutdown_worker(&mut self) {
        if let Some(mut worker) = self.worker.take() {
            worker.shutdown();
        }
    }
}

enum ReaderEvent {
    Ready { request_id: u64, payload: String },
    Exited,
}

#[derive(Debug)]
enum WorkerError {
    TimedOut(String),
    Crashed(String),
    Protocol(String),
}

struct ExifToolWorker {
    child: Child,
    stdin: ChildStdin,
    reader_rx: Receiver<ReaderEvent>,
    next_request_id: u64,
}

impl ExifToolWorker {
    fn start(app: &tauri::AppHandle) -> Result<Self, String> {
        let exiftool_path = resolve_exiftool_path(app)?;

        let mut child = Command::new(&exiftool_path)
            .args(["-stay_open", "True", "-@", "-"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|error| format!("Failed to start ExifTool at {:?}: {error}", exiftool_path))?;

        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| "Failed to open ExifTool stdin.".to_string())?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| "Failed to open ExifTool stdout.".to_string())?;
        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| "Failed to open ExifTool stderr.".to_string())?;

        let (tx, rx) = mpsc::channel::<ReaderEvent>();
        std::thread::spawn(move || {
            let tx_stdout = tx.clone();
            let mut payload = String::new();
            for line in BufReader::new(stdout).lines() {
                match line {
                    Ok(value) => {
                        if let Some(request_id) = parse_ready_marker(&value) {
                            let completed_payload = std::mem::take(&mut payload);
                            if tx_stdout
                                .send(ReaderEvent::Ready {
                                    request_id,
                                    payload: completed_payload,
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
                    Err(_) => {
                        let _ = tx_stdout.send(ReaderEvent::Exited);
                        return;
                    }
                }
            }

            let _ = tx_stdout.send(ReaderEvent::Exited);
        });

        std::thread::spawn(move || {
            let stderr_reader = BufReader::new(stderr);
            for _ in stderr_reader.lines() {
                // Drain stderr to avoid pipe backpressure that can block stdout.
            }
        });

        Ok(Self {
            child,
            stdin,
            reader_rx: rx,
            next_request_id: 1,
        })
    }

    fn is_alive(&mut self) -> bool {
        match self.child.try_wait() {
            Ok(None) => true,
            Ok(Some(_)) => false,
            Err(_) => false,
        }
    }

    fn read_metadata(
        &mut self,
        source_path: &Path,
        timeout: Duration,
    ) -> Result<RawMetadataDates, WorkerError> {
        let request_id = self.next_request_id;
        self.next_request_id += 1;

        self.send_request(source_path, request_id)?;

        let deadline = Instant::now() + timeout;
        loop {
            let now = Instant::now();
            if now >= deadline {
                return Err(WorkerError::TimedOut(format!(
                    "ExifTool request timed out after {}ms for {}",
                    timeout.as_millis(),
                    source_path.to_string_lossy()
                )));
            }

            let remaining = deadline.saturating_duration_since(now);
            match self.reader_rx.recv_timeout(remaining) {
                Ok(ReaderEvent::Ready {
                    request_id: ready_id,
                    payload,
                }) => {
                    if ready_id != request_id {
                        return Err(WorkerError::Protocol(format!(
                            "ExifTool response framing mismatch: expected {request_id}, got {ready_id}."
                        )));
                    }
                    return parse_exiftool_json(&payload).map_err(WorkerError::Protocol);
                }
                Ok(ReaderEvent::Exited) => {
                    return Err(WorkerError::Crashed(
                        "ExifTool worker exited unexpectedly during metadata request.".to_string(),
                    ));
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    return Err(WorkerError::TimedOut(format!(
                        "ExifTool request timed out after {}ms for {}",
                        timeout.as_millis(),
                        source_path.to_string_lossy()
                    )));
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    return Err(WorkerError::Crashed(
                        "ExifTool worker output channel disconnected unexpectedly.".to_string(),
                    ));
                }
            }
        }
    }

    fn send_request(&mut self, source_path: &Path, request_id: u64) -> Result<(), WorkerError> {
        self.stdin.write_all(b"-json\n").map_err(|error| {
            WorkerError::Crashed(format!("Failed to write ExifTool request: {error}"))
        })?;

        for tag in EXIFTOOL_TAGS {
            self.stdin
                .write_all(format!("{tag}\n").as_bytes())
                .map_err(|error| {
                    WorkerError::Crashed(format!("Failed to write ExifTool tag request: {error}"))
                })?;
        }

        self.stdin
            .write_all(format!("{}\n", source_path.to_string_lossy()).as_bytes())
            .map_err(|error| {
                WorkerError::Crashed(format!("Failed to write ExifTool source path: {error}"))
            })?;
        self.stdin
            .write_all(format!("-execute{request_id}\n").as_bytes())
            .map_err(|error| {
                WorkerError::Crashed(format!("Failed to write ExifTool execute marker: {error}"))
            })?;
        self.stdin.flush().map_err(|error| {
            WorkerError::Crashed(format!("Failed to flush ExifTool stdin: {error}"))
        })?;

        Ok(())
    }

    fn shutdown(&mut self) {
        let _ = self.stdin.write_all(b"-stay_open\nFalse\n");
        let _ = self.stdin.write_all(b"-execute\n");
        let _ = self.stdin.flush();

        if self.child.try_wait().ok().flatten().is_none() {
            let _ = self.child.kill();
            let _ = self.child.wait();
        }
    }
}

impl Drop for ExifToolWorker {
    fn drop(&mut self) {
        self.shutdown();
    }
}

fn parse_ready_marker(line: &str) -> Option<u64> {
    line.strip_prefix("{ready")
        .and_then(|value| value.strip_suffix('}'))
        .and_then(|value| value.parse::<u64>().ok())
}

pub fn parse_exiftool_json(json: &str) -> Result<RawMetadataDates, String> {
    let values: Vec<serde_json::Value> =
        serde_json::from_str(json).map_err(|error| format!("Invalid ExifTool JSON: {error}"))?;
    let first = values
        .first()
        .ok_or_else(|| "ExifTool returned an empty JSON array.".to_string())?;
    let object = first
        .as_object()
        .ok_or_else(|| "ExifTool JSON payload was not an object.".to_string())?;

    Ok(RawMetadataDates {
        sub_sec_date_time_original: read_tag(object, "SubSecDateTimeOriginal"),
        date_time_original: read_tag(object, "DateTimeOriginal"),
        create_date: read_tag(object, "CreateDate"),
        modify_date: read_tag(object, "ModifyDate"),
        media_create_date: read_tag(object, "MediaCreateDate"),
        track_create_date: read_tag(object, "TrackCreateDate"),
        file_modify_date: read_tag(object, "FileModifyDate"),
    })
}

pub fn select_metadata_date(
    file_kind: &FileKind,
    raw_dates: &RawMetadataDates,
) -> Result<Option<SelectedMetadataDate>, String> {
    let mut parse_failures: Vec<String> = Vec::new();

    for (tag, value) in metadata_priority(file_kind, raw_dates) {
        let Some(raw_value) = value else {
            continue;
        };

        if let Some((chosen_date, date_kind)) =
            parse_metadata_timestamp(raw_value, tag == "FileModifyDate")
        {
            return Ok(Some(SelectedMetadataDate {
                chosen_date,
                chosen_date_source: tag.to_string(),
                date_kind,
                raw_metadata_date: raw_value.to_string(),
            }));
        }

        parse_failures.push(format!("{tag}={raw_value}"));
    }

    if parse_failures.is_empty() {
        Ok(None)
    } else {
        Err(format!(
            "Could not parse metadata date values in priority order: {}",
            parse_failures.join(", ")
        ))
    }
}

fn metadata_priority<'a>(
    file_kind: &FileKind,
    raw_dates: &'a RawMetadataDates,
) -> [(&'static str, Option<&'a String>); 5] {
    match file_kind {
        FileKind::Photo => [
            (
                "SubSecDateTimeOriginal",
                raw_dates.sub_sec_date_time_original.as_ref(),
            ),
            ("DateTimeOriginal", raw_dates.date_time_original.as_ref()),
            ("CreateDate", raw_dates.create_date.as_ref()),
            ("ModifyDate", raw_dates.modify_date.as_ref()),
            ("FileModifyDate", raw_dates.file_modify_date.as_ref()),
        ],
        FileKind::Video => [
            ("MediaCreateDate", raw_dates.media_create_date.as_ref()),
            ("TrackCreateDate", raw_dates.track_create_date.as_ref()),
            ("CreateDate", raw_dates.create_date.as_ref()),
            ("ModifyDate", raw_dates.modify_date.as_ref()),
            ("FileModifyDate", raw_dates.file_modify_date.as_ref()),
        ],
        _ => [
            ("CreateDate", raw_dates.create_date.as_ref()),
            ("ModifyDate", raw_dates.modify_date.as_ref()),
            ("FileModifyDate", raw_dates.file_modify_date.as_ref()),
            ("DateTimeOriginal", raw_dates.date_time_original.as_ref()),
            ("MediaCreateDate", raw_dates.media_create_date.as_ref()),
        ],
    }
}

fn parse_metadata_timestamp(raw: &str, is_fallback: bool) -> Option<(String, DateKind)> {
    if let Some(local) = parse_timezone_aware_to_local(raw) {
        let kind = if is_fallback {
            DateKind::Fallback
        } else {
            DateKind::TimezoneAware
        };
        return Some((local.format("%Y-%m-%d %H:%M:%S").to_string(), kind));
    }

    if let Some(local_naive) = parse_timezone_unknown_as_local(raw) {
        let kind = if is_fallback {
            DateKind::Fallback
        } else {
            DateKind::TimezoneUnknown
        };
        return Some((local_naive.format("%Y-%m-%d %H:%M:%S").to_string(), kind));
    }

    None
}

fn parse_timezone_aware_to_local(raw: &str) -> Option<DateTime<Local>> {
    let candidates = [
        "%Y:%m:%d %H:%M:%S%.f%:z",
        "%Y:%m:%d %H:%M:%S%:z",
        "%Y:%m:%d %H:%M:%S%.f%z",
        "%Y:%m:%d %H:%M:%S%z",
    ];

    for format in candidates {
        if let Ok(parsed) = DateTime::parse_from_str(raw, format) {
            return Some(parsed.with_timezone(&Local));
        }
    }

    if let Some(stripped) = raw.strip_suffix('Z') {
        let normalized = format!("{stripped}+00:00");
        for format in ["%Y:%m:%d %H:%M:%S%.f%:z", "%Y:%m:%d %H:%M:%S%:z"] {
            if let Ok(parsed) = DateTime::parse_from_str(&normalized, format) {
                return Some(parsed.with_timezone(&Local));
            }
        }
    }

    None
}

fn parse_timezone_unknown_as_local(raw: &str) -> Option<NaiveDateTime> {
    let candidates = ["%Y:%m:%d %H:%M:%S%.f", "%Y:%m:%d %H:%M:%S"];

    for format in candidates {
        if let Ok(naive) = NaiveDateTime::parse_from_str(raw, format) {
            return match Local.from_local_datetime(&naive) {
                chrono::LocalResult::Single(value) => Some(value.naive_local()),
                chrono::LocalResult::Ambiguous(first, _) => Some(first.naive_local()),
                chrono::LocalResult::None => None,
            };
        }
    }

    None
}

fn read_tag(object: &serde_json::Map<String, serde_json::Value>, key: &str) -> Option<String> {
    object
        .get(key)
        .and_then(|value| value.as_str())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
}

fn resolve_exiftool_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let mut candidates: Vec<PathBuf> = Vec::new();

    if let Ok(resource_dir) = app.path().resource_dir() {
        candidates.push(resource_dir.join("exiftool-x86_64-pc-windows-msvc.exe"));
        candidates.push(resource_dir.join("exiftool.exe"));
        candidates.push(
            resource_dir
                .join("binaries")
                .join("exiftool-x86_64-pc-windows-msvc.exe"),
        );
    }

    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            candidates.push(exe_dir.join("exiftool-x86_64-pc-windows-msvc.exe"));
            candidates.push(exe_dir.join("exiftool.exe"));
        }
    }

    let current_dir = std::env::current_dir()
        .map_err(|error| format!("Failed to resolve current directory: {error}"))?;
    candidates.push(
        current_dir
            .join("src-tauri")
            .join("binaries")
            .join("exiftool-x86_64-pc-windows-msvc.exe"),
    );

    if let Some(found) = candidates.into_iter().find(|path| path.exists()) {
        return Ok(found);
    }

    Err(
        "Missing ExifTool sidecar binary. Expected `src-tauri/binaries/exiftool-x86_64-pc-windows-msvc.exe`."
            .to_string(),
    )
}

#[cfg(test)]
mod tests {
    use super::{parse_exiftool_json, select_metadata_date, RawMetadataDates};
    use crate::models::{DateKind, FileKind};

    #[test]
    fn parses_known_exiftool_fields() {
        let json = r#"[{
            "SourceFile": "C:/todo/example.jpg",
            "SubSecDateTimeOriginal": "2026:03:05 07:20:17.616+11:00",
            "DateTimeOriginal": "2026:03:05 07:20:17",
            "CreateDate": "2026:03:05 07:20:17"
        }]"#;

        let parsed = parse_exiftool_json(json).expect("json should parse");
        assert_eq!(
            parsed.sub_sec_date_time_original.as_deref(),
            Some("2026:03:05 07:20:17.616+11:00")
        );
        assert_eq!(
            parsed.date_time_original.as_deref(),
            Some("2026:03:05 07:20:17")
        );
        assert_eq!(parsed.media_create_date, None);
    }

    #[test]
    fn rejects_empty_json_payload() {
        let result = parse_exiftool_json("[]");
        assert!(result.is_err());
    }

    fn empty_raw_dates() -> RawMetadataDates {
        RawMetadataDates {
            sub_sec_date_time_original: None,
            date_time_original: None,
            create_date: None,
            modify_date: None,
            media_create_date: None,
            track_create_date: None,
            file_modify_date: None,
        }
    }

    #[test]
    fn chooses_photo_priority_order() {
        let mut raw = empty_raw_dates();
        raw.date_time_original = Some("2026:03:05 07:20:17".to_string());
        raw.create_date = Some("2024:01:01 01:02:03".to_string());

        let selected = select_metadata_date(&FileKind::Photo, &raw)
            .expect("selection should succeed")
            .expect("a date should be selected");

        assert_eq!(selected.chosen_date_source, "DateTimeOriginal");
        assert!(matches!(selected.date_kind, DateKind::TimezoneUnknown));
        assert_eq!(selected.raw_metadata_date, "2026:03:05 07:20:17");
    }

    #[test]
    fn chooses_video_priority_order() {
        let mut raw = empty_raw_dates();
        raw.track_create_date = Some("2026:03:05 07:20:17".to_string());
        raw.create_date = Some("2025:01:01 00:00:00".to_string());

        let selected = select_metadata_date(&FileKind::Video, &raw)
            .expect("selection should succeed")
            .expect("a date should be selected");

        assert_eq!(selected.chosen_date_source, "TrackCreateDate");
        assert!(matches!(selected.date_kind, DateKind::TimezoneUnknown));
    }

    #[test]
    fn marks_file_modify_date_as_fallback() {
        let mut raw = empty_raw_dates();
        raw.file_modify_date = Some("2026:03:05 07:20:17".to_string());

        let selected = select_metadata_date(&FileKind::Photo, &raw)
            .expect("selection should succeed")
            .expect("a date should be selected");

        assert_eq!(selected.chosen_date_source, "FileModifyDate");
        assert!(matches!(selected.date_kind, DateKind::Fallback));
    }

    #[test]
    fn parses_timezone_aware_dates() {
        let mut raw = empty_raw_dates();
        raw.sub_sec_date_time_original = Some("2026:03:05 07:20:17.616+11:00".to_string());

        let selected = select_metadata_date(&FileKind::Photo, &raw)
            .expect("selection should succeed")
            .expect("a date should be selected");

        assert_eq!(selected.chosen_date_source, "SubSecDateTimeOriginal");
        assert!(matches!(selected.date_kind, DateKind::TimezoneAware));
        assert!(selected.chosen_date.starts_with("2026-03-05 07:20:17"));
    }

    #[test]
    fn parses_timezone_less_dates_as_local_wall_time() {
        let mut raw = empty_raw_dates();
        raw.create_date = Some("2026:03:05 07:20:17".to_string());

        let selected = select_metadata_date(&FileKind::Photo, &raw)
            .expect("selection should succeed")
            .expect("a date should be selected");

        assert_eq!(selected.chosen_date, "2026-03-05 07:20:17");
        assert!(matches!(selected.date_kind, DateKind::TimezoneUnknown));
    }
}
