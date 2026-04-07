use arbiter_crypto::authn::SigningKey;
use arbiter_proto::home_path;
use std::path::{Path, PathBuf};

#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("I/O error")]
    Io(#[from] std::io::Error),

    #[error("Invalid signing key length in storage: expected {expected} bytes, got {actual} bytes")]
    InvalidKeyLength { expected: usize, actual: usize },
}

pub trait SigningKeyStorage {
    fn load_or_create(&self) -> std::result::Result<SigningKey, StorageError>;
}

#[derive(Debug, Clone)]
pub struct FileSigningKeyStorage {
    path: PathBuf,
}

impl FileSigningKeyStorage {
    pub const DEFAULT_FILE_NAME: &str = "sdk_client_ml_dsa.key";

    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    pub fn from_default_location() -> std::result::Result<Self, StorageError> {
        Ok(Self::new(home_path()?.join(Self::DEFAULT_FILE_NAME)))
    }

    fn read_key(path: &Path) -> std::result::Result<SigningKey, StorageError> {
        let bytes = std::fs::read(path)?;
        let raw: [u8; 32] =
            bytes
                .try_into()
                .map_err(|v: Vec<u8>| StorageError::InvalidKeyLength {
                    expected: 32,
                    actual: v.len(),
                })?;
        Ok(SigningKey::from_seed(raw))
    }
}

impl SigningKeyStorage for FileSigningKeyStorage {
    fn load_or_create(&self) -> std::result::Result<SigningKey, StorageError> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        if self.path.exists() {
            return Self::read_key(&self.path);
        }

        let key = SigningKey::generate();
        let raw_key = key.to_seed();

        // Use create_new to prevent accidental overwrite if another process creates the key first.
        match std::fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&self.path)
        {
            Ok(mut file) => {
                use std::io::Write as _;
                file.write_all(&raw_key)?;
                Ok(key)
            }
            Err(err) if err.kind() == std::io::ErrorKind::AlreadyExists => {
                Self::read_key(&self.path)
            }
            Err(err) => Err(StorageError::Io(err)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{FileSigningKeyStorage, SigningKeyStorage, StorageError};

    fn unique_temp_key_path() -> std::path::PathBuf {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock should be after unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "arbiter-client-key-{}-{}.bin",
            std::process::id(),
            nanos
        ))
    }

    #[test]
    fn file_storage_creates_and_reuses_key() {
        let path = unique_temp_key_path();
        let storage = FileSigningKeyStorage::new(path.clone());

        let key_a = storage
            .load_or_create()
            .expect("first load_or_create should create key");
        let key_b = storage
            .load_or_create()
            .expect("second load_or_create should read same key");

        assert_eq!(key_a.to_seed(), key_b.to_seed());
        assert!(path.exists());

        std::fs::remove_file(path).expect("temp key file should be removable");
    }

    #[test]
    fn file_storage_rejects_invalid_key_length() {
        let path = unique_temp_key_path();
        std::fs::write(&path, [42u8; 31]).expect("should write invalid key file");
        let storage = FileSigningKeyStorage::new(path.clone());

        let err = storage
            .load_or_create()
            .expect_err("storage should reject non-32-byte key file");

        match err {
            StorageError::InvalidKeyLength { expected, actual } => {
                assert_eq!(expected, 32);
                assert_eq!(actual, 31);
            }
            other => panic!("unexpected error: {other:?}"),
        }

        std::fs::remove_file(path).expect("temp key file should be removable");
    }
}
