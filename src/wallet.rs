use secp256k1::{Secp256k1, SecretKey, PublicKey};
use rand::{RngCore, rngs::OsRng};
use sha2::{Sha256, Digest};

pub fn generate_keypair() -> Result<(SecretKey, PublicKey), secp256k1::Error> {
    let secp = Secp256k1::new();
    let mut rng = OsRng::default();
    let mut secret_key_bytes = [0u8; 32];
    
    // Generate random bytes for the secret key
    rng.fill_bytes(&mut secret_key_bytes);

    // Create a secret key from the random bytes
    let secret_key = SecretKey::from_slice(&secret_key_bytes)?;

    // Derive the public key from the secret key
    let public_key = PublicKey::from_secret_key(&secp, &secret_key);
    
    Ok((secret_key, public_key))
}

pub fn generate_address(public_key: &PublicKey) -> String {
    let serialized_pub_key = public_key.serialize_uncompressed();
    let sha256 = Sha256::digest(&serialized_pub_key[..]);
    format!("{:x}", sha256)
}