use std::{
    fs::{self, File, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

use chrono::{DateTime, Utc};
use tauri::{AppHandle, Manager};

use crate::models::{
    HistoryIndexEntry, HistoryRecord, HistoryRun, HistoryRunCompletedRecord, HistoryRunStatus,
};

pub const HISTORY_VERSION: u32 = 1;
pub const HISTORY_DIR_NAME: &str = "history";
pub const HISTORY_RUNS_DIR_NAME: &str = "runs";
pub const HISTORY_INDEX_FILENAME: &str = "index.json";
pub const MAX_INDEX_RUNS: usize = 200;

static RUN_SEQUENCE: AtomicU64 = AtomicU64::new(0);

#[derive(Debug)]
pub struct ActiveHistoryRun {
    pub run: HistoryRun,
    history_root: PathBuf,
    log_file: File,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct HistoryIndexFile {
    history_version: u32,
    runs: Vec<HistoryIndexEntry>,
}

pub fn history_root_for_app(app: &AppHandle) -> Result<PathBuf, String> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|error| format!("Failed to resolve appData directory: {error}"))?;
    Ok(history_root_from_app_data(&app_data_dir))
}

pub fn history_root_from_app_data(app_data_dir: &Path) -> PathBuf {
    app_data_dir.join(HISTORY_DIR_NAME)
}

pub fn history_runs_dir(history_root: &Path) -> PathBuf {
    history_root.join(HISTORY_RUNS_DIR_NAME)
}

pub fn history_index_path(history_root: &Path) -> PathBuf {
    history_root.join(HISTORY_INDEX_FILENAME)
}

pub fn run_log_filename(started_at: DateTime<Utc>, run_id: &str) -> String {
    format!(
        "{}_{}.jsonl",
        started_at.format("%Y-%m-%dT%H-%M-%SZ"),
        run_id
    )
}

pub fn start_history_run(
    app: &AppHandle,
    selected_folder: &Path,
) -> Result<ActiveHistoryRun, String> {
    let history_root = history_root_for_app(app)?;
    let runs_dir = history_runs_dir(&history_root);
    fs::create_dir_all(&runs_dir).map_err(|error| {
        format!(
            "Failed to create history runs directory at '{}': {error}",
            runs_dir.display()
        )
    })?;

    let started_at = Utc::now();
    let run_id = generate_run_id(started_at);
    let file_name = run_log_filename(started_at, &run_id);
    let log_path = runs_dir.join(file_name);
    let history_file_path = log_path.to_string_lossy().to_string();

    let mut log_file = OpenOptions::new()
        .create_new(true)
        .append(true)
        .open(&log_path)
        .map_err(|error| {
            format!(
                "Failed to open history run log at '{}': {error}",
                log_path.display()
            )
        })?;

    let run = HistoryRun {
        run_id: run_id.clone(),
        history_version: HISTORY_VERSION,
        app_version: env!("CARGO_PKG_VERSION").to_string(),
        selected_folder: selected_folder.to_string_lossy().to_string(),
        folder_label: folder_label_from_path(selected_folder),
        started_at: started_at.to_rfc3339(),
        completed_at: None,
        run_status: HistoryRunStatus::Running,
        history_file_path: history_file_path.clone(),
    };

    append_history_record(
        &mut log_file,
        &HistoryRecord::RunStarted { run: run.clone() },
    )?;

    upsert_history_index_entry(&history_root, index_entry_from_run(&run))?;

    Ok(ActiveHistoryRun {
        run,
        history_root,
        log_file,
    })
}

pub fn complete_history_run(
    active_run: &mut ActiveHistoryRun,
    run_status: HistoryRunStatus,
) -> Result<(), String> {
    let completed_at = Utc::now().to_rfc3339();
    append_history_record(
        &mut active_run.log_file,
        &HistoryRecord::RunCompleted(HistoryRunCompletedRecord {
            run_id: active_run.run.run_id.clone(),
            completed_at: completed_at.clone(),
            run_status: run_status.clone(),
        }),
    )?;

    active_run.run.completed_at = Some(completed_at);
    active_run.run.run_status = run_status;
    upsert_history_index_entry(
        &active_run.history_root,
        index_entry_from_run(&active_run.run),
    )
}

fn generate_run_id(now: DateTime<Utc>) -> String {
    let sequence = RUN_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    format!(
        "{:x}{:x}{:x}",
        now.timestamp_micros().unsigned_abs(),
        std::process::id(),
        sequence
    )
}

fn folder_label_from_path(path: &Path) -> String {
    path.file_name()
        .and_then(|segment| segment.to_str())
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| path.to_string_lossy().to_string())
}

fn append_history_record(file: &mut File, record: &HistoryRecord) -> Result<(), String> {
    let line = serde_json::to_string(record)
        .map_err(|error| format!("Failed to serialize history record: {error}"))?;
    file.write_all(line.as_bytes())
        .map_err(|error| format!("Failed to append history record: {error}"))?;
    file.write_all(b"\n")
        .map_err(|error| format!("Failed to append history record newline: {error}"))?;
    file.flush()
        .map_err(|error| format!("Failed to flush history record: {error}"))
}

fn load_history_index(history_root: &Path) -> Result<HistoryIndexFile, String> {
    let index_path = history_index_path(history_root);
    if !index_path.exists() {
        return Ok(HistoryIndexFile {
            history_version: HISTORY_VERSION,
            runs: Vec::new(),
        });
    }

    let raw = fs::read_to_string(&index_path).map_err(|error| {
        format!(
            "Failed to read history index at '{}': {error}",
            index_path.display()
        )
    })?;

    serde_json::from_str(&raw).map_err(|error| {
        format!(
            "Failed to parse history index at '{}': {error}",
            index_path.display()
        )
    })
}

fn write_history_index(history_root: &Path, index: &HistoryIndexFile) -> Result<(), String> {
    fs::create_dir_all(history_root).map_err(|error| {
        format!(
            "Failed to create history directory at '{}': {error}",
            history_root.display()
        )
    })?;

    let serialized = serde_json::to_string_pretty(index)
        .map_err(|error| format!("Failed to serialize history index: {error}"))?;

    let index_path = history_index_path(history_root);
    fs::write(&index_path, serialized).map_err(|error| {
        format!(
            "Failed to write history index at '{}': {error}",
            index_path.display()
        )
    })
}

fn upsert_history_index_entry(history_root: &Path, entry: HistoryIndexEntry) -> Result<(), String> {
    let mut index = load_history_index(history_root)?;

    if let Some(existing) = index.runs.iter_mut().find(|run| run.run_id == entry.run_id) {
        *existing = entry;
    } else {
        index.runs.push(entry);
    }

    index.runs.sort_by(|left, right| {
        right
            .started_at
            .cmp(&left.started_at)
            .then_with(|| right.run_id.cmp(&left.run_id))
    });
    index.runs.truncate(MAX_INDEX_RUNS);
    write_history_index(history_root, &index)
}

fn index_entry_from_run(run: &HistoryRun) -> HistoryIndexEntry {
    HistoryIndexEntry {
        run_id: run.run_id.clone(),
        folder_label: run.folder_label.clone(),
        selected_folder: run.selected_folder.clone(),
        started_at: run.started_at.clone(),
        completed_at: run.completed_at.clone(),
        run_status: run.run_status.clone(),
        undo_available: false,
        redo_available: false,
        history_file_path: run.history_file_path.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{DateTime, Duration, TimeZone};
    use tempfile::tempdir;

    #[test]
    fn builds_history_paths_under_app_data() {
        let app_data = Path::new(r"C:\Users\user\AppData\Roaming\media-organizer");
        let history_root = history_root_from_app_data(app_data);
        let runs_dir = history_runs_dir(&history_root);
        let index_path = history_index_path(&history_root);

        assert_eq!(
            history_root,
            Path::new(r"C:\Users\user\AppData\Roaming\media-organizer\history")
        );
        assert_eq!(
            runs_dir,
            Path::new(r"C:\Users\user\AppData\Roaming\media-organizer\history\runs")
        );
        assert_eq!(
            index_path,
            Path::new(r"C:\Users\user\AppData\Roaming\media-organizer\history\index.json")
        );
    }

    #[test]
    fn generates_sortable_run_log_filename() {
        let started_at: DateTime<Utc> = Utc
            .with_ymd_and_hms(2026, 5, 24, 1, 2, 3)
            .single()
            .expect("valid utc time");

        let file_name = run_log_filename(started_at, "runabc123");
        assert_eq!(file_name, "2026-05-24T01-02-03Z_runabc123.jsonl");
    }

    #[test]
    fn appends_jsonl_history_records() {
        let temp = tempdir().expect("temp dir");
        let log_path = temp.path().join("history.jsonl");
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)
            .expect("open log");

        let started_record = HistoryRecord::RunStarted {
            run: HistoryRun {
                run_id: "run-1".to_string(),
                history_version: HISTORY_VERSION,
                app_version: "0.1.0".to_string(),
                selected_folder: r"C:\todo".to_string(),
                folder_label: "todo".to_string(),
                started_at: "2026-05-24T01:02:03Z".to_string(),
                completed_at: None,
                run_status: HistoryRunStatus::Running,
                history_file_path: r"C:\appdata\history\runs\run-1.jsonl".to_string(),
            },
        };
        let completed_record = HistoryRecord::RunCompleted(HistoryRunCompletedRecord {
            run_id: "run-1".to_string(),
            completed_at: "2026-05-24T01:05:00Z".to_string(),
            run_status: HistoryRunStatus::Completed,
        });

        append_history_record(&mut file, &started_record).expect("append start");
        append_history_record(&mut file, &completed_record).expect("append complete");
        drop(file);

        let contents = fs::read_to_string(&log_path).expect("read log");
        let lines: Vec<&str> = contents.lines().collect();
        assert_eq!(lines.len(), 2);
        assert!(lines[0].contains(r#""record_type":"run_started""#));
        assert!(lines[1].contains(r#""record_type":"run_completed""#));
    }

    #[test]
    fn keeps_index_sorted_and_bounded() {
        let temp = tempdir().expect("temp dir");
        let history_root = temp.path().join("history");
        let base_started_at = Utc
            .with_ymd_and_hms(2026, 5, 24, 0, 0, 0)
            .single()
            .expect("valid utc time");

        for value in 0..(MAX_INDEX_RUNS + 5) {
            let started_at = base_started_at + Duration::seconds(value as i64);
            let entry = HistoryIndexEntry {
                run_id: format!("run-{value}"),
                folder_label: "todo".to_string(),
                selected_folder: r"C:\todo".to_string(),
                started_at: started_at.to_rfc3339(),
                completed_at: None,
                run_status: HistoryRunStatus::Running,
                undo_available: false,
                redo_available: false,
                history_file_path: format!(r"C:\appdata\history\runs\run-{value}.jsonl"),
            };
            upsert_history_index_entry(&history_root, entry).expect("upsert index entry");
        }

        let index_path = history_index_path(&history_root);
        let raw = fs::read_to_string(index_path).expect("read index");
        let index: HistoryIndexFile = serde_json::from_str(&raw).expect("parse index");

        assert_eq!(index.runs.len(), MAX_INDEX_RUNS);
        assert_eq!(index.runs[0].run_id, format!("run-{}", MAX_INDEX_RUNS + 4));
        assert_eq!(index.runs[MAX_INDEX_RUNS - 1].run_id, "run-5");
    }
}
