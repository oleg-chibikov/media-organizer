use std::path::{Path, PathBuf};

use chrono::{Datelike, NaiveDateTime};

use crate::{models::DestinationRootMode, settings::SettingsStore};

/// Where a planned file goes: a dated folder under a root, or the folder it already sits in.
#[derive(Debug, Clone)]
pub(crate) enum DestinationLayout {
    DatedFolders(PathBuf),
    /// Renames in place. Files in an accepted place folder still move under `place_root`.
    SameFolder {
        place_root: Option<PathBuf>,
    },
}

impl DestinationLayout {
    /// The layout for `selected_folder`. In rename in place mode the place folder root is
    /// resolved only `with_places`, so a missing custom root doesn't block plain renames.
    pub(crate) fn for_folder(
        settings: &SettingsStore,
        selected_folder: &Path,
        with_places: bool,
    ) -> Result<Self, String> {
        let root = settings.effective_destination_root(selected_folder)?;
        if !matches!(root.mode, DestinationRootMode::InPlace) {
            return Ok(Self::DatedFolders(PathBuf::from(root.root_path)));
        }
        let place_root = with_places
            .then(|| settings.place_folder_root(selected_folder))
            .transpose()?;
        Ok(Self::SameFolder { place_root })
    }

    /// The root place folders go under, when there is one.
    pub(crate) fn place_root(&self) -> Option<&Path> {
        match self {
            Self::DatedFolders(root) => Some(root),
            Self::SameFolder { place_root } => place_root.as_deref(),
        }
    }

    /// `<root>/<year>/<month>/<day>`, the file's place folder under the place root,
    /// or the source file's own folder.
    pub(super) fn destination_dir(
        &self,
        source_path: &Path,
        date: &NaiveDateTime,
        place_dir: Option<PathBuf>,
    ) -> Result<PathBuf, String> {
        if let Some(place_dir) = place_dir {
            let root = self
                .place_root()
                .ok_or("No root folder for place folders was resolved.")?;
            return Ok(root.join(place_dir));
        }
        match self {
            Self::DatedFolders(root) => Ok(root
                .join(date.format("%Y").to_string())
                .join(date.month().to_string())
                .join(date.day().to_string())),
            Self::SameFolder { .. } => source_path
                .parent()
                .map(Path::to_path_buf)
                .ok_or_else(|| format!("{} has no parent folder.", source_path.display())),
        }
    }
}
