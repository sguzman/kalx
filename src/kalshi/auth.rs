use std::fs;
use std::sync::Arc;

use anyhow::{Context, Result};
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use http::HeaderValue;
use reqwest::Method;
use rsa::pkcs1::DecodeRsaPrivateKey;
use rsa::pkcs8::DecodePrivateKey;
use rsa::pss::BlindedSigningKey;
use rsa::rand_core::OsRng;
use rsa::signature::{RandomizedSigner, SignatureEncoding};
use rsa::RsaPrivateKey;
use sha2::Sha256;

#[derive(Clone)]
pub struct Signer {
    api_key_id: String,
    private_key: Arc<RsaPrivateKey>,
}

impl Signer {
    pub fn new(api_key_id: &str, private_key_path: &str) -> Result<Self> {
        let pem = fs::read_to_string(private_key_path)
            .with_context(|| format!("failed to read private key {private_key_path}"))?;
        let private_key = RsaPrivateKey::from_pkcs8_pem(&pem)
            .or_else(|_| RsaPrivateKey::from_pkcs1_pem(&pem))
            .context("failed to decode private key as PKCS#8 or PKCS#1 PEM")?;

        Ok(Self {
            api_key_id: api_key_id.to_string(),
            private_key: Arc::new(private_key),
        })
    }

    pub fn sign_path(&self, timestamp: &str, method: &str, path: &str) -> Result<String> {
        let signing_key = BlindedSigningKey::<Sha256>::new((*self.private_key).clone());
        let payload = format!("{timestamp}{}{path}", method.to_ascii_uppercase());
        let mut rng = OsRng;
        let signature = signing_key.sign_with_rng(&mut rng, payload.as_bytes());
        Ok(STANDARD.encode(signature.to_vec()))
    }

    pub fn auth_headers(&self, timestamp: &str, method: &Method, path: &str) -> Result<Vec<(&'static str, HeaderValue)>> {
        let signature = self.sign_path(timestamp, method.as_str(), path)?;
        Ok(vec![
            ("KALSHI-ACCESS-KEY", HeaderValue::from_str(&self.api_key_id)?),
            ("KALSHI-ACCESS-TIMESTAMP", HeaderValue::from_str(timestamp)?),
            ("KALSHI-ACCESS-SIGNATURE", HeaderValue::from_str(&signature)?),
        ])
    }
}

#[cfg(test)]
mod tests {
    use super::Signer;

    #[test]
    fn strips_query_is_callers_job() {
        let timestamp = "1703123456789";
        let raw = "/trade-api/v2/portfolio/orders";
        let signer = Signer::new(
            "test-key",
            "/tmp/kalx-nonexistent-do-not-call",
        );
        assert!(signer.is_err());
        assert_eq!(raw, "/trade-api/v2/portfolio/orders");
        assert!(!raw.contains('?'));
        assert_eq!(format!("{timestamp}GET{raw}"), "1703123456789GET/trade-api/v2/portfolio/orders");
    }
}
