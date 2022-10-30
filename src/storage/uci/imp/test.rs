use std::{
    fs::{File, OpenOptions, self},
    io::Read,
    os::unix::prelude::OpenOptionsExt,
};

use crate::storage::uci::parser::uci_parse_to_uci;

use super::*;

#[test]
fn test_uci_add_section() -> Result<()> {
    let mut uci = Uci::new("test");
    uci.add_section("a", "b")?;
    let sec = uci.get_section("b")?;
    assert_eq!(sec, Some(("a".to_string(), "b".to_string())));
    Ok(())
}

#[test]
fn test_uci_del_section() -> Result<()> {
    let mut uci = Uci::new("test");
    uci.add_section("ab", "bb")?;
    let sec = uci.get_section("bb")?;
    assert_eq!(sec, Some(("ab".to_string(), "bb".to_string())));
    uci.del_section("bb")?;
    let sec = uci.get_section("bb")?;
    assert_eq!(sec, None);
    Ok(())
}

#[test]
fn test_uci_set_option() -> Result<()> {
    let mut uci = Uci::new("test");
    uci.add_section("ab", "bb")?;
    uci.set_option("bb", "cc", vec!["dd"])?;
    let opt = uci.get_option("bb", "cc")?;
    assert_eq!(opt, ("cc".to_string(), &vec!["dd".to_string()]));
    Ok(())
}

#[test]
fn test_uci_get_all_options() -> Result<()> {
    let mut uci = Uci::new("test");
    uci.add_section("ab", "bb")?;
    uci.set_option("bb", "cc", vec!["dd"])?;
    uci.set_option("bb", "dd", vec!["ee"])?;
    uci.set_option("bb", "ee", vec!["ff"])?;
    let opts = uci.get_all_options("bb")?;
    assert_eq!(
        opts,
        vec![
            ("cc".to_string(), &vec!["dd".to_string()]),
            ("dd".to_string(), &vec!["ee".to_string()]),
            ("ee".to_string(), &vec!["ff".to_string()])
        ]
    );
    Ok(())
}

#[test]
fn test_uci_get_option_last() -> Result<()> {
    let mut uci = Uci::new("test");
    uci.add_section("ab", "bb")?;
    uci.set_option("bb", "cc", vec!["dd", "ee", "ff", "gg"])?;
    let opt = uci.get_option_last("bb", "cc")?;
    assert_eq!(opt, ("cc".to_string(), Some("gg".to_string())));
    Ok(())
}

#[test]
fn test_uci_get_option_first() -> Result<()> {
    let mut uci = Uci::new("test");
    uci.add_section("ab", "bb")?;
    uci.set_option("bb", "cc", vec!["dd", "ee", "ff", "gg"])?;
    let opt = uci.get_option_first("bb", "cc")?;
    assert_eq!(opt, ("cc".to_string(), Some("dd".to_string())));
    Ok(())
}

#[test]
fn test_uci_is_bool_value() -> Result<()> {
    let uci = Uci::new("test");
    assert!(uci.is_bool_value("true"));
    assert!(uci.is_bool_value("1"));
    assert!(uci.is_bool_value("on"));
    assert!(uci.is_bool_value("yes"));
    assert!(uci.is_bool_value("enabled"));
    assert!(!uci.is_bool_value("0"));
    assert!(!uci.is_bool_value("false"));
    assert!(!uci.is_bool_value("disabled"));
    Ok(())
}

#[test]
fn test_uci_get_section() -> Result<()> {
    let mut uci = Uci::new("test");
    uci.add_section("ab", "bb")?;
    let sec = uci.get_section("bb")?;
    assert_eq!(sec, Some(("ab".to_string(), "bb".to_string())));
    Ok(())
}

#[test]
fn test_uci_get_all_sections() -> Result<()> {
    let mut uci = Uci::new("test");
    uci.add_section("ab", "bb")?;
    uci.add_section("ss", "cc")?;
    uci.add_section("ww", "dd")?;
    uci.add_section("qq", "ee")?;
    let secs = uci.get_all_sections();
    assert_eq!(
        secs,
        vec![
            ("ab".to_string(), "bb".to_string()),
            ("ss".to_string(), "cc".to_string()),
            ("ww".to_string(), "dd".to_string()),
            ("qq".to_string(), "ee".to_string())
        ]
    );
    Ok(())
}

#[test]
fn test_uci_del_all() -> Result<()> {
    let mut uci = Uci::new("test");
    uci.add_section("ab", "bb")?;
    uci.add_section("ab", "cc")?;
    uci.add_section("ab", "dd")?;
    uci.add_section("ab", "ee")?;
    uci.del_all("ab")?;
    let secs = uci.get_all("ab");
    assert_eq!(secs, vec![]);
    Ok(())
}

#[test]
fn test_uci_get_section_first() -> Result<()> {
    let mut uci = Uci::new("test");
    uci.add_section("ab", "bb")?;
    uci.add_section("ab", "cc")?;
    uci.add_section("ab", "dd")?;
    uci.add_section("ab", "ee")?;
    if let Some(sec) = uci.get_section_first("ab") {
        assert_eq!(sec, ("ab".to_string(), "bb".to_string()));
    };
    Ok(())
}

#[test]
fn test_uci_get_section_last() -> Result<()> {
    let mut uci = Uci::new("test");
    uci.add_section("ab", "bb")?;
    uci.add_section("ab", "cc")?;
    uci.add_section("ab", "dd")?;
    uci.add_section("ab", "ee")?;
    if let Some(sec) = uci.get_section_last("ab") {
        assert_eq!(sec, ("ab".to_string(), "ee".to_string()));
    };
    Ok(())
}

#[test]
fn test_uci_set_package() -> Result<()> {
    let mut uci = Uci::new("test");
    uci.set_package("ab")?;
    assert_eq!("ab", uci.get_package());
    Ok(())
}

#[test]
fn test_uci_for_each() -> Result<()> {
    let mut uci = Uci::new("test");
    uci.add_section("ab", "bb")?;
    uci.add_section("ab", "cc")?;
    uci.add_section("ab", "dd")?;
    uci.add_section("ab", "ee")?;

    let mut res = vec![];
    uci.for_each("ab", |sec| {
        res.push(sec.name.to_string());
    });
    assert_eq!(res, vec!["bb", "cc", "dd", "ee"]);
    Ok(())
}

#[test]
fn test_uci_write_in() -> Result<()> {
    let uci_str = include_str!("../test_data/uci_config");
    let uci = uci_parse_to_uci("uci_config", uci_str.to_string())?;

    let mut open_options = OpenOptions::new();

    open_options.read(true).write(true).create_new(true);
    open_options.mode(0o644);
    let file = open_options.open(".write_in_uci_config.tmp")?;
    let mut buf = BufWriter::new(file);
    uci.write_in(&mut buf)?;
    buf.flush()?;
    let mut file = File::open(".write_in_uci_config.tmp")?;

    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    assert_eq!(contents, uci_str);
    fs::remove_file(".write_in_uci_config.tmp")?;
    Ok(())
}
