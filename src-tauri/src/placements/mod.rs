//! The registry of trip and outing folders that runs filled, so a later scan of the
//! next month continues a folder instead of starting a new one.

mod continuation;
mod country_continuation;
mod folders;
mod lookup;
mod matching;
mod month_folder;
mod month_placement;
mod placed;
mod placed_files;
mod placed_filter;
mod recording;
mod restoring;
mod spans;
mod storage;
mod store;

#[cfg(test)]
mod tests;

pub(crate) use continuation::continue_placed_suggestions;
pub(crate) use folders::place_folder;
pub(crate) use month_folder::MonthFolder;
pub(crate) use placed::{PlacedFile, PlacedSuggestion};
pub(crate) use placed_files::placed_files;
pub(crate) use recording::{record_moves, MovedIntoPlace};
pub(crate) use restoring::{restore_files, UndoneMove};
pub(crate) use store::PlacementStore;
