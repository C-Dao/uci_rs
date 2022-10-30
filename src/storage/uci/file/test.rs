use std::fs;

use super::*;
use crate::utils::Result;

#[test]
fn test_uci_file_load_config() -> Result<()> {
    let uci = load_config("uci_config", "src/storage/uci/test_data")?;
    assert_eq!(uci.get_package(), "uci_config");
    Ok(())
}

#[test]
fn test_uci_file_save_config() -> Result<()> {
    let uci_str = include_str!("../test_data/uci_config");
    let uci = uci_parse_to_uci("uci_config", uci_str.to_string())?;
    save_config(".uci_config.tmp", uci)?;

    let mut file = File::open(".uci_config.tmp/uci_config")?;

    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    assert_eq!(contents, uci_str);
    fs::remove_dir_all(".uci_config.tmp")?;
    Ok(())
}
