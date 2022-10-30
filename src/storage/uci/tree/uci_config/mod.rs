use std::str::from_utf8;

use super::uci_section::UciSection;
use crate::utils::{Error, Result};

#[derive(Clone, Debug)]
pub struct UciConfig {
    pub name: String,
    pub sections: Vec<UciSection>,
    pub modified: bool,
}

impl UciConfig {
    pub fn new(name: &str) -> UciConfig {
        UciConfig {
            name: name.to_owned(),
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

    fn _get_named(&self, name: &str) -> Option<&UciSection> {
        self.sections.iter().find(|section| section.name == name)
    }

    fn _get_named_mut(&mut self, name: &str) -> Option<&mut UciSection> {
        self.sections
            .iter_mut()
            .find(|section| section.name == name)
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
            return Err(Error::new("invalid name: index out of bounds"));
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
            return Err(Error::new("invalid name: index out of bounds"));
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

    pub(crate) fn set_name(&mut self, name: &str) {
        self.name = name.into();
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
            Ok(self._get_named(name))
        }
    }

    pub fn get_mut(&mut self, name: &str) -> Result<Option<&mut UciSection>> {
        if name.starts_with('@') {
            self._get_unnamed_mut(name)
        } else {
            Ok(self._get_named_mut(name))
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
            let same_name_sec_mut = self
                .get_mut(self.get_section_name(&section).as_str())
                .unwrap()
                .unwrap();
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

    pub fn del_all(&mut self, typ: &str) {
        let secs = self
            .sections
            .clone()
            .into_iter()
            .filter(|sec| sec.sec_type != typ)
            .collect();
        self.sections = secs;
    }
}

fn unmangle_section_name(section_name: &str) -> Result<(String, i32)> {
    let len = section_name.len();
    let bytes_section_name = section_name.as_bytes();
    if len < 5 {
        return Err(Error::new(
            "implausible section selector: must be at least 5 characters long",
        ));
    };

    if bytes_section_name[0] as char != '@' {
        return Err(Error::new(
            "invalid syntax: section selector must start with @ sign",
        ));
    };

    let (mut bra, ket) = (0, len - 1);

    for (i, r) in bytes_section_name.iter().enumerate() {
        if i != 0 && *r as char == '@' {
            return Err(Error::new("invalid syntax: multiple @ signs found"));
        };
        if bra > 0 && *r as char == '[' {
            return Err(Error::new("invalid syntax: multiple open brackets found"));
        };
        if i != ket && *r as char == ']' {
            return Err(Error::new("invalid syntax: multiple closed brackets found"));
        };
        if *r as char == '[' {
            bra = i;
        };
    }

    if bra == 0 || bra >= ket {
        return Err(Error::new(
            "invalid syntax: section selector must have format '@type[index]'",
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
