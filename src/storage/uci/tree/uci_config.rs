use std::io::{BufWriter, Write};
use std::str::from_utf8;

use super::{uci_option::UciOptionType, uci_section::UciSection};
use crate::utils::tempfile::TempFile;
use crate::utils::{Error, Result};

#[derive(Clone, Debug)]
pub struct UciConfig {
    pub name: String,
    pub pkg_name: String,
    pub sections: Vec<UciSection>,
    pub modified: bool,
}

impl UciConfig {
    pub fn new(name: &str) -> UciConfig {
        UciConfig {
            name: name.to_string(),
            pkg_name: name.to_string(),
            sections: Vec::new(),
            modified: false,
        }
    }

    fn _index(&self, section: &UciSection) -> Option<usize> {
        self.sections
            .iter()
            .filter(|sec| sec.sec_type == section.sec_type)
            .position(|sec| sec == section)
    }

    fn _get_named(&self, name: &str) -> Result<Option<&UciSection>> {
        Ok(self.sections.iter().find(|section| section.name == name))
    }

    fn _get_named_mut(&mut self, name: &str) -> Result<Option<&mut UciSection>> {
        Ok(self
            .sections
            .iter_mut()
            .find(|section| section.name == name))
    }

    fn _get_unnamed(&self, name: &str) -> Result<Option<&UciSection>> {
        let (sec_type, sec_index) = unmangle_section_name(name)?;
        let count = self._count(&sec_type);
        let index = if sec_index >= 0 {
            sec_index as i32
        } else {
            count as i32 + sec_index
        };

        if index < 0 || index >= count as i32 {
            return Err(Error::new("invalid name: index out of bounds".to_string()));
        };

        let section = self
            .sections
            .iter()
            .filter(|sec| sec.sec_type == sec_type)
            .map(|sec| sec)
            .nth(index as usize);

        Ok(section)
    }

    fn _get_unnamed_mut(&mut self, name: &str) -> Result<Option<&mut UciSection>> {
        let (sec_type, sec_index) = unmangle_section_name(name)?;
        let count = self._count(&sec_type);
        let index = if sec_index >= 0 {
            sec_index as i32
        } else {
            count as i32 + sec_index
        };

        if index < 0 || index >= count as i32 {
            return Err(Error::new("invalid name: index out of bounds".to_string()));
        };

        let section = self
            .sections
            .iter_mut()
            .filter(|sec| sec.sec_type == sec_type)
            .map(|sec| sec)
            .nth(index as usize);

        Ok(section)
    }

    fn _count(&self, sec_type: &str) -> usize {
        self.sections
            .iter()
            .filter(|sec| sec.sec_type == sec_type)
            .collect::<Vec<&UciSection>>()
            .len()
    }

    pub(crate) fn set_pkg_name(&mut self, name: String) {
        self.pkg_name = name;
    }

    pub fn write_in(&self, file: &mut TempFile) -> Result<()> {
        let mut buf = BufWriter::new(file);

        if self.pkg_name != "" {
            buf.write_fmt(format_args!("\npackage '{}'\n", self.pkg_name))?;
        }

        for sec in self.sections.iter() {
            if sec.name == "" {
                buf.write_fmt(format_args!("\nconfig {}\n", sec.sec_type))?;
            } else {
                buf.write_fmt(format_args!("\nconfig {} '{}'\n", sec.sec_type, sec.name))?;
            }

            for opt in sec.options.iter() {
                match opt.opt_type {
                    UciOptionType::TypeOption => {
                        buf.write_fmt(format_args!("\toption {} '{}'\n", opt.name, opt.values[0]))?;
                    }
                    UciOptionType::TypeList => {
                        for v in opt.values.iter() {
                            buf.write_fmt(format_args!("\tlist {} '{}'\n", opt.name, v))?;
                        }
                    }
                }
            }
        }

        buf.write_all(b"\n")?;
        Ok(())
    }

    pub fn get_section_name(&self, section: &UciSection) -> String {
        if section.name != "" {
            return section.name.clone();
        }
        format!("@{}[{}]", section.sec_type, self._index(section).unwrap())
    }

    pub fn get(&self, name: &str) -> Result<Option<&UciSection>> {
        if name.starts_with("@") {
            self._get_unnamed(name)
        } else {
            self._get_named(name)
        }
    }

    pub fn get_mut(&mut self, name: &str) -> Result<Option<&mut UciSection>> {
        if name.starts_with("@") {
            self._get_unnamed_mut(name)
        } else {
            self._get_named_mut(name)
        }
    }

    pub fn add(&mut self, section: UciSection) -> &mut UciSection {
        self.sections.push(section);
        self.sections.last_mut().unwrap()
    }

    pub fn merge(&mut self, section: UciSection) -> &mut UciSection {
        if self
            .sections
            .iter()
            .find(|sec| self.get_section_name(&section) == self.get_section_name(sec))
            .is_some()
        {
            let same_name_sec_mut = self.get_mut(&section.name).unwrap().unwrap();
            for opt in section.options.into_iter() {
                same_name_sec_mut.merge(opt)
            }

            return same_name_sec_mut;
        };

        self.add(section)
    }

    pub fn del(&mut self, name: &str) {
        if let Some(idx) = self
            .sections
            .iter()
            .position(|sec| self.get_section_name(sec) == name)
        {
            self.sections.remove(idx);
        };
    }
}

fn unmangle_section_name(section_name: &str) -> Result<(String, i32)> {
    let len = section_name.len();
    let bytes_section_name = section_name.as_bytes();
    if len < 5 {
        return Err(Error::new(
            "implausible section selector: must be at least 5 characters long".to_string(),
        ));
    };

    if bytes_section_name[0] as char != '@' {
        return Err(Error::new(
            "invalid syntax: section selector must start with @ sign".to_string(),
        ));
    };

    let (mut bra, ket) = (0, len - 1);

    for (i, r) in bytes_section_name.iter().enumerate() {
        if i != 0 && *r as char == '@' {
            return Err(Error::new(
                "invalid syntax: multiple @ signs found".to_string(),
            ));
        };
        if bra > 0 && *r as char == '[' {
            return Err(Error::new(
                "invalid syntax: multiple open brackets found".to_string(),
            ));
        };
        if i != ket && *r as char == ']' {
            return Err(Error::new(
                "invalid syntax: multiple closed brackets found".to_string(),
            ));
        };
        if *r as char == '[' {
            bra = i;
        };
    }

    if bra == 0 || bra >= ket {
        return Err(Error::new(
            "invalid syntax: section selector must have format '@type[index]'".to_string(),
        ));
    };

    let sec_type = from_utf8(&bytes_section_name[1..bra]).unwrap().to_string();
    let sec_index = match from_utf8(&bytes_section_name[bra + 1..ket])
        .unwrap()
        .parse::<i32>()
    {
        Ok(num) => num,
        Err(err) => {
            return Err(Error::new(format!(
                "invalid syntax: index must be numeric: {}",
                err.to_string()
            )))
        }
    };

    Ok((sec_type, sec_index))
}

#[cfg(test)]
mod test {
    use std::path::Path;

    use super::*;
    use crate::storage::uci::{parser::uci_parse, tree::uci_option::UciOption};
    use crate::utils::tempfile::TempFile;
    #[test]
    fn test_unmangle_section_name() {
        let test_cases = vec![
            (
                "",
                Err(format!(
                    "implausible section selector: must be at least 5 characters long"
                )),
            ),
            (
                "aa[0]",
                Err(format!(
                    "invalid syntax: section selector must start with @ sign"
                )),
            ),
            (
                "@@[0]",
                Err(format!("invalid syntax: multiple @ signs found")),
            ),
            (
                "@@@@@@@@@@@",
                Err(format!("invalid syntax: multiple @ signs found")),
            ),
            (
                "@[[0]",
                Err(format!("invalid syntax: multiple open brackets found")),
            ),
            (
                "@][0]",
                Err(format!("invalid syntax: multiple closed brackets found")),
            ),
            (
                "@aa0]",
                Err(format!(
                    "invalid syntax: section selector must have format '@type[index]'"
                )),
            ),
            (
                "@a[b]",
                Err(format!(
                    "invalid syntax: index must be numeric: invalid digit found in string"
                )),
            ),
            ("@a[0]", Ok((format!("a"), 0))),
            ("@a[4223]", Ok((format!("a"), 4223))),
            ("@a[-1]", Ok((format!("a"), -1))),
            ("@abcdEFGHijkl[-255]", Ok((format!("abcdEFGHijkl"), -255))),
            (
                "@abcdEFGHijkl[0xff]",
                Err(format!(
                    "invalid syntax: index must be numeric: invalid digit found in string"
                )),
            ),
        ];

        for (name, expected) in test_cases {
            match unmangle_section_name(name) {
                Ok((typ, idx)) => {
                    assert_eq!(Ok((typ, idx)), expected);
                }
                Err(err) => {
                    assert_eq!(Err(err.message), expected);
                }
            }
        }
    }

    #[test]
    fn test_config_get() {
        let config = uci_parse("unnamed",format!("\npackage 'abc'\nconfig foo named\n\toption pos '0'\n\toption unnamed '0'\n\tlist list 0\n\nconfig foo\n\toption pos '1'\n\toption unnamed '1'\n\tlist list 10\n\nconfig foo\n\toption pos '2'\n\toption unnamed '1'\n\tlist list 20\n\nconfig foo named\n\toption pos '3'\n\toption unnamed '0'\n\tlist list 30\n"));

        assert!(config.is_ok());

        let test_cases = vec![
            UciSection {
                name: format!("named"),
                sec_type: format!("foo"),
                options: vec![
                    UciOption::new(
                        format!("pos"),
                        UciOptionType::TypeOption,
                        vec![format!("3")],
                    ),
                    UciOption::new(
                        format!("unnamed"),
                        UciOptionType::TypeOption,
                        vec![format!("0")],
                    ),
                    UciOption::new(
                        format!("list"),
                        UciOptionType::TypeList,
                        vec![format!("0"), format!("30")],
                    ),
                ],
            },
            UciSection {
                name: format!("@foo[0]"),
                sec_type: format!("foo"),
                options: vec![
                    UciOption::new(
                        format!("pos"),
                        UciOptionType::TypeOption,
                        vec![format!("3")],
                    ),
                    UciOption::new(
                        format!("unnamed"),
                        UciOptionType::TypeOption,
                        vec![format!("0")],
                    ),
                    UciOption::new(
                        format!("list"),
                        UciOptionType::TypeList,
                        vec![format!("0"), format!("30")],
                    ),
                ],
            },
            UciSection {
                name: format!("@foo[1]"),
                sec_type: format!("foo"),
                options: vec![
                    UciOption::new(
                        format!("pos"),
                        UciOptionType::TypeOption,
                        vec![format!("1")],
                    ),
                    UciOption::new(
                        format!("unnamed"),
                        UciOptionType::TypeOption,
                        vec![format!("1")],
                    ),
                    UciOption::new(
                        format!("list"),
                        UciOptionType::TypeList,
                        vec![format!("10")],
                    ),
                ],
            },
            UciSection {
                name: format!("@foo[2]"),
                sec_type: format!("foo"),
                options: vec![
                    UciOption::new(
                        format!("pos"),
                        UciOptionType::TypeOption,
                        vec![format!("2")],
                    ),
                    UciOption::new(
                        format!("unnamed"),
                        UciOptionType::TypeOption,
                        vec![format!("1")],
                    ),
                    UciOption::new(
                        format!("list"),
                        UciOptionType::TypeList,
                        vec![format!("20")],
                    ),
                ],
            },
            UciSection {
                name: format!("@foo[-3]"),
                sec_type: format!("foo"),
                options: vec![
                    UciOption::new(
                        format!("pos"),
                        UciOptionType::TypeOption,
                        vec![format!("3")],
                    ),
                    UciOption::new(
                        format!("unnamed"),
                        UciOptionType::TypeOption,
                        vec![format!("0")],
                    ),
                    UciOption::new(
                        format!("list"),
                        UciOptionType::TypeList,
                        vec![format!("0"), format!("30")],
                    ),
                ],
            },
            UciSection {
                name: format!("@foo[-2]"),
                sec_type: format!("foo"),
                options: vec![
                    UciOption::new(
                        format!("pos"),
                        UciOptionType::TypeOption,
                        vec![format!("1")],
                    ),
                    UciOption::new(
                        format!("unnamed"),
                        UciOptionType::TypeOption,
                        vec![format!("1")],
                    ),
                    UciOption::new(
                        format!("list"),
                        UciOptionType::TypeList,
                        vec![format!("10")],
                    ),
                ],
            },
            UciSection {
                name: format!("@foo[-1]"),
                sec_type: format!("foo"),
                options: vec![
                    UciOption::new(
                        format!("pos"),
                        UciOptionType::TypeOption,
                        vec![format!("2")],
                    ),
                    UciOption::new(
                        format!("unnamed"),
                        UciOptionType::TypeOption,
                        vec![format!("1")],
                    ),
                    UciOption::new(
                        format!("list"),
                        UciOptionType::TypeList,
                        vec![format!("20")],
                    ),
                ],
            },
        ];

        for expected_sec in test_cases {
            let sec = config.as_ref().unwrap().get(&expected_sec.name);
            assert!(sec.is_ok());
            assert_eq!(
                sec.as_ref().unwrap().unwrap().sec_type,
                expected_sec.sec_type
            );
            for expected_opt in expected_sec.options {
                let opt = sec.as_ref().unwrap().unwrap().get(&expected_opt.name);
                assert!(opt.is_some());
                assert_eq!(opt.unwrap().values, expected_opt.values);
            }
        }
    }

    #[test]
    fn test_config_del() {
        let test_cases = vec![
            (
                UciConfig {
                    name: "test_config".to_owned(),
                    pkg_name: "test_config".to_owned(),
                    sections: vec![UciSection {
                        name: format!("named"),
                        sec_type: format!("foo"),
                        options: vec![],
                    }],
                    modified: false,
                },
                "named",
                None,
            ),
            (
                UciConfig {
                    name: "test_config".to_owned(),
                    pkg_name: "test_config".to_owned(),
                    sections: vec![UciSection {
                        name: "".to_string(),
                        sec_type: format!("foo"),
                        options: vec![],
                    }],
                    modified: false,
                },
                "@foo[0]",
                None,
            ),
        ];

        for (mut cfg, del_name, expected) in test_cases {
            cfg.del(del_name);
            if let Ok(sec) = cfg.get(del_name) {
                assert_eq!(sec, expected);
            };
        }
    }

    #[test]
    fn test_config_write_in() {
        if let Ok(mut tempfile) = TempFile::new("test_temp") {
            let config = UciConfig {
                name: "test_config".to_owned(),
                pkg_name: "test_config".to_owned(),
                sections: vec![UciSection {
                    name: format!("named"),
                    sec_type: format!("foo"),
                    options: vec![
                        UciOption::new(
                            format!("pos1"),
                            UciOptionType::TypeOption,
                            vec![format!("3")],
                        ),
                        UciOption::new(
                            format!("pos2"),
                            UciOptionType::TypeOption,
                            vec![format!("3")],
                        ),
                        UciOption::new(
                            format!("pos3"),
                            UciOptionType::TypeList,
                            vec![format!("3"), format!("5")],
                        ),
                    ],
                }],
                modified: false,
            };
            match config.write_in(&mut tempfile) {
                Ok(()) => match tempfile.persist(Path::new("test_temp/test_config")) {
                    Ok(()) => {
                        assert!(true)
                    }
                    Err(err) => {
                        panic!("{:?}", err)
                    }
                },
                Err(err) => panic!("{:?}", err),
            };
        };
    }
}
