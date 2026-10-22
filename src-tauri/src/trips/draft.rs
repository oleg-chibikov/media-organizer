//! Suggestions while they are being grouped, as indexes into the sorted shots.

use chrono::NaiveDateTime;

use super::place_table::PlaceId;
use crate::models::SuggestionKind;

pub(super) struct Draft {
    pub(super) kind: SuggestionKind,
    pub(super) groups: Vec<DraftGroup>,
    /// Trip shots in no place folder.
    pub(super) root: Vec<usize>,
    /// Geotagged members in time order. Files with no GPS join through them.
    pub(super) anchors: Vec<Anchor>,
}

pub(super) struct DraftGroup {
    pub(super) place: PlaceId,
    pub(super) shots: Vec<usize>,
}

#[derive(Clone, Copy)]
pub(super) struct Anchor {
    pub(super) taken: NaiveDateTime,
    /// Index into `Draft::groups`, or `None` for the trip root.
    pub(super) group: Option<usize>,
}

impl Draft {
    pub(super) fn first_taken(&self) -> NaiveDateTime {
        self.anchors[0].taken
    }

    pub(super) fn last_taken(&self) -> NaiveDateTime {
        self.anchors[self.anchors.len() - 1].taken
    }

    pub(super) fn add(&mut self, shot: usize, group: Option<usize>) {
        match group {
            Some(group) => self.groups[group].shots.push(shot),
            None => self.root.push(shot),
        }
    }
}
