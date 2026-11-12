//! 只有自己能读的文件：按机器的 `settings.json`（代理地址可能带密码）、接口令牌、`daemon.json`、投递令牌。

use std::io::Write;
use std::path::Path;

/// 写一个只有自己能读（unix 上 0600）的文件：先写到旁边的临时文件（建的时候就是 0600）再换名，
/// 读的一方不会读到一半。错误不带路径，由调用方补上。
pub fn write_private(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let tmp = path.with_extension("tmp");
    let mut opts = std::fs::OpenOptions::new();
    opts.write(true).create(true).truncate(true);
    #[cfg(unix)]
    std::os::unix::fs::OpenOptionsExt::mode(&mut opts, 0o600);
    let mut f = opts.open(&tmp)?;
    // 临时文件早就在时 `mode` 不管用，写进字节之前再设一次。
    #[cfg(unix)]
    f.set_permissions(std::os::unix::fs::PermissionsExt::from_mode(0o600))?;
    f.write_all(bytes)?;
    f.sync_all()?;
    std::fs::rename(&tmp, path)
}
