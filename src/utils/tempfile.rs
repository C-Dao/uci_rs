use std::env;
use std::ffi::{OsString};
use std::fmt;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::iter::repeat_with;
use std::mem;
use std::path::{Path, PathBuf};

use std::os::unix::fs::OpenOptionsExt;

use super::error::PathError;
use super::error::PersistError;
use super::Result;

const NUM_RETRIES: u32 = 1 << 31;
const NUM_RAND_CHARS: usize = 6;

fn create_named(mut path: PathBuf, open_options: &mut OpenOptions) -> io::Result<TempFile> {
    if !path.is_absolute() {
        path = env::current_dir()?.join(path)
    }
    open_options.read(true).write(true).create_new(true);
    open_options.mode(0o600);
    open_options.open(&path).map(|file| TempFile {
        path: path.into_boxed_path(),
        file,
    })
}

fn persist(old_path: &Path, new_path: &Path, overwrite: bool) -> io::Result<()> {
    if overwrite {
        fs::rename(old_path, new_path)?;
    } else {
        fs::hard_link(old_path, new_path)?;
        fs::remove_file(old_path)?;
    }
    Ok(())
}

#[derive(Clone, Eq, PartialEq)]
pub struct TempFile<F = File> {
    pub path: Box<Path>,
    pub file: F,
}

impl TempFile<File> {
    const RANDOM_LEN: usize = NUM_RAND_CHARS;
    const PREFIX: &'static str = ".tmp";
    const SUFFIX: &'static str = "";
    const APPEND: bool = false;
    pub fn new<P: AsRef<Path>>(dir: P) -> io::Result<TempFile> {
        let num_retries = if Self::RANDOM_LEN != 0 {
            NUM_RETRIES
        } else {
            1
        };

        for _ in 0..num_retries {
            let path = dir.as_ref().join(Self::tmp_name());
            return match create_named(path, OpenOptions::new().append(Self::APPEND)) {
                Err(ref e) if e.kind() == io::ErrorKind::AlreadyExists => continue,
                Err(ref e) if e.kind() == io::ErrorKind::AddrInUse => continue,
                res => res,
            };
        }

        Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "too many temporary files exist",
        ))
    }

    fn tmp_name() -> OsString {
        let mut buf =
            OsString::with_capacity(Self::PREFIX.len() + Self::SUFFIX.len() + Self::RANDOM_LEN);
        buf.push(Self::PREFIX);
        let mut char_buf = [0u8; 4];
        for c in repeat_with(fastrand::alphanumeric).take(Self::RANDOM_LEN) {
            buf.push(c.encode_utf8(&mut char_buf));
        }
        buf.push(Self::SUFFIX);
        buf
    }
}

impl<F> TempFile<F> {
    pub fn close(mut self) -> Result<()> {
        let result = fs::remove_file(&self.path).map_err(|err| PathError {
            path: self.path.clone().into(),
            error: err,
        })?;
        self.path = PathBuf::new().into_boxed_path();
        mem::forget(self);
        Ok(result)
    }

    pub fn persist<P: AsRef<Path>>(mut self, new_path: P) -> Result<()> {
        let TempFile { ref path, ref file } = self;
        match persist(&self.path, new_path.as_ref(), true) {
            Ok(_) => {
                self.path = PathBuf::new().into_boxed_path();
                mem::forget(self);
                Ok(())
            }
            Err(error) => Err(PersistError {
                file: TempFile {
                    path: path.clone(),
                    file,
                },
                error,
            }
            .into()),
        }
    }

    pub fn as_file(&self) -> &F {
        &self.file
    }

    pub fn as_file_mut(&mut self) -> &mut F {
        &mut self.file
    }
}

impl<F: Read> Read for TempFile<F> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.as_file_mut().read(buf).map_err(|error| {
            PathError {
                path: self.path.clone().into_path_buf(),
                error: error,
            }
            .into()
        })
    }
}

impl<'a, F> Read for &'a TempFile<F>
where
    &'a F: Read,
{
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.as_file().read(buf).map_err(|error| {
            PathError {
                path: self.path.clone().into_path_buf(),
                error: error,
            }
            .into()
        })
    }
}

impl<F: Write> Write for TempFile<F> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.as_file_mut().write(buf).map_err(|error| {
            PathError {
                path: self.path.clone().into_path_buf(),
                error: error,
            }
            .into()
        })
    }
    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        self.as_file_mut().flush().map_err(|error| {
            PathError {
                path: self.path.clone().into_path_buf(),
                error: error,
            }
            .into()
        })
    }
}

impl<'a, F> Write for &'a TempFile<F>
where
    &'a F: Write,
{
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.as_file().write(buf).map_err(|error| {
            PathError {
                path: self.path.clone().into_path_buf(),
                error: error,
            }
            .into()
        })
    }
    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        self.as_file().flush().map_err(|error| {
            PathError {
                path: self.path.clone().into_path_buf(),
                error: error,
            }
            .into()
        })
    }
}

impl<F: Seek> Seek for TempFile<F> {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.as_file_mut().seek(pos).map_err(|error| {
            PathError {
                path: self.path.clone().into_path_buf(),
                error: error,
            }
            .into()
        })
    }
}

impl<'a, F> Seek for &'a TempFile<F>
where
    &'a F: Seek,
{
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.as_file().seek(pos).map_err(|error| {
            PathError {
                path: self.path.clone().into_path_buf(),
                error: error,
            }
            .into()
        })
    }
}

impl<F> std::os::unix::io::AsRawFd for TempFile<F>
where
    F: std::os::unix::io::AsRawFd,
{
    #[inline]
    fn as_raw_fd(&self) -> std::os::unix::io::RawFd {
        self.as_file().as_raw_fd()
    }
}

impl<F> Drop for TempFile<F> {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

impl<F> fmt::Debug for TempFile<F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "TempFile({:?})", self.path)
    }
}

impl<F> AsRef<Path> for TempFile<F> {
    #[inline]
    fn as_ref(&self) -> &Path {
        &self.path
    }
}
