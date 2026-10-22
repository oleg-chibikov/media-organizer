//! Read-only byte comparison shared by planning and copy verification.

mod cached_same_bytes;
mod same_bytes;

#[cfg(test)]
mod tests;

pub(crate) use cached_same_bytes::files_have_same_bytes_cached;
pub(crate) use same_bytes::files_have_same_bytes;
