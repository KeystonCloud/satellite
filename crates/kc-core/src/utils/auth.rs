use argon2::{Algorithm, Argon2, Params, PasswordHash, PasswordHasher, PasswordVerifier, Version};
use password_hash::{Error, SaltString, rand_core::OsRng};
use thiserror::Error;
use tokio::task;

#[derive(Debug, Error)]
pub enum HashError {
    #[error("Erreur de hachage (interne)")]
    HashFailed(#[from] Error),
    #[error("Erreur de thread Tokio")]
    TaskFailed(#[from] task::JoinError),
}

pub async fn hash_password(password: String) -> Result<String, HashError> {
    let hash_string = task::spawn_blocking(move || -> Result<String, Error> {
        let salt = SaltString::generate(&mut OsRng);
        let params = Params::new(19456, 2, 1, None).unwrap();
        let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
        let hash = argon2.hash_password(password.as_bytes(), &salt)?;

        Ok(hash.to_string())
    })
    .await??;

    Ok(hash_string)
}

pub async fn verify_password(password: String, hash_str: String) -> Result<bool, HashError> {
    let is_valid = task::spawn_blocking(move || -> Result<bool, Error> {
        let parsed_hash = PasswordHash::new(&hash_str)?;
        let argon2 = Argon2::default();

        match argon2.verify_password(password.as_bytes(), &parsed_hash) {
            Ok(_) => Ok(true),
            Err(Error::Password) => Ok(false),
            Err(e) => Err(e),
        }
    })
    .await??;

    Ok(is_valid)
}
