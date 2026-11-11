//! SHA-256（SPEC §0.7）：裸值是小写十六进制，通用哈希字段写 `sha256:<hex>`。

use sha2::{Digest, Sha256};

/// 小写十六进制。
pub fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut out = String::with_capacity(64);
    for b in digest {
        out.push_str(&format!("{b:02x}"));
    }
    out
}

/// `sha256:<hex>`。
pub fn hash_tag(bytes: &[u8]) -> String {
    format!("sha256:{}", sha256_hex(bytes))
}

/// 一个文件的大小和 SHA-256，按固定大小的块读，不把整个文件读进内存。
pub(crate) fn sha256_file(path: &std::path::Path) -> crate::Result<(u64, String)> {
    use std::io::Read;
    let io = |e| crate::Error::io(path, e);
    let mut file = std::fs::File::open(path).map_err(io)?;
    let mut hasher = Sha256::new();
    let mut buf = vec![0u8; 1 << 20];
    let mut size = 0u64;
    loop {
        let n = file.read(&mut buf).map_err(io)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
        size += n as u64;
    }
    let mut out = String::with_capacity(64);
    for b in hasher.finalize() {
        out.push_str(&format!("{b:02x}"));
    }
    Ok((size, out))
}

pub(crate) fn sha256_bytes(bytes: &[u8]) -> [u8; 32] {
    Sha256::digest(bytes).into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_hash_reads_in_blocks() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("f");
        let bytes: Vec<u8> = (0..(3 << 20) + 7).map(|i| (i % 251) as u8).collect();
        std::fs::write(&path, &bytes).unwrap();
        assert_eq!(
            sha256_file(&path).unwrap(),
            (bytes.len() as u64, sha256_hex(&bytes))
        );
    }

    #[test]
    fn empty_input() {
        assert_eq!(
            hash_tag(b""),
            "sha256:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }
}
