use argon2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use base64::{engine::general_purpose::STANDARD as BASE64_STANDARD, Engine};
use rand::rngs::OsRng;
use sha2::{Digest, Sha256};

pub fn hash_password(password: &str) -> Result<String, String> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|err| format!("Falha ao gerar hash da senha: {err}"))
}

pub fn verify_password(password: &str, password_hash: &str) -> Result<bool, String> {
    let parsed_hash = PasswordHash::new(password_hash)
        .map_err(|err| format!("Falha ao interpretar hash armazenado: {err}"))?;

    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

pub fn machine_key() -> String {
    let host = std::env::var("COMPUTERNAME")
        .or_else(|_| std::env::var("HOSTNAME"))
        .unwrap_or_else(|_| "unknown-host".to_string());
    let user = std::env::var("USERNAME")
        .or_else(|_| std::env::var("USER"))
        .unwrap_or_else(|_| "unknown-user".to_string());
    format!("{host}:{user}")
}

fn derive_secret(seed: &str) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(b"ARGWS-PONTO-MANAGER-LIC");
    hasher.update(seed.as_bytes());
    hasher.finalize().to_vec()
}

pub fn encrypt_text(seed: &str, plain: &str) -> String {
    let secret = derive_secret(seed);
    let bytes = plain
        .as_bytes()
        .iter()
        .enumerate()
        .map(|(idx, byte)| byte ^ secret[idx % secret.len()])
        .collect::<Vec<_>>();
    BASE64_STANDARD.encode(bytes)
}

pub fn decrypt_text(seed: &str, cipher: &str) -> Result<String, String> {
    let secret = derive_secret(seed);
    let bytes = BASE64_STANDARD
        .decode(cipher)
        .map_err(|err| format!("Falha ao decodificar conteúdo criptografado: {err}"))?;
    let plain = bytes
        .iter()
        .enumerate()
        .map(|(idx, byte)| byte ^ secret[idx % secret.len()])
        .collect::<Vec<_>>();
    String::from_utf8(plain).map_err(|err| format!("Falha ao reconstruir texto criptografado: {err}"))
}

pub fn integrity_hash(seed: &str, payload: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(seed.as_bytes());
    hasher.update(payload.as_bytes());
    format!("{:x}", hasher.finalize())
}
