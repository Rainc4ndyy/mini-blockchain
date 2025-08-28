use anyhow::Result;
use core::convert::TryFrom;
use p256::ecdsa::{signature::hazmat::PrehashSigner, Signature, SigningKey, VerifyingKey};
use p256::elliptic_curve::consts::U32;
use p256::elliptic_curve::generic_array::GenericArray;
use rand::rngs::OsRng;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug, Serialize, Deserialize)]
pub struct Wallet {
    #[serde(serialize_with = "serialize_key", deserialize_with = "deserialize_key")]
    signing_key: SigningKey,
    pub public_key: VerifyingKey,
}

impl Wallet {
    pub fn new() -> Self {
        let signing_key = SigningKey::random(&mut OsRng);
        let public_key = *signing_key.verifying_key();
        Wallet {
            signing_key,
            public_key,
        }
    }

    pub fn sign_prehashed(&self, hash: &[u8]) -> Signature {
        self.signing_key.sign_prehash(hash).unwrap()
    }
}

fn serialize_key<S>(key: &SigningKey, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&hex::encode(key.to_bytes()))
}

fn deserialize_key<'de, D>(deserializer: D) -> Result<SigningKey, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error;
    let hex_str = String::deserialize(deserializer)?;
    let bytes = hex::decode(hex_str).map_err(Error::custom)?;

    let key_bytes = <&GenericArray<u8, U32>>::try_from(&bytes[..]).map_err(|_| {
        Error::custom(format!(
            "This doesn't look like a valid 32-byte private key. Length was {}.",
            bytes.len()
        ))
    })?;

    SigningKey::from_bytes(key_bytes).map_err(Error::custom)
}