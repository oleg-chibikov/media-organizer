use serde_json::{Map, Value};

use super::RawMetadataDates;

/// Android phones write the model of a video to `AndroidModel` or Samsung's `SamsungModel`.
const DEVICE_NAME_TAGS: [&str; 7] = [
    "Model",
    "AndroidModel",
    "SamsungModel",
    "DeviceModelName",
    "CameraModelName",
    "Make",
    "AndroidMake",
];

#[cfg(test)]
pub(super) fn parse_exiftool_json(json: &str) -> Result<RawMetadataDates, String> {
    parse_exiftool_json_batch(json)?
        .into_iter()
        .next()
        .ok_or_else(|| "ExifTool returned an empty JSON array.".to_string())
}

pub(super) fn parse_exiftool_json_batch(json: &str) -> Result<Vec<RawMetadataDates>, String> {
    let values: Vec<Value> =
        serde_json::from_str(json).map_err(|error| format!("Invalid ExifTool JSON: {error}"))?;

    values
        .iter()
        .map(|value| {
            let object = value.as_object().ok_or_else(|| {
                "ExifTool JSON payload contained a non-object result.".to_string()
            })?;

            Ok(RawMetadataDates {
                source_file: read_tag(object, "SourceFile"),
                sub_sec_date_time_original: read_tag(object, "SubSecDateTimeOriginal"),
                date_time_original: read_tag(object, "DateTimeOriginal"),
                create_date: read_tag(object, "CreateDate"),
                modify_date: read_tag(object, "ModifyDate"),
                media_create_date: read_tag(object, "MediaCreateDate"),
                track_create_date: read_tag(object, "TrackCreateDate"),
                file_modify_date: read_tag(object, "FileModifyDate"),
                device_name: DEVICE_NAME_TAGS
                    .iter()
                    .find_map(|key| read_tag(object, key)),
            })
        })
        .collect()
}

fn read_tag(object: &Map<String, Value>, key: &str) -> Option<String> {
    object
        .get(key)
        .and_then(|value| value.as_str())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
}
