mod uci_config;
mod uci_option;
mod uci_section;

use crate::utils::Error;
use smol::fs::unix::PermissionsExt;
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::sync::Mutex;
use tempfile::NamedTempFile;

use uci_config::UciConfig;
use uci_option::UciOption;
use uci_section::UciSection;

use self::uci_option::UciOptionType;

pub struct UciTree {
    dir: Box<Path>,
    configs: HashMap<String, UciConfig>,
    lock: Mutex<bool>,
}

impl UciTree {
    pub fn new(root: String) -> Self {
        Self {
            dir: Box::from(Path::new(&root)),
            configs: HashMap::new(),
            lock: Mutex::new(false),
        }
    }

    fn _load_config(&self, name: &str) -> Result<(), Error> {
        let mut file = File::open(self.dir.join(name))?;
        let mut string_buffer = String::new();

        file.read_to_string(&mut string_buffer)?;

        let cfg = todo!("parse from string_buffer");

        self.configs.insert(name.to_string(), cfg);

        Ok(())
    }

    fn _save_config(&self, mut config: &mut UciConfig) -> Result<(), Error> {
        let mut temp_file = NamedTempFile::new_in(&self.dir)?;
        match config.write_in(&mut temp_file) {
            Ok(()) => {
                let mut perms = temp_file.as_file().metadata()?.permissions();
                perms.set_mode(0o644);
                temp_file.as_file_mut().set_permissions(perms)?;
                temp_file.as_file_mut().sync_all()?;
                temp_file.persist(self.dir.join(&config.name))?;
                config.modified = false;
                Ok(())
            }
            Err(err) => {
                temp_file.close()?;
                Err(err)
            }
        }
    }

    fn _ensure_config_loaded(&mut self, config_name: &str) -> Result<&UciConfig, Error> {
        if let Some(cfg) = self.configs.get(config_name) {
            return Ok(cfg);
        };

        match self.load_config(config_name) {
            Ok(_) => {
                let Some(cfg) = self.configs.get(config_name);
                Ok(cfg)
            }
            Err(err) => Err(err),
        }
    }

    fn _ensure_config_loaded_mut(&self, config_name: &str) -> Result<&mut UciConfig, Error> {
        if let Some(cfg) = self.configs.get_mut(config_name) {
            return Ok(cfg);
        };

        match self.load_config(config_name) {
            Ok(_) => {
                let Some(cfg) = self.configs.get_mut(config_name);
                Ok(cfg)
            }
            Err(err) => Err(err),
        }
    }

    fn _lookup_values(
        &self,
        config_name: &str,
        section_name: &str,
        option_name: &str,
    ) -> Result<Option<Vec<String>>, Error> {
        match self._lookup_option(config_name, section_name, option_name) {
            Ok(Some(option)) => Ok(Some(option.values)),
            Ok(None) => Ok(None),
            Err(err) => Err(err),
        }
    }

    fn _lookup_option(
        &self,
        config_name: &str,
        section_name: &str,
        option_name: &str,
    ) -> Result<Option<&UciOption>, Error> {
        match self.configs.get(config_name) {
            Some(config) => match config.get(section_name) {
                Ok(Some(section)) => Ok(section.get(option_name)),
                Ok(None) => Ok(None),
                Err(err) => Err(err),
            },
            None => Ok(None),
        }
    }
}

pub trait Uci {
    fn load_config(&mut self, name: &str) -> Result<(), Error>;
    fn load_config_force(&mut self, name: &str) -> Result<(), Error>;
    fn commit(&mut self) -> Result<(), Error>;
    fn revert(&mut self, config_names: Vec<String>) -> Result<(), Error>;
    fn get_sections(
        &mut self,
        config_name: &str,
        sec_type: &str,
    ) -> Result<Option<Vec<String>>, Error>;
    fn add_section(
        &mut self,
        config_name: &str,
        section_name: &str,
        sec_type: &str,
    ) -> Result<(), Error>;
    fn get(
        &mut self,
        config_name: &str,
        section_name: &str,
        option_name: &str,
    ) -> Result<Option<Vec<String>>, Error>;
    fn get_last(
        &self,
        config_name: &str,
        section_name: &str,
        option_name: &str,
    ) -> Result<Option<String>, Error>;
    fn get_bool(
        &self,
        config_name: &str,
        section_name: &str,
        option_name: &str,
    ) -> Result<bool, Error>;
    fn set_type(
        &mut self,
        config_name: &str,
        section_name: &str,
        option_name: &str,
        opt_type: UciOptionType,
        values: Vec<String>,
    ) -> Result<(), Error>;
    fn set(
        &mut self,
        config_names: &str,
        section_name: &str,
        option_name: &str,
        values: Vec<String>,
    ) -> Result<(), Error>;
    fn del(
        &mut self,
        config_name: &str,
        section_name: &str,
        option_name: &str,
    ) -> Result<(), Error>;
    fn del_section(&mut self, config_name: &str, section_name: &str) -> Result<(), Error>;
}

impl Uci for UciTree {
    fn load_config(&mut self, name: &str) -> Result<(), Error> {
        let lock = self.lock.lock();

        if let Some(config) = self.configs.get(name) {
            return Err(Error::new(format!("{} already loaded", name)));
        };

        self._load_config(name);
        Ok(())
    }

    fn load_config_force(&mut self, name: &str) -> Result<(), Error> {
        let lock = self.lock.lock();

        self._load_config(name);

        Ok(())
    }

    fn commit(&mut self) -> Result<(), Error> {
        let lock = self.lock.lock();

        if let Err(error) = self
            .configs
            .iter_mut()
            .filter(|(_, config)| config.modified == true)
            .try_for_each(|(_, config)| -> Result<(), Error> { self._save_config(config) })
        {
            return Err(error);
        };

        Ok(())
    }

    fn revert(&mut self, config_names: Vec<String>) -> Result<(), Error> {
        let lock = self.lock.lock();

        for config_name in config_names {
            self.configs.remove(&config_name);
        }

        Ok(())
    }

    fn get_sections(
        &mut self,
        config_name: &str,
        sec_type: &str,
    ) -> Result<Option<Vec<String>>, Error> {
        if let Ok(cfg) = self._ensure_config_loaded(config_name) {
            return Ok(Some(
                cfg.sections
                    .into_iter()
                    .filter(|section| section.sec_type == sec_type)
                    .map(|section| cfg.get_section_name(&section))
                    .collect(),
            ));
        };

        Ok(None)
    }

    fn get(
        &mut self,
        config_name: &str,
        section_name: &str,
        option_name: &str,
    ) -> Result<Option<Vec<String>>, Error> {
        let lock = self.lock.lock();
        match self._lookup_values(config_name, section_name, option_name) {
            Ok(Some(values)) => {
                return Ok(Some(values));
            }
            Ok(None) => match self.load_config(config_name) {
                Ok(_) => match self._lookup_values(config_name, section_name, option_name) {
                    Ok(values) => Ok(values),
                    Err(err) => Err(err),
                },
                Err(err) => Err(err),
            },
            Err(err) => Err(err),
        }
    }

    fn get_last(
        &self,
        config_name: &str,
        section_name: &str,
        option_name: &str,
    ) -> Result<Option<String>, Error> {
        match self.get(config_name, section_name, option_name) {
            Ok(Some(values)) => Ok(Some(values.last().unwrap().clone())),
            Ok(None) => Ok(None),
            Err(err) => Err(err),
        }
    }

    fn get_bool(
        &self,
        config_name: &str,
        section_name: &str,
        option_name: &str,
    ) -> Result<bool, Error> {
        match self.get_last(config_name, section_name, option_name) {
            Ok(Some(val)) => match val.as_str() {
                "1" => Ok(true),
                "on" => Ok(true),
                "true" => Ok(true),
                "yes" => Ok(true),
                "enabled" => Ok(true),
                "0" => Ok(false),
                "false" => Ok(false),
                "no" => Ok(false),
                "disabled" => Ok(false),
            },
            Ok(None) => Ok(false),
            Err(err) => Err(err),
        }
    }

    fn set_type(
        &mut self,
        config_name: &str,
        section_name: &str,
        option_name: &str,
        opt_type: UciOptionType,
        values: Vec<String>,
    ) -> Result<(), Error> {
        let lock = self.lock.lock();

        match self._ensure_config_loaded(config_name) {
            Ok(cfg) => match cfg.get(section_name) {
                Ok(Some(sec)) => match sec.get(option_name) {
                    Some(opt) => Ok(opt.set_values(values)),
                    None => Ok(sec.add(UciOption::new(option_name.to_string(), opt_type, values))),
                },
                Ok(None) => Err(Error::new(format!("section '{}' not found", section_name))),
                Err(err) => Err(err),
            },

            Err(err) => Err(err),
        }
    }

    fn set(
        &mut self,
        config_name: &str,
        section_name: &str,
        option_name: &str,
        values: Vec<String>,
    ) -> Result<(), Error> {
        if values.len() > 1 {
            self.set_type(
                config_name,
                section_name,
                option_name,
                UciOptionType::TypeList,
                values,
            )
        } else {
            self.set_type(
                config_name,
                section_name,
                option_name,
                UciOptionType::TypeOption,
                values,
            )
        }
    }

    fn del(
        &mut self,
        config_name: &str,
        section_name: &str,
        option_name: &str,
    ) -> Result<(), Error> {
        let lock = self.lock.lock();

        match self._ensure_config_loaded_mut(config_name) {
            Ok(cfg) => match cfg.get(section_name) {
                Ok(Some(sec)) => {
                    cfg.modified = sec.del(option_name);
                    Ok(())
                }
                Ok(None) => Ok(()),
                Err(err) => Err(err),
            },
            Err(err) => Err(err),
        }
    }

    fn add_section(
        &mut self,
        config_name: &str,
        section_name: &str,
        sec_type: &str,
    ) -> Result<(), Error> {
        let lock = self.lock.lock();

        let mut cfg_res = self._ensure_config_loaded_mut(config_name);

        let cfg = if cfg_res.is_err() {
            let mut cfg = UciConfig::new(config_name.to_string());
            cfg.modified = true;
            self.configs.insert(config_name.to_string(), cfg);
            self.configs.get(config_name).unwrap()
        } else {
            cfg_res.unwrap()
        };

        match cfg.get(section_name) {
            Ok(Some(sec)) => {
                if sec.sec_type != sec_type {
                    Err(Error::new(format!(
                        "type mismatch for {}.{}, got {}, want {}",
                        config_name, section_name, sec.sec_type, sec_type
                    )))
                } else {
                    Ok(())
                }
            }
            _ => {
                cfg.add(UciSection::new(
                    sec_type.to_string(),
                    section_name.to_string(),
                ));
                cfg.modified = true;
                Ok(())
            }
        }
    }

    fn del_section(&mut self, config_name: &str, section_name: &str) -> Result<(), Error> {
        let lock = self.lock.lock();
        match self._ensure_config_loaded_mut(config_name) {
            Ok(cfg) => {
                cfg.del(section_name);
                cfg.modified = true;
                Ok(())
            }
            Err(err) => Err(err),
        }
    }
}
