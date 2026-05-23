use std::{
    path::{Path, PathBuf},
    process::Command,
};

use tauri::Manager;

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

pub fn read_metadata_with_exiftool(
    app: &tauri::AppHandle,
    source_path: &Path,
) -> Result<RawMetadataDates, String> {
    let exiftool_path = resolve_exiftool_path(app)?;
    let output = Command::new(&exiftool_path)
        .args([
            "-json",
            "-SubSecDateTimeOriginal",
            "-DateTimeOriginal",
            "-CreateDate",
            "-ModifyDate",
            "-MediaCreateDate",
            "-TrackCreateDate",
            "-FileModifyDate",
        ])
        .arg(source_path)
        .output()
        .map_err(|error| format!("Failed to run ExifTool at {:?}: {error}", exiftool_path))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(if stderr.is_empty() {
            format!("ExifTool exited with status {}", output.status)
        } else {
            format!("ExifTool failed: {stderr}")
        });
    }

    let stdout = String::from_utf8(output.stdout)
        .map_err(|error| format!("ExifTool output was not valid UTF-8: {error}"))?;
    parse_exiftool_json(&stdout)
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

fn read_tag(
    object: &serde_json::Map<String, serde_json::Value>,
    key: &str,
) -> Option<String> {
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
    use super::parse_exiftool_json;

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
}
