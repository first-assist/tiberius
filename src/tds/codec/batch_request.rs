use super::{AllHeaderTy, Encode, ALL_HEADERS_LEN_TX};
use bytes::{BufMut, BytesMut};
use std::borrow::Cow;

pub struct BatchRequest<'a> {
    queries: Cow<'a, str>,
    transaction_descriptor: [u8; 8],
    notification: Option<crate::QueryNotification>,
}

impl<'a> BatchRequest<'a> {
    pub fn new(queries: impl Into<Cow<'a, str>>, transaction_descriptor: [u8; 8]) -> Self {
        Self {
            queries: queries.into(),
            transaction_descriptor,
            notification: None,
        }
    }
}

impl BatchRequest<'_> {
    pub fn with_notification(mut self, notification: crate::QueryNotification) -> Self {
        self.notification = Some(notification);
        self
    }
}

impl<'a> Encode<BytesMut> for BatchRequest<'a> {
    fn encode(self, dst: &mut BytesMut) -> crate::Result<()> {
        let notification_len = self.notification.as_ref().map_or(0, |n| n.encoded_len());
        dst.put_u32_le(ALL_HEADERS_LEN_TX as u32 + notification_len);
        dst.put_u32_le(ALL_HEADERS_LEN_TX as u32 - 4);
        dst.put_u16_le(AllHeaderTy::TransactionDescriptor as u16);
        dst.put_slice(&self.transaction_descriptor);
        dst.put_u32_le(1);

        if let Some(notification) = self.notification {
            notification.encode(dst);
        }

        for c in self.queries.encode_utf16() {
            dst.put_u16_le(c);
        }

        Ok(())
    }
}

#[cfg(test)]
mod notification_tests {
    use super::*;
    #[test]
    fn batch_total_length_includes_notification_and_preserves_sql() {
        let mut plain = BytesMut::new();
        BatchRequest::new("SELECT 1", [0; 8])
            .encode(&mut plain)
            .unwrap();
        let mut notified = BytesMut::new();
        let notification = crate::QueryNotification::new("A", "B", 60).unwrap();
        BatchRequest::new("SELECT 1", [0; 8])
            .with_notification(notification)
            .encode(&mut notified)
            .unwrap();
        assert_eq!(u32::from_le_bytes(plain[..4].try_into().unwrap()), 22);
        assert_eq!(u32::from_le_bytes(notified[..4].try_into().unwrap()), 40);
        assert_eq!(&plain[4..22], &notified[4..22]);
        assert_eq!(&plain[22..], &notified[40..]);
    }
}
