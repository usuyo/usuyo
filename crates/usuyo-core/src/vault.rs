//! 凭据和密钥存在哪（协议 §6、§2.11）。
//!
//! - macOS、Windows：系统钥匙串（keyring），服务名 `usuyo`，账户名是「按机器目录的短哈希/键」
//!   （键是 `site:<站点>`、`secret:<插件>:<设置>`）：钥匙串按系统用户，不按 `USUYO_HOME`，
//!   开发时另开的目录不会盖掉正式那份。
//! - 别的系统（Linux 还没接 Secret Service）：按机器目录里的 `secrets.json`，0600，只有自己能读。
//!   这是替身，不是钥匙串：文件读权限挡得住别的用户，挡不住以你的身份跑的程序。

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, PoisonError};

use usuyo_plugin::{PluginError, Vault};

/// 这台机器上该用的那个。
pub fn system_vault(machine_root: &Path) -> Arc<dyn Vault> {
    #[cfg(any(target_os = "macos", target_os = "windows"))]
    {
        let root = machine_root
            .canonicalize()
            .unwrap_or_else(|_| machine_root.to_owned());
        let hash = usuyo_library::sha256_hex(root.to_string_lossy().as_bytes());
        Arc::new(Keychain {
            scope: hash[..12].to_owned(),
        })
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        Arc::new(FileVault::new(machine_root.join("secrets.json")))
    }
}

/// 系统钥匙串。
#[cfg(any(target_os = "macos", target_os = "windows"))]
#[derive(Debug)]
pub struct Keychain {
    scope: String,
}

#[cfg(any(target_os = "macos", target_os = "windows"))]
const SERVICE: &str = "usuyo";

#[cfg(any(target_os = "macos", target_os = "windows"))]
impl Keychain {
    fn entry(&self, key: &str) -> Result<keyring::Entry, PluginError> {
        keyring::Entry::new(SERVICE, &format!("{}/{key}", self.scope))
            .map_err(|e| PluginError::Vault(format!("keychain: {e}")))
    }
}

#[cfg(any(target_os = "macos", target_os = "windows"))]
impl Vault for Keychain {
    fn read(&self, key: &str) -> Result<Option<String>, PluginError> {
        match self.entry(key)?.get_password() {
            Ok(v) => Ok(Some(v)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(PluginError::Vault(format!("keychain read: {e}"))),
        }
    }

    fn write(&self, key: &str, value: &str) -> Result<(), PluginError> {
        self.entry(key)?
            .set_password(value)
            .map_err(|e| PluginError::Vault(format!("keychain write: {e}")))
    }

    fn remove(&self, key: &str) -> Result<(), PluginError> {
        match self.entry(key)?.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(e) => Err(PluginError::Vault(format!("keychain remove: {e}"))),
        }
    }
}

/// 没有钥匙串服务时的替身：一个 0600 的 JSON 文件。也是 [`crate::EngineConfig::new`] 的默认，
/// 测试和临时目录里的引擎不碰系统钥匙串。
#[derive(Debug)]
pub struct FileVault {
    path: PathBuf,
    lock: Mutex<()>,
}

impl FileVault {
    pub fn new(path: PathBuf) -> FileVault {
        FileVault {
            path,
            lock: Mutex::new(()),
        }
    }

    fn load(&self) -> Result<BTreeMap<String, String>, PluginError> {
        match std::fs::read(&self.path) {
            Ok(b) => serde_json::from_slice(&b).map_err(|e| {
                PluginError::Vault(format!("{} is not readable: {e}", self.path.display()))
            }),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(BTreeMap::new()),
            Err(source) => Err(PluginError::Io {
                path: self.path.clone(),
                source,
            }),
        }
    }

    fn save(&self, map: &BTreeMap<String, String>) -> Result<(), PluginError> {
        use std::io::Write;
        let io = |path: &Path, source| PluginError::Io {
            path: path.to_owned(),
            source,
        };
        if let Some(dir) = self.path.parent() {
            std::fs::create_dir_all(dir).map_err(|e| io(dir, e))?;
        }
        let bytes = serde_json::to_vec(map).map_err(|e| PluginError::Vault(e.to_string()))?;
        let tmp = self.path.with_extension("tmp");
        let mut opts = std::fs::OpenOptions::new();
        opts.write(true).create(true).truncate(true);
        #[cfg(unix)]
        std::os::unix::fs::OpenOptionsExt::mode(&mut opts, 0o600);
        let mut f = opts.open(&tmp).map_err(|e| io(&tmp, e))?;
        #[cfg(unix)]
        f.set_permissions(std::os::unix::fs::PermissionsExt::from_mode(0o600))
            .map_err(|e| io(&tmp, e))?;
        f.write_all(&bytes).map_err(|e| io(&tmp, e))?;
        std::fs::rename(&tmp, &self.path).map_err(|e| io(&self.path, e))
    }
}

impl Vault for FileVault {
    fn read(&self, key: &str) -> Result<Option<String>, PluginError> {
        let _guard = self.lock.lock().unwrap_or_else(PoisonError::into_inner);
        Ok(self.load()?.remove(key))
    }

    fn write(&self, key: &str, value: &str) -> Result<(), PluginError> {
        let _guard = self.lock.lock().unwrap_or_else(PoisonError::into_inner);
        let mut map = self.load()?;
        map.insert(key.into(), value.into());
        self.save(&map)
    }

    fn remove(&self, key: &str) -> Result<(), PluginError> {
        let _guard = self.lock.lock().unwrap_or_else(PoisonError::into_inner);
        let mut map = self.load()?;
        if map.remove(key).is_some() {
            self.save(&map)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_vault_is_private_and_round_trips() {
        let dir = tempfile::tempdir().unwrap();
        let v = FileVault::new(dir.path().join("secrets.json"));
        assert_eq!(v.read("a").unwrap(), None);
        v.write("a", "1").unwrap();
        assert_eq!(v.read("a").unwrap().as_deref(), Some("1"));
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = std::fs::metadata(dir.path().join("secrets.json"))
                .unwrap()
                .permissions()
                .mode();
            assert_eq!(mode & 0o777, 0o600);
        }
        v.remove("a").unwrap();
        v.remove("a").unwrap();
        assert_eq!(v.read("a").unwrap(), None);
    }
}
