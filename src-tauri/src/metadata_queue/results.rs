use std::time::Duration;

use crate::{
    metadata::{self, RawMetadataDates},
    models::{MetadataReadRequest, MetadataResult, MetadataStatus},
};

const MAX_BATCH_TIMEOUT_MS: u128 = 120_000;

pub(super) fn metadata_result_from_raw(
    request: MetadataReadRequest,
    raw_dates: RawMetadataDates,
    did_restart: bool,
) -> MetadataResult {
    let restart_warning = did_restart
        .then(|| "ExifTool worker restarted after a previous crash or stall.".to_string());

    let selection = match metadata::select_metadata_date(&request.file_kind, &raw_dates) {
        Ok(value) => value,
        Err(error) => return metadata_error_result(request.file_id, error),
    };

    if let Some(date) = selection {
        return MetadataResult {
            file_id: request.file_id,
            chosen_date: Some(date.chosen_date),
            chosen_date_source: Some(date.chosen_date_source),
            date_kind: Some(date.date_kind),
            raw_metadata_date: Some(date.raw_metadata_date),
            device_name: raw_dates.device_name,
            metadata_status: MetadataStatus::Ready,
            error: restart_warning,
        };
    }

    MetadataResult {
        file_id: request.file_id,
        chosen_date: None,
        chosen_date_source: None,
        date_kind: None,
        raw_metadata_date: None,
        device_name: raw_dates.device_name,
        metadata_status: MetadataStatus::NoDate,
        error: restart_warning,
    }
}

pub(crate) fn metadata_error_result(file_id: String, error: String) -> MetadataResult {
    MetadataResult {
        file_id,
        chosen_date: None,
        chosen_date_source: None,
        date_kind: None,
        raw_metadata_date: None,
        device_name: None,
        metadata_status: MetadataStatus::Error,
        error: Some(error),
    }
}

pub(super) fn metadata_batch_timeout(base_timeout: Duration, batch_size: usize) -> Duration {
    let scaled = base_timeout.as_millis().saturating_mul(batch_size as u128);
    Duration::from_millis(scaled.min(MAX_BATCH_TIMEOUT_MS) as u64)
}

pub(super) fn unwrap_metadata_results(results: Vec<Option<MetadataResult>>) -> Vec<MetadataResult> {
    results
        .into_iter()
        .map(|result| {
            result.unwrap_or_else(|| {
                metadata_error_result(
                    String::new(),
                    "Metadata worker did not produce a result.".to_string(),
                )
            })
        })
        .collect()
}
