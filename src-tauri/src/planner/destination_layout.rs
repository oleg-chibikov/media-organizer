use std::path::{Path, PathBuf};

use chrono::{Datelike, NaiveDateTime};

use crate::models::{DestinationRootMode, EffectiveDestinationRoot};

/// Where a planned file goes: a dated folder under a root, or the folder it already sits in.
#[derive(Debug, Clone)]
pub(crate) enum DestinationLayout {
    DatedFolders(PathBuf),
    SameFolder,
}

impl DestinationLayout {
    pub(crate) fn for_root(root: &EffectiveDestinationRoot) -> Self {
        match root.mode {
            DestinationRootMode::InPlace => Self::SameFolder,
            DestinationRootMode::CustomRoot | DestinationRootMode::ProcessedFolder => {
                Self::DatedFolders(PathBuf::from(&root.root_path))
            }
        }
    }

    /// `<root>/<year>/<month>/<day>` or the source file's own folder.
    pub(super) fn destination_dir(
        &self,
        source_path: &Path,
        date: &NaiveDateTime,
    ) -> Result<PathBuf, String> {
        match self {
            Self::DatedFolders(root) => Ok(root
                .join(date.format("%Y").to_string())
                .join(date.month().to_string())
                .join(date.day().to_string())),
            Self::SameFolder => source_path
                .parent()
                .map(Path::to_path_buf)
                .ok_or_else(|| format!("{} has no parent folder.", source_path.display())),
        }
    }
}
