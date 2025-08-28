use core::convert::TryFrom;
use ecdsa::SignatureSize;
use p256::ecdsa::{signature::hazmat::PrehashVerifier, Signature, VerifyingKey};
use p256::elliptic_curve::generic_array::GenericArray;
use p256::NistP256;
use serde::{Deserialize, Serialize};
use sha2::digest::typenum::Unsigned;
use sha2::{Digest, Sha256};
use std::fmt;
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PublicKey(#[serde(with = "serde_verifying_key")] pub VerifyingKey);

impl Hash for PublicKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.to_encoded_point(true).as_bytes().hash(state);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub source: Option<PublicKey>,
    pub destination: PublicKey,
    pub amount: u64,
    #[serde(with = "serde_signature")]
    pub signature: Option<Signature>,
}

impl Transaction {
    pub fn new(sender_wallet: &super::wallet::Wallet, destination: PublicKey, amount: u64) -> Self {
        let mut tx = Transaction {
            source: Some(PublicKey(sender_wallet.public_key)),
            destination,
            amount,
            signature: None,
        };
        let hash = tx.calculate_hash();
        tx.signature = Some(sender_wallet.sign_prehashed(&hash));
        tx
    }

    pub fn new_coinbase(destination: PublicKey, amount: u64) -> Self {
        Transaction {
            source: None,
            destination,
            amount,
            signature: None,
        }
    }

    pub fn is_valid(&self) -> bool {
        match (&self.source, &self.signature) {
            (Some(source_key), Some(signature)) => {
                let hash = self.calculate_hash();
                source_key.0.verify_prehash(&hash, signature).is_ok()
            }
            (None, None) => true,
            _ => false,
        }
    }

    fn calculate_hash(&self) -> Vec<u8> {
        let mut hasher = Sha256::new();
        let data =
            serde_json::to_vec(&(&self.source, &self.destination, &self.amount)).unwrap();
        hasher.update(data);
        hasher.finalize().to_vec()
    }
}

impl fmt::Display for Transaction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let source_str = match &self.source {
            Some(key) => hex::encode(key.0.to_encoded_point(true)),
            None => "COINBASE (Mining Reward)".to_string(),
        };
        let dest_str = hex::encode(self.destination.0.to_encoded_point(true));
        write!(
            f,
            "  from:   {}...\n  to:     {}...\n  amount: {}",
            &source_str[..10],
            &dest_str[..10],
            self.amount
        )
    }
}

mod serde_verifying_key {
    use super::*;
    use serde::{de::Error, Deserializer, Serializer};

    pub fn serialize<S>(key: &VerifyingKey, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&hex::encode(key.to_encoded_point(true)))
    }
    pub fn deserialize<'de, D>(deserializer: D) -> Result<VerifyingKey, D::Error>
    where
        D: Deserializer<'de>,
    {
        let hex_str = String::deserialize(deserializer)?;
        VerifyingKey::from_sec1_bytes(&hex::decode(hex_str).map_err(Error::custom)?)
            .map_err(Error::custom)
    }
}
mod serde_signature {
    use super::*;
    use serde::{de::Error, Deserializer, Serializer};

    pub fn serialize<S>(sig: &Option<Signature>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match sig {
            Some(s) => serializer.serialize_str(&hex::encode(s.to_bytes())),
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Signature>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let opt_hex_str: Option<String> = Option::deserialize(deserializer)?;
        match opt_hex_str {
            Some(hex_str) => {
                let bytes = hex::decode(hex_str).map_err(Error::custom)?;

                let sig_bytes = <&GenericArray<u8, SignatureSize<NistP256>>>::try_from(
                    &bytes[..],
                )
                .map_err(|_| {
                    Error::custom(format!(
                        "Invalid signature length: expected {}, found {}",
                        SignatureSize::<NistP256>::to_usize(),
                        bytes.len()
                    ))
                })?;

                Signature::from_bytes(sig_bytes).map_err(Error::custom).map(Some)
            }
            None => Ok(None),
        }
    }
}