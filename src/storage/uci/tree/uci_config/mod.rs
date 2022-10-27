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
            name: name.to_owned(),
            pkg_name: "".to_owned(),
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
            .nth(index as usize);

        Ok(section)
    }

    fn _count(&self, sec_type: &str) -> usize {
        self.sections
            .iter()
            .filter(|sec| sec.sec_type == sec_type)
            .count()
    }

    pub(crate) fn set_pkg_name(&mut self, name: String) {
        self.pkg_name = name;
    }

    pub fn write_in(&self, file: &mut TempFile) -> Result<()> {
        let mut buf = BufWriter::new(file);

        if !self.pkg_name.is_empty() {
            buf.write_fmt(format_args!("\npackage '{}'\n", self.pkg_name))?;
        }

        for sec in self.sections.iter() {
            if sec.name.is_empty() {
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
        if !section.name.is_empty() {
            return section.name.clone();
        }
        format!("@{}[{}]", section.sec_type, self._index(section).unwrap())
    }

    pub fn get(&self, name: &str) -> Result<Option<&UciSection>> {
        if name.starts_with('@') {
            self._get_unnamed(name)
        } else {
            self._get_named(name)
        }
    }

    pub fn get_mut(&mut self, name: &str) -> Result<Option<&mut UciSection>> {
        if name.starts_with('@') {
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
            .any(|sec| self.get_section_name(&section) == self.get_section_name(sec))
        {
            let same_name_sec_mut = self.get_mut(self.get_section_name(&section).as_str()).unwrap().unwrap();
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
                err
            )))
        }
    };

    Ok((sec_type, sec_index))
}

#[cfg(test)]
mod test;
