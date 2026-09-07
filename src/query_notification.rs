use bytes::{BufMut, BytesMut};

/// Per-request SQL Server Query Notification registration (TDS 7.2+).
#[derive(Clone, Debug)]
pub struct QueryNotification {
    id: Vec<u16>,
    options: Vec<u16>,
    timeout: u32,
}

impl QueryNotification {
    /// Create a registration with a correlation ID, Service Broker options
    /// (for example `service=Changes;local database=Example`), and timeout.
    /// The timeout is the SQL Server notification timeout in seconds.
    pub fn new(id: &str, options: &str, timeout_seconds: u32) -> crate::Result<Self> {
        let id: Vec<_> = id.encode_utf16().collect();
        let options: Vec<_> = options.encode_utf16().collect();
        if id.is_empty() || options.is_empty() || id.len() > 32767 || options.len() > 32767 {
            return Err(crate::error::Error::Conversion(
                "notification ID and options must contain 1..32767 UTF-16 code units".into(),
            ));
        }
        Ok(Self {
            id,
            options,
            timeout: timeout_seconds,
        })
    }

    pub(crate) fn encoded_len(&self) -> u32 {
        (14 + (self.id.len() + self.options.len()) * 2) as u32
    }

    pub(crate) fn encode(self, dst: &mut BytesMut) {
        dst.put_u32_le(self.encoded_len());
        dst.put_u16_le(1);
        for value in [&self.id, &self.options] {
            dst.put_u16_le((value.len() * 2) as u16);
            for unit in value {
                dst.put_u16_le(*unit);
            }
        }
        dst.put_u32_le(self.timeout);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn encodes_byte_lengths_and_little_endian_fields() {
        let mut bytes = BytesMut::new();
        QueryNotification::new("A", "😀", 60)
            .unwrap()
            .encode(&mut bytes);
        assert_eq!(
            &bytes[..],
            &[20, 0, 0, 0, 1, 0, 2, 0, 65, 0, 4, 0, 61, 216, 0, 222, 60, 0, 0, 0]
        );
    }
    #[test]
    fn rejects_empty_and_overflowing_utf16_fields() {
        assert!(QueryNotification::new("", "x", 1).is_err());
        assert!(QueryNotification::new(&"😀".repeat(16384), "x", 1).is_err());
    }
}
