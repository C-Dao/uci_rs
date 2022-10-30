use std::io::{BufWriter, Write};

use crate::utils::{Error, Result};

use super::tree::*;

#[allow(dead_code)]

pub struct Uci {
    config: UciConfig,
}

impl Uci {
    #[must_use]
    pub fn new(name: &str) -> Self {
        Self::default(name)
    }

    fn default(name: &str) -> Self {
        Self {
            config: UciConfig::new(name),
        }
    }

    pub(super) fn insert_config(&mut self, config: UciConfig) {
        self.config = config;
    }

    fn _lookup_values(&self, section: &str, option: &str) -> Result<&Vec<String>> {
        match self._lookup_option(section, option) {
            Ok(option) => Ok(&option.values),
            Err(err) => Err(err),
        }
    }

    fn _lookup_option(&self, section: &str, option: &str) -> Result<&UciOption> {
        match self.config.get(section) {
            Ok(Some(sec)) => match sec.get(option) {
                Some(opt) => Ok(opt),
                None => Err(Error::new(format!(
                    "option of {}.{} not found",
                    section, option
                ))),
            },
            Ok(None) => Err(Error::new(format!(
                "option of {}.{} not found",
                section, option
            ))),
            Err(err) => Err(err),
        }
    }

    fn _set_option_with_type(
        &mut self,
        section: &str,
        option: &str,
        opt_type: UciOptionType,
        values: Vec<String>,
    ) -> Result<()> {
        let sec_opt = self.config.get_mut(section)?;
        match sec_opt {
            Some(sec) => match sec.get_mut(option) {
                Some(opt) => {
                    opt.set_values(values);
                    Ok(())
                }
                None => {
                    sec.add(UciOption::new(&option, opt_type, values));
                    Ok(())
                }
            },
            None => Err(Error::new(format!("section '{}' not found", section))),
        }
    }
}

pub trait UciCommand {
    fn add_section(&mut self, typ: &str, name: &str) -> Result<()>;
    fn del_option(&mut self, section: &str, option: &str) -> Result<()>;
    fn del_all(&mut self, typ: &str) -> Result<()>;
    fn del_section(&mut self, section: &str) -> Result<()>;
    fn get_option(&self, section: &str, option: &str) -> Result<(String, &Vec<String>)>;
    fn get_all_options(&self, section: &str) -> Result<Vec<(String, &Vec<String>)>>;
    fn get_option_last(&self, section: &str, option: &str) -> Result<(String, Option<String>)>;
    fn get_option_first(&self, section: &str, option: &str) -> Result<(String, Option<String>)>;
    fn is_bool_value(&self, value: &str) -> bool;
    fn get_section(&self, section: &str) -> Result<Option<(String, String)>>;
    fn get_all(&self, typ: &str) -> Vec<(String, String)>;
    fn get_all_sections(&self) -> Vec<(String, String)>;
    fn get_section_first(&self, typ: &str) -> Option<(String, String)>;
    fn get_section_last(&self, typ: &str) -> Option<(String, String)>;
    fn set_package(&mut self, package: &str) -> Result<()>;
    fn get_package(&self) -> String;
    fn set_option(&mut self, section: &str, option: &str, values: Vec<&str>) -> Result<()>;
    fn for_each<F>(&self, typ: &str, func: F)
    where
        F: FnMut(&UciSection) -> ();
    fn write_in<W: Write>(&self, buf: &mut BufWriter<W>) -> Result<()>;
}

impl UciCommand for Uci {
    fn write_in<W: Write>(&self, buf: &mut BufWriter<W>) -> Result<()> {
        if !self.config.name.is_empty() {
            buf.write_fmt(format_args!("\npackage '{}'\n", self.config.name))?;
        }

        for sec in self.config.sections.iter() {
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

    fn get_option(&self, section: &str, option: &str) -> Result<(String, &Vec<String>)> {
        self._lookup_option(section, option)
            .map(|opt| (opt.name.to_owned(), opt.values.as_ref()))
    }

    fn get_option_last(&self, section: &str, option: &str) -> Result<(String, Option<String>)> {
        let (name, values) = self.get_option(section, option)?;
        Ok((name, values.last().map(|v| v.clone())))
    }

    fn is_bool_value(&self, value: &str) -> bool {
        match value {
            "1" => true,
            "on" => true,
            "true" => true,
            "yes" => true,
            "enabled" => true,
            "0" => false,
            "false" => false,
            "no" => false,
            "disabled" => false,
            _ => false,
        }
    }

    fn set_option(&mut self, section: &str, option: &str, values: Vec<&str>) -> Result<()> {
        if values.len() > 1 {
            self._set_option_with_type(
                section,
                option,
                UciOptionType::TypeList,
                values.into_iter().map(|s| s.to_string()).collect(),
            )
        } else {
            self._set_option_with_type(
                section,
                option,
                UciOptionType::TypeOption,
                values.into_iter().map(|s| s.to_string()).collect(),
            )
        }
    }

    fn del_option(&mut self, section: &str, option: &str) -> Result<()> {
        let sec_opt = self.config.get_mut(section)?;
        match sec_opt {
            Some(sec) => {
                self.config.modified = sec.del(option);
                Ok(())
            }
            None => Ok(()),
        }
    }

    fn add_section(&mut self, typ: &str, name: &str) -> Result<()> {
        if name.is_empty() {
            self.config.add(UciSection::new(typ, name));
            self.config.modified = true;
            Ok(())
        } else {
            match self.config.get(name) {
                Ok(Some(sec)) => {
                    if sec.sec_type != typ {
                        self.config.del(name);
                        self.config.add(UciSection::new(typ, name));
                        self.config.modified = true;
                    }
                    Ok(())
                }
                _ => {
                    self.config.add(UciSection::new(typ, name));
                    self.config.modified = true;
                    Ok(())
                }
            }
        }
    }

    fn del_section(&mut self, section: &str) -> Result<()> {
        self.config.del(section);
        self.config.modified = true;
        Ok(())
    }

    fn set_package(&mut self, package: &str) -> Result<()> {
        self.config.set_name(package);
        Ok(())
    }

    fn get_package(&self) -> String {
        self.config.name.clone()
    }

    fn del_all(&mut self, typ: &str) -> Result<()> {
        self.config.del_all(typ);
        Ok(())
    }

    fn get_all_options(&self, section: &str) -> Result<Vec<(String, &Vec<String>)>> {
        let sec_opt = self.config.get(section)?;
        match sec_opt {
            Some(sec) => Ok(sec
                .options
                .iter()
                .map(|opt| (opt.name.clone(), opt.values.as_ref()))
                .collect()),
            None => Ok(vec![]),
        }
    }

    fn get_option_first(&self, section: &str, option: &str) -> Result<(String, Option<String>)> {
        let opt = self._lookup_option(section, option)?;

        Ok((opt.name.clone(), opt.values.first().map(|v| v.clone())))
    }

    fn get_section(&self, section: &str) -> Result<Option<(String, String)>> {
        let sec_opt = self.config.get(section)?;
        if let Some(sec) = sec_opt {
            Ok(Some((
                sec.sec_type.clone(),
                self.config.get_section_name(sec),
            )))
        } else {
            Ok(None)
        }
    }

    fn get_all_sections(&self) -> Vec<(String, String)> {
        self.config
            .sections
            .iter()
            .map(|sec| (sec.sec_type.clone(), self.config.get_section_name(sec)))
            .collect()
    }

    fn get_all(&self, typ: &str) -> Vec<(String, String)> {
        self.config
            .sections
            .iter()
            .filter(|sec| sec.sec_type == typ)
            .map(|sec| (sec.sec_type.clone(), self.config.get_section_name(sec)))
            .collect()
    }

    fn get_section_first(&self, typ: &str) -> Option<(String, String)> {
        self.config.sections.iter().find_map(|sec| {
            if sec.sec_type == typ {
                Some((sec.sec_type.clone(), self.config.get_section_name(sec)))
            } else {
                None
            }
        })
    }

    fn get_section_last(&self, typ: &str) -> Option<(String, String)> {
        self.config
            .sections
            .iter()
            .filter_map(|sec| {
                if sec.sec_type == typ {
                    Some((sec.sec_type.clone(), self.config.get_section_name(sec)))
                } else {
                    None
                }
            })
            .last()
    }

    fn for_each<F>(&self, typ: &str, mut func: F)
    where
        F: FnMut(&UciSection) -> (),
    {
        self.config
            .sections
            .iter()
            .filter(|sec| sec.sec_type == typ)
            .for_each(|sec| func(sec))
    }
}

#[cfg(test)]
mod test;
