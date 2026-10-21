//! Read-only byte comparison shared by planning and copy verification.

mod same_bytes;

#[cfg(test)]
mod tests;

pub(crate) use same_bytes::files_have_same_bytes;
