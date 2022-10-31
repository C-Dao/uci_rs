use std::fs::{self, File, OpenOptions};
use std::io::{BufWriter, Write};
use std::os::unix::fs::OpenOptionsExt;
use std::{env, io};
use std::{io::Read, path::Path};

use super::parser::uci_parse_to_uci;
use super::{imp::UciCommand, Uci};

use crate::utils::{Error, Result};

const DEFAULT_LOAD_DIR: &str = "/etc/config";
const NUM_RETRIES: u32 = 5;

pub fn load_config(name: &str, dir: &str) -> Result<Uci> {
    let load_path = if dir.is_empty() {
        Path::new(DEFAULT_LOAD_DIR).join(name)
    } else {
        Path::new(dir).join(name)
    };
    let mut file = File::open(load_path)?;
    let mut string_buffer = String::new();

    file.read_to_string(&mut string_buffer)?;

    let uci = uci_parse_to_uci(name, string_buffer)?;

    Ok(uci)
}

pub fn save_config(dir: &str, uci: Uci) -> Result<()> {
    let save_dir = if dir.is_empty() {
        Path::new(DEFAULT_LOAD_DIR)
    } else {
        Path::new(dir)
    };
    let mut path = save_dir.join(&uci.get_package());

    if !path.is_absolute() {
        path = env::current_dir()?.join(path)
    }

    let mut open_options = OpenOptions::new();
    open_options.read(true).write(true).create_new(true);
    open_options.mode(0o644);

    for _ in 0..NUM_RETRIES {
        return match open_options.open(&path) {
            Ok(file) => {
                let mut buf = BufWriter::new(file);
                uci.write_in(&mut buf).map(|_| {
                    buf.flush()?;
                    Ok(())
                })?
            }
            Err(err) if err.kind() == io::ErrorKind::NotFound => {
                fs::create_dir(save_dir)?;
                continue;
            }
            Err(ref e) if e.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(ref e) if e.kind() == io::ErrorKind::AddrInUse => continue,
            Err(_) => break,
        };
    }

    Err(Error::new("too many files exist"))
}
#[cfg(test)]
mod test;
