use std::io::Cursor;

use hmac::{digest::MacError, Hmac, Mac};
use prost::{DecodeError, Message};
use sha2::Sha256;
use thiserror::Error;

use crate::proto::salesvc::{Token, TokenData};

pub type HmacSha256 = Hmac<Sha256>;
pub type Key = Vec<u8>;

pub struct TokenManager {
    secret: Key,
}

impl TokenManager {
    pub fn new(secret: Key) -> Self {
        Self { secret }
    }

    pub fn generate_token(&self, data: TokenData) -> Token {
        let data = data.encode_to_vec();

        let mut mac = HmacSha256::new_from_slice(&self.secret).unwrap();
        mac.update(&data);
        let result = mac.finalize();
        let tag = result.into_bytes().to_vec();

        Token { data, tag }
    }

    pub fn verify_token(&self, token: Token) -> Result<TokenData, TokenVerifyError> {
        let mut mac = HmacSha256::new_from_slice(&self.secret).unwrap();
        mac.update(&token.data);
        mac.verify_slice(&token.tag)?;

        let data = TokenData::decode(Cursor::new(token.data))?;

        Ok(data)
    }
}

#[derive(Error, Debug)]
pub enum TokenVerifyError {
    #[error("Invalid token signature: {0}")]
    InvalidSignature(#[from] MacError),

    #[error("Invalid token data: {0}")]
    InvalidData(#[from] DecodeError),
}

impl From<TokenVerifyError> for tonic::Status {
    fn from(_: TokenVerifyError) -> tonic::Status {
        tonic::Status::invalid_argument("offer is invalid")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        let manager = TokenManager::new(b"secret".to_vec());

        let data = TokenData {
            ..Default::default()
        };

        let token = manager.generate_token(data.clone());
        let result = manager.verify_token(token).unwrap();

        assert_eq!(data.flight_id, result.flight_id);
    }

    #[test]
    fn test_invalid_data() {
        let manager = TokenManager::new(b"secret".to_vec());

        let data = TokenData {
            ..Default::default()
        };

        let mut token = manager.generate_token(data.clone());

        token.data = TokenData {
            flight_id: String::from("idk"),
            ..Default::default()
        }
        .encode_to_vec();

        let _ = manager.verify_token(token).unwrap_err();
    }
}
