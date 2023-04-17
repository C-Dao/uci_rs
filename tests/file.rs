use std::{fs::{self, File}, io::Read};

use uci_rs::{load_config, parse_raw_to_uci, save_config, Result, UciCommand};

#[test]
fn test_uci_file_load_config() -> Result<()> {
    let uci = load_config("uci_config", "tests/.test_data")?;
    assert_eq!(uci.get_package(), "uci_config");
    Ok(())
}

#[test]
fn test_uci_file_save_config() -> Result<()> {
    let uci_str = include_str!(".test_data/uci_config");
    let uci = parse_raw_to_uci("uci_config", uci_str.to_string())?;
    save_config(".uci_config.tmp", uci)?;

    let mut file = File::open(".uci_config.tmp/uci_config")?;

    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    assert_eq!(contents.trim_end(), uci_str.trim_end());
    fs::remove_dir_all(".uci_config.tmp")?;
    Ok(())
}
