//! Byte-level file primitives shared by planning, moving and redo.

mod new_file_copy;
mod same_bytes;

#[cfg(test)]
mod tests;

pub(crate) use new_file_copy::{copy_into, copy_into_new_file};
pub(crate) use same_bytes::files_have_same_bytes;
