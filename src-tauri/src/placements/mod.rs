//! The registry of trip and outing folders that runs filled, so a later scan of the
//! next month continues a folder instead of starting a new one.

mod continuation;
mod lookup;
mod matching;
mod placed;
mod recording;
mod spans;
mod storage;
mod store;

#[cfg(test)]
mod tests;

pub(crate) use continuation::continue_placed_suggestions;
pub(crate) use placed::{top_folder, PlacedFile, PlacedSuggestion};
pub(crate) use recording::{forget_files, record_moves, MovedIntoPlace};
pub(crate) use store::PlacementStore;
