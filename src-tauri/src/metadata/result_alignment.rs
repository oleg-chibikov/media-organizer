use std::{collections::HashMap, path::PathBuf};

use super::RawMetadataDates;

/// Pairs ExifTool results with the requested paths, one slot per path.
///
/// ExifTool leaves out a file it can't open, which shifts every later result by one. When
/// results are missing they are matched by `SourceFile`, and a file without one gets `None`.
pub(super) fn align_results(
    source_paths: &[PathBuf],
    results: Vec<RawMetadataDates>,
) -> Vec<Option<RawMetadataDates>> {
    if results.len() == source_paths.len() {
        return results.into_iter().map(Some).collect();
    }
    let by_source: HashMap<String, RawMetadataDates> = results
        .into_iter()
        .filter_map(|result| Some((source_key(result.source_file.as_deref()?), result)))
        .collect();
    source_paths
        .iter()
        .map(|path| by_source.get(&source_key(&path.to_string_lossy())).cloned())
        .collect()
}

/// ExifTool may echo Windows paths with forward slashes.
fn source_key(path: &str) -> String {
    path.replace('\\', "/")
}
