use hmac::{digest::MacError, Hmac, Mac};
use prost::DecodeError;
use sha2::Sha256;
use thiserror::Error;
use time::OffsetDateTime;

use crate::proto::google::r#type::Money;

pub type HmacSha256 = Hmac<Sha256>;
pub type Key = Vec<u8>;

pub struct TagManager {
    secret: Key,
}

impl TagManager {
    pub fn new(secret: Key) -> Self {
        Self { secret }
    }

    fn build_mac(&self, flight_id: &str, price: &Money, expiration: i64) -> HmacSha256 {
        let mut mac = HmacSha256::new_from_slice(&self.secret).unwrap();

        mac.update(flight_id.as_bytes());
        mac.update(&expiration.to_le_bytes());

        let Money {
            currency_code,
            units,
            nanos,
        } = price;

        mac.update(currency_code.as_bytes());
        mac.update(&units.to_le_bytes());
        mac.update(&nanos.to_le_bytes());

        mac
    }

    pub fn generate_tag(&self, flight_id: &str, price: &Money, expiration: i64) -> Vec<u8> {
        let mac = self.build_mac(flight_id, price, expiration);

        let result = mac.finalize();

        result.into_bytes().to_vec()
    }

    pub fn verify_offer(
        &self,
        flight_id: &str,
        price: &Money,
        expiration: i64,
        tag: &[u8],
    ) -> Result<(), TokenVerifyError> {
        let mac = self.build_mac(flight_id, price, expiration);

        mac.verify_slice(tag)?;
        Self::verify_expired(expiration)?;

        Ok(())
    }

    fn verify_expired(expiration: i64) -> Result<(), TokenVerifyError> {
        if expiration < OffsetDateTime::now_utc().unix_timestamp() {
            return Err(TokenVerifyError::Expired);
        }

        Ok(())
    }
}

#[derive(Error, Debug)]
pub enum TokenVerifyError {
    #[error("Invalid token signature: {0}")]
    InvalidSignature(#[from] MacError),

    #[error("Invalid token data: {0}")]
    InvalidData(#[from] DecodeError),

    #[error("Token is expired")]
    Expired,
}

impl From<TokenVerifyError> for tonic::Status {
    fn from(_: TokenVerifyError) -> tonic::Status {
        tonic::Status::invalid_argument("offer is invalid")
    }
}

#[cfg(test)]
mod tests {
    use time::Duration;

    use super::*;

    #[test]
    fn test_basic() {
        let manager = TagManager::new(b"secret".to_vec());
        let expiration = (OffsetDateTime::now_utc() + Duration::minutes(10)).unix_timestamp();

        let flight_id = "id";
        let price = Money {
            currency_code: String::from("USD"),
            units: 100,
            nanos: 0,
        };

        let token = manager.generate_tag(flight_id, &price, expiration);
        manager
            .verify_offer(flight_id, &price, expiration, &token)
            .unwrap();
    }

    #[test]
    fn test_invalid_data() {
        let manager = TagManager::new(b"secret".to_vec());
        let expiration = (OffsetDateTime::now_utc() + Duration::minutes(10)).unix_timestamp();

        let tag = manager.generate_tag(
            "id",
            &Money {
                currency_code: String::from("USD"),
                units: 100,
                nanos: 0,
            },
            expiration,
        );

        manager
            .verify_offer(
                "id",
                &Money {
                    currency_code: String::from("USD"),
                    units: 10,
                    nanos: 0,
                },
                expiration,
                &tag,
            )
            .unwrap_err();
    }

    #[test]
    fn test_expired() {
        let manager = TagManager::new(b"secret".to_vec());
        let expiration = (OffsetDateTime::now_utc() - Duration::minutes(10)).unix_timestamp();

        let tag = manager.generate_tag(
            "id",
            &Money {
                currency_code: String::from("USD"),
                units: 100,
                nanos: 0,
            },
            expiration,
        );

        manager
            .verify_offer(
                "id",
                &Money {
                    currency_code: String::from("USD"),
                    units: 100,
                    nanos: 0,
                },
                expiration,
                &tag,
            )
            .unwrap_err();
    }
}
