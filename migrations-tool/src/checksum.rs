use blake2::{Blake2b, Digest, digest::consts::U32};

type Blake2b256 = Blake2b<U32>;

/// Compute a BLAKE2b-256 checksum over the UTF-8 bytes of `content`.
///
/// Returns a 64-character lowercase hex string.
pub(crate) fn compute(content: &str) -> String {
    let mut hasher = Blake2b256::new();
    hasher.update(content.as_bytes());
    let result = hasher.finalize();
    result.iter().fold(String::with_capacity(64), |mut s, b| {
        use std::fmt::Write as _;
        let _ = write!(s, "{b:02x}");
        s
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic() {
        let a = compute("DEFINE TABLE foo;");
        let b = compute("DEFINE TABLE foo;");
        assert_eq!(a, b);
    }

    #[test]
    fn different_content_differs() {
        let a = compute("DEFINE TABLE foo;");
        let b = compute("DEFINE TABLE bar;");
        assert_ne!(a, b);
    }

    #[test]
    fn length_is_64() {
        assert_eq!(compute("x").len(), 64);
    }
}
