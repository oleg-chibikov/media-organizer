//! A suggestion id that stays the same for the same files across runs.

use crate::models::SuggestionKind;

const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0100_0000_01b3;

/// FNV-1a over the kind, the first file id and the start date. Unlike
/// `DefaultHasher`, the result doesn't change between Rust versions.
pub(super) fn suggestion_id(kind: SuggestionKind, first_file_id: &str, start: &str) -> String {
    let kind = match kind {
        SuggestionKind::Trip => "trip",
        SuggestionKind::Outing => "outing",
    };
    let mut hash = FNV_OFFSET;
    for byte in [kind, first_file_id, start].join("\n").bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    format!("{kind}-{hash:016x}")
}
