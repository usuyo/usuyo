//! 资料库的目录树、打开和建库（SPEC §1.1）。

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use usuyo_contract::{DeviceRecord, Extra, LibraryFile, Settings};

use crate::fsx::{self, read_json, read_opt, write_json};
use crate::id::uuid7;
use crate::segments::Segments;
use crate::{Error, Result, time};

/// 这一版写的 `formatVersion`。
pub const FORMAT_VERSION: u32 = 1;

const TOP_DIRS: &[&str] = &["items", "topics", "cards", "drafts", "canvas"];
const META_DIRS: &[&str] = &[
    "devices",
    "journal",
    "facts",
    "changes",
    "subscriptions",
    "imports",
    "trash",
    "tmp",
];

/// 按机器的 `libraries/<libraryId>/device.json`。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct MachineDevice {
    device_id: String,
    library_path: PathBuf,
    #[serde(flatten)]
    extra: Extra,
}

/// 打开着的一个资料库；一台机器上一个库一个写方（`.usuyo/lock`）。
#[derive(Debug)]
pub struct Library {
    pub(crate) root: PathBuf,
    machine_dir: PathBuf,
    file: LibraryFile,
    pub(crate) device_id: String,
    pub(crate) settings: Settings,
    pub(crate) journal: Segments,
    pub(crate) facts: Segments,
    read_only: Option<&'static str>,
    previous_path: Option<PathBuf>,
    _lock: Option<fs::File>,
}

impl Library {
    /// 系统的按机器目录（SPEC §0.1）。
    pub fn default_machine_root() -> Result<PathBuf> {
        let env = |k: &str| std::env::var_os(k).map(PathBuf::from);
        let missing = |k: &str| Error::NotFound(format!("environment variable {k}"));
        if cfg!(target_os = "macos") {
            Ok(env("HOME")
                .ok_or_else(|| missing("HOME"))?
                .join("Library/Application Support/usuyo"))
        } else if cfg!(windows) {
            Ok(env("LOCALAPPDATA")
                .ok_or_else(|| missing("LOCALAPPDATA"))?
                .join("usuyo"))
        } else {
            match env("XDG_STATE_HOME") {
                Some(p) => Ok(p.join("usuyo")),
                None => Ok(env("HOME")
                    .ok_or_else(|| missing("HOME"))?
                    .join(".local/state/usuyo")),
            }
        }
    }

    /// 建一个新库；目录里已经有库时回 `conflict`。
    pub fn init(root: &Path, machine_root: &Path) -> Result<Library> {
        let lib_json = root.join(".usuyo/library.json");
        if lib_json.exists() {
            return Err(Error::Conflict(format!(
                "{} is already a library",
                root.display()
            )));
        }
        for d in TOP_DIRS {
            let p = root.join(d);
            fs::create_dir_all(&p).map_err(|e| Error::io(&p, e))?;
        }
        for d in META_DIRS {
            let p = root.join(".usuyo").join(d);
            fs::create_dir_all(&p).map_err(|e| Error::io(&p, e))?;
        }
        write_json(&root.join(".usuyo/settings.json"), &Settings::default())?;
        write_json(
            &lib_json,
            &LibraryFile {
                format_version: FORMAT_VERSION,
                extensions: Extra::new(),
                library_id: uuid7(),
                created_at: time::now(),
                sync: None,
                extra: Extra::new(),
            },
        )?;
        Library::open(root, machine_root)
    }

    /// 打开已有的库。版本比这一版新、有不认识的 `extensions` 时只读打开。
    pub fn open(root: &Path, machine_root: &Path) -> Result<Library> {
        let root = fs::canonicalize(root).map_err(|e| Error::io(root, e))?;
        let file: LibraryFile = read_json(&root.join(".usuyo/library.json"))?;
        let read_only = if file.format_version > FORMAT_VERSION {
            Some("too-new")
        } else if file.extensions.contains_key("migrating") {
            Some("migrating")
        } else if !file.extensions.is_empty() {
            Some("unknown-extension")
        } else {
            None
        };
        let lock = match read_only {
            Some(_) => None,
            None => Some(take_lock(&root)?),
        };
        let settings = match read_opt(&root.join(".usuyo/settings.json"))? {
            Some(b) => fsx::parse_json(&root.join(".usuyo/settings.json"), &b)?,
            None => Settings::default(),
        };
        let machine_dir = machine_root.join("libraries").join(&file.library_id);
        fsx::private_dir(machine_root)?;
        fsx::private_dir(&machine_dir)?;
        let device_path = machine_dir.join("device.json");
        let (device_id, previous_path) = match read_opt(&device_path)? {
            Some(b) => {
                let d: MachineDevice = fsx::parse_json(&device_path, &b)?;
                let moved = (d.library_path != root).then_some(d.library_path);
                (d.device_id, moved)
            }
            None => {
                let d = MachineDevice {
                    device_id: uuid7(),
                    library_path: root.clone(),
                    extra: Extra::new(),
                };
                if read_only.is_none() {
                    write_json(&device_path, &d)?;
                }
                (d.device_id, None)
            }
        };
        let journal = Segments::open(root.join(".usuyo/journal").join(&device_id))?;
        let facts = Segments::open(root.join(".usuyo/facts").join(&device_id))?;
        let lib = Library {
            root,
            machine_dir,
            file,
            device_id,
            settings,
            journal,
            facts,
            read_only,
            previous_path,
            _lock: lock,
        };
        if lib.read_only.is_none() {
            lib.touch_device()?;
            lib.clear_staging()?;
        }
        Ok(lib)
    }

    /// 上次没收尾的暂存：插件调用目录（`usuyo-call-*`）和收下时的拼装目录（UUIDv7）。
    /// 拿着锁时才清，这时不会有调用在用它们；别的（迁移计划）不动。
    fn clear_staging(&self) -> Result<()> {
        for dir in crate::item::read_dirs(&self.path(".usuyo/tmp"))? {
            let name = dir.file_name().and_then(|n| n.to_str()).unwrap_or_default();
            if name.starts_with("usuyo-call-") || crate::id::is_uuid7(name) {
                fsx::remove_dir(&dir)?;
            }
        }
        Ok(())
    }

    fn touch_device(&self) -> Result<()> {
        let path = self
            .root
            .join(".usuyo/devices")
            .join(format!("{}.json", self.device_id));
        let mut rec = match read_opt(&path)? {
            Some(b) => fsx::parse_json::<DeviceRecord>(&path, &b)?,
            None => DeviceRecord {
                name: std::env::consts::OS.to_owned(),
                revoked_at: None,
                last_seen_at: None,
                extra: Extra::new(),
            },
        };
        rec.last_seen_at = Some(time::now());
        write_json(&path, &rec)
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn library_id(&self) -> &str {
        &self.file.library_id
    }

    pub fn device_id(&self) -> &str {
        &self.device_id
    }

    pub fn settings(&self) -> &Settings {
        &self.settings
    }

    /// 本机的 `libraries/<libraryId>/`：索引、`jobs.db`、`sync.db` 归别的 crate 放在这里。
    pub fn machine_dir(&self) -> &Path {
        &self.machine_dir
    }

    /// 只读打开的原因（`too-new`、`migrating`、`unknown-extension`）。
    pub fn read_only(&self) -> Option<&'static str> {
        self.read_only
    }

    /// 同一个 `libraryId` 上次在别的路径打开过：界面问「这是拷贝吗」。
    pub fn previous_path(&self) -> Option<&Path> {
        self.previous_path.as_deref()
    }

    /// 不是拷贝，只是挪了地方：记下新路径。
    pub fn confirm_moved(&mut self) -> Result<()> {
        self.writable()?;
        self.write_machine_device()?;
        self.previous_path = None;
        Ok(())
    }

    /// 是拷贝：换新 `libraryId`、`sync` 置空，免得同步进同一个云端。
    pub fn adopt_as_copy(&mut self) -> Result<()> {
        self.writable()?;
        self.file.library_id = uuid7();
        self.file.sync = None;
        write_json(&self.root.join(".usuyo/library.json"), &self.file)?;
        self.machine_dir = self
            .machine_dir
            .parent()
            .map(|p| p.join(&self.file.library_id))
            .ok_or_else(|| Error::InvalidArgs("machine dir has no parent".into()))?;
        fsx::private_dir(&self.machine_dir)?;
        self.write_machine_device()?;
        self.previous_path = None;
        Ok(())
    }

    fn write_machine_device(&self) -> Result<()> {
        write_json(
            &self.machine_dir.join("device.json"),
            &MachineDevice {
                device_id: self.device_id.clone(),
                library_path: self.root.clone(),
                extra: Extra::new(),
            },
        )
    }

    /// `.usuyo/tmp/` 下一个新的空文件夹：插件的输出目录放这里，收下时改名进 `raw/`，不跨文件系统。
    pub fn staging_dir(&self) -> Result<PathBuf> {
        self.writable()?;
        let p = self.root.join(".usuyo/tmp").join(uuid7());
        fs::create_dir_all(&p).map_err(|e| Error::io(&p, e))?;
        Ok(p)
    }

    pub(crate) fn path(&self, rel: &str) -> PathBuf {
        self.root.join(rel)
    }

    pub(crate) fn writable(&self) -> Result<()> {
        match self.read_only {
            Some(reason) => Err(Error::LibraryVersion { reason }),
            None => Ok(()),
        }
    }

    pub(crate) fn save_settings(&mut self, settings: Settings) -> Result<()> {
        write_json(&self.root.join(".usuyo/settings.json"), &settings)?;
        self.settings = settings;
        Ok(())
    }
}

fn take_lock(root: &Path) -> Result<fs::File> {
    let path = root.join(".usuyo/lock");
    let f = fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .write(true)
        .open(&path)
        .map_err(|e| Error::io(&path, e))?;
    match f.try_lock() {
        Ok(()) => Ok(f),
        Err(fs::TryLockError::WouldBlock) => Err(Error::Busy),
        Err(fs::TryLockError::Error(e)) => Err(Error::io(&path, e)),
    }
}
