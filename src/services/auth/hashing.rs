use sha2::{Digest, Sha512};

pub fn hash(value: &str, secret: &str) -> String {
    let mut hasher = Sha512::new();
    hasher.update(value);
    hasher.update(secret);
    let result = hasher.finalize();
    hex::encode(result)
}
