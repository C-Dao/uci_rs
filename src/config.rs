use std::fs::File;
use std::io::BufWriter;
use std::{io::Read, path::Path};

use super::parser::parse_raw_to_uci;
use crate::file::TempFile;
use crate::imp::{Uci, UciCommand};
use crate::utils::Result;

const DEFAULT_LOAD_DIR: &str = "/etc/config";

pub fn load_config(name: &str, dir: &str) -> Result<Uci> {
    let load_path = if dir.is_empty() {
        Path::new(DEFAULT_LOAD_DIR).join(name)
    } else {
        Path::new(dir).join(name)
    };
    let mut file = File::open(load_path)?;
    let mut string_buffer = String::new();

    file.read_to_string(&mut string_buffer)?;

    let uci = parse_raw_to_uci(name, string_buffer)?;

    Ok(uci)
}

pub fn save_config(dir: &str, uci: Uci) -> Result<()> {
    let save_dir = if dir.is_empty() {
        Path::new(DEFAULT_LOAD_DIR)
    } else {
        Path::new(dir)
    };

    let temp_file = TempFile::new(save_dir, uci.get_package())?;

    let mut buf = BufWriter::new(temp_file);

    match uci.write_in(&mut buf) {
        Ok(()) => {
            let mut temp_file = buf.into_inner()?;
            temp_file.as_file_mut().sync_all()?;
            temp_file.persist(save_dir.join(&uci.get_package()))?;
            Ok(())
        }
        Err(err) => {
            let temp_file = buf.into_inner()?;
            temp_file.close()?;
            Err(err)
        }
    }
}
