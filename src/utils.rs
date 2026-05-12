#[derive(Default)]
pub(crate) struct HeaderWriter(Vec<u8>);

impl HeaderWriter {
    pub fn push(&mut self, key: impl AsRef<[u8]>, value: impl AsRef<[u8]>) {
        self.0.extend_from_slice(key.as_ref());
        self.0.push(b' ');

        for &byte in value.as_ref() {
            self.0.push(byte);
            if byte == b'\n' {
                self.0.push(b' ');
            }
        }
        self.0.push(b'\n');
    }

    pub fn push_authorship(
        &mut self,
        key: impl AsRef<[u8]>,
        name: impl AsRef<[u8]>,
        timestamp: i64,
        timestamp_offset: impl AsRef<[u8]>,
    ) {
        let mut value = Vec::new();
        value.extend_from_slice(name.as_ref());
        value.push(b' ');
        value.extend_from_slice(timestamp.to_string().as_bytes());
        value.push(b' ');
        value.extend_from_slice(timestamp_offset.as_ref());
        self.push(key, value);
    }

    pub fn build(mut self, message: Option<impl AsRef<[u8]>>) -> Vec<u8> {
        if let Some(message) = message {
            self.0.push(b'\n');
            self.0.extend_from_slice(message.as_ref());
        }
        self.0
    }
}

/// Build a SWHID-format signature tuple from raw parts.
///
/// Used by both `git` (libgit2) and `git_gix` (gitoxide) backends.
pub(crate) fn build_signature(
    name: &[u8],
    email: &[u8],
    timestamp: i64,
    offset: &str,
) -> (Box<[u8]>, i64, Box<[u8]>) {
    let mut full_name = Vec::with_capacity(name.len() + email.len() + 3);
    full_name.extend_from_slice(name);
    full_name.extend_from_slice(b" <");
    full_name.extend_from_slice(email);
    full_name.push(b'>');
    (
        full_name.into(),
        timestamp,
        offset.as_bytes().to_vec().into_boxed_slice(),
    )
}

/// Returns `Err(item)` if the `item` is present twice in a row.
pub(crate) fn check_unique<T: AsRef<[u8]>>(items: impl IntoIterator<Item = T>) -> Result<(), T> {
    let mut items = items.into_iter();

    if let Some(first_item) = items.next() {
        let mut previous_item = first_item;
        for item in items {
            if item.as_ref() == previous_item.as_ref() {
                return Err(item);
            }
            previous_item = item;
        }
    }

    Ok(())
}
