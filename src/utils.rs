#[derive(Default)]
pub(crate) struct HeaderWriter(Vec<u8>);

impl HeaderWriter {
    /// Push a header key-value pair with proper multiline continuation handling.
    ///
    /// Git's object format uses a space after newlines to indicate continuation
    /// lines. However, we should only add the continuation space when the newline
    /// is NOT the final byte of the value, to avoid corrupting values that end
    /// with a newline (e.g., PGP signatures).
    pub fn push(&mut self, key: impl AsRef<[u8]>, value: impl AsRef<[u8]>) {
        self.0.extend_from_slice(key.as_ref());
        self.0.push(b' ');

        let value_bytes = value.as_ref();
        let value_len = value_bytes.len();
        
        for (i, &byte) in value_bytes.iter().enumerate() {
            self.0.push(byte);
            // Only add continuation space if:
            // 1. This is a newline byte
            // 2. This is NOT the last byte of the value
            // This prevents adding a continuation space after a trailing newline
            if byte == b'\n' && i < value_len - 1 {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn header_writer_simple() {
        let mut writer = HeaderWriter::default();
        writer.push(b"key", b"value");
        let result = writer.build(None::<&[u8]>);
        assert_eq!(result, b"key value\n");
    }

    #[test]
    fn header_writer_multiline() {
        let mut writer = HeaderWriter::default();
        writer.push(b"key", b"line1\nline2");
        let result = writer.build(None::<&[u8]>);
        // Should have continuation space after first newline, but not at end
        assert_eq!(result, b"key line1\n line2\n");
    }

    #[test]
    fn header_writer_trailing_newline() {
        // P0.4 fix: value ending with newline should not get extra continuation space
        let mut writer = HeaderWriter::default();
        writer.push(b"gpgsig", b"-----BEGIN PGP SIGNATURE-----\nblah\n-----END PGP SIGNATURE-----\n");
        let result = writer.build(None::<&[u8]>);
        // Should NOT have continuation space after final newline
        // The final newline should be followed directly by the header terminator
        let expected = b"gpgsig -----BEGIN PGP SIGNATURE-----\n blah\n -----END PGP SIGNATURE-----\n\n";
        assert_eq!(result, expected);
    }

    #[test]
    fn header_writer_no_trailing_newline() {
        let mut writer = HeaderWriter::default();
        writer.push(b"key", b"value\nline2");
        let result = writer.build(None::<&[u8]>);
        // Should have continuation space after newline (not at end)
        assert_eq!(result, b"key value\n line2\n");
    }

    #[test]
    fn header_writer_with_message() {
        let mut writer = HeaderWriter::default();
        writer.push(b"key", b"value");
        let result = writer.build(Some(b"message" as &[u8]));
        assert_eq!(result, b"key value\n\nmessage");
    }
}
