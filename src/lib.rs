use std::ffi::{CStr, CString};
use std::ops::Drop;
use std::path::Path;

use anyhow::{anyhow, Result};
use thiserror::Error;

#[derive(Debug, Error)]
#[error("Kyoto Cabinet caused an error: {message:?}({code:?})")]
pub struct KyotoCabinetError {
    message: String,
    code: i32,
}

impl KyotoCabinetError {
    pub fn new(db: *mut ffi::KCDB) -> Self {
        unsafe {
            let message = CStr::from_ptr(ffi::kcdbemsg(db))
                .to_str()
                .unwrap_or("???")
                .to_string();

            let code = ffi::kcdbecode(db);

            Self { message, code }
        }
    }
}

pub struct OpenOptions {
    mode: u32,
}

impl OpenOptions {
    pub fn new() -> Self {
        OpenOptions { mode: 0 }
    }

    pub fn read(&mut self, read: bool) -> &mut Self {
        if read {
            self.mode |= ffi::KCOREADER;
        } else {
            self.mode &= !ffi::KCOREADER;
        }

        self
    }

    pub fn write(&mut self, write: bool) -> &mut Self {
        if write {
            self.mode |= ffi::KCOWRITER;
        } else {
            self.mode &= !ffi::KCOWRITER;
        }

        self
    }

    pub fn create(&mut self, create: bool) -> &mut Self {
        if create {
            self.mode |= ffi::KCOCREATE;
        } else {
            self.mode &= !ffi::KCOCREATE;
        }

        self
    }

    pub fn truncate(&mut self, truncate: bool) -> &mut Self {
        if truncate {
            self.mode |= ffi::KCOTRUNCATE;
        } else {
            self.mode &= !ffi::KCOTRUNCATE;
        }

        self
    }

    pub fn open<P: AsRef<Path>>(&self, path: P) -> Result<KyotoCabinet> {
        unsafe {
            let path = path
                .as_ref()
                .to_str()
                .ok_or_else(|| anyhow!("{}", path.as_ref().display()))
                .and_then(|x| CString::new(x).map_err(|e| anyhow!(e)))?;

            let db = ffi::kcdbnew();
            let ok = ffi::kcdbopen(db, path.as_ptr(), self.mode);

            match ok {
                0 => Err(KyotoCabinetError::new(db).into()),
                _ => Ok(KyotoCabinet { db }),
            }
        }
    }
}

#[derive(Debug)]
pub struct KyotoCabinet {
    db: *mut ffi::KCDB,
}

impl KyotoCabinet {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        OpenOptions::new().read(true).open(path)
    }

    pub fn create<P: AsRef<Path>>(path: P) -> Result<Self> {
        OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)
    }

    pub fn close(&mut self) -> Result<()> {
        unsafe {
            if self.db.is_null() {
                return Ok(());
            }

            let ok = ffi::kcdbclose(self.db);

            let result = match (ok, self.err()) {
                (_, Some(e)) => Err(e.into()),
                _ => {
                    ffi::kcdbdel(self.db);
                    self.db = std::ptr::null_mut();
                    Ok(())
                }
            };

            result
        }
    }

    pub fn err(&self) -> Option<KyotoCabinetError> {
        let err = KyotoCabinetError::new(self.db);

        match err.code as _ {
            ffi::KCESUCCESS => None,
            _ => Some(err),
        }
    }

    pub fn set(&self, key: &[u8], val: &[u8]) -> Result<()> {
        unsafe {
            let ok = ffi::kcdbset(
                self.db,
                key.as_ptr() as _,
                key.len(),
                val.as_ptr() as _,
                val.len(),
            );

            let result = match (ok, self.err()) {
                (_, Some(e)) => Err(e.into()),
                _ => Ok(()),
            };

            result
        }
    }

    pub fn get(&self, key: &[u8]) -> Option<Vec<u8>> {
        unsafe {
            let mut len = 0usize;
            let ptr = ffi::kcdbget(self.db, key.as_ptr() as _, key.len(), &mut len);

            if ptr.is_null() {
                return None;
            }

            let mut val: Vec<u8> = Vec::with_capacity(len);
            val.set_len(len);

            std::ptr::copy(ptr as _, val.as_mut_ptr(), len);
            ffi::kcfree(ptr as _);

            Some(val)
        }
    }
}

impl Drop for KyotoCabinet {
    fn drop(&mut self) {
        let _ = self.close();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn get() -> Result<()> {
        let dir = tempdir()?;
        let path = dir.path().join("test.kch");

        let db = KyotoCabinet::create(&path)?;

        let key = b"key";
        let val = b"val";

        db.set(key, val)?;

        assert_eq!(&db.get(key).ok_or(anyhow!(""))?, val);

        Ok(())
    }
}
