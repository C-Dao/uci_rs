pub mod uci_config;
pub mod uci_option;
pub mod uci_section;

use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::sync::Mutex;

use crate::utils::tempfile::TempFile;
use crate::utils::{Error, Result};

use self::uci_config::UciConfig;
use self::uci_option::{UciOption, UciOptionType};
use self::uci_section::UciSection;
use super::parser::uci_parse;

pub struct UciTree {
    dir: Box<Path>,
    configs: HashMap<String, UciConfig>,
    lock: Mutex<bool>,
}

impl UciTree {
    #[must_use]
    #[allow(dead_code)]
    pub fn new(root: &str) -> Self {
        Self {
            dir: Box::from(Path::new(root)),
            configs: HashMap::new(),
            lock: Mutex::new(false),
        }
    }

    fn _load_config(&mut self, name: &str) -> Result<()> {
        let _lock = self.lock.lock();

        let mut file = File::open(self.dir.join(name))?;
        let mut string_buffer = String::new();

        file.read_to_string(&mut string_buffer)?;

        let cfg = uci_parse(&name, string_buffer)?;

        self.configs.insert(name.to_string(), cfg);

        Ok(())
    }

    fn _save_config(&self, config: &UciConfig) -> Result<()> {
        let mut temp_file = TempFile::new(&self.dir)?;
        match config.write_in(&mut temp_file) {
            Ok(()) => {
                let mut perms = temp_file.as_file().metadata()?.permissions();
                perms.set_mode(0o644);
                temp_file.as_file_mut().set_permissions(perms)?;
                temp_file.as_file_mut().sync_all()?;
                temp_file.persist(self.dir.join(&config.name))?;
                Ok(())
            }
            Err(err) => {
                temp_file.close()?;
                Err(err)
            }
        }
    }

    fn _ensure_config_loaded(&mut self, config_name: &str) -> Result<&UciConfig> {
        if self.configs.contains_key(config_name) {
            return Ok(self.configs.get(config_name).unwrap());
        };

        self.load_config(config_name)?;

        if let Some(cfg) = self.configs.get(config_name) {
            Ok(cfg)
        } else {
            Err(Error::new(format!("load config {} fail", config_name)))
        }
    }

    fn _ensure_config_loaded_mut(&mut self, config_name: &str) -> Result<&mut UciConfig> {
        if self.configs.contains_key(config_name) {
            return Ok(self.configs.get_mut(config_name).unwrap());
        };

        self.load_config(config_name)?;

        return Ok(self.configs.get_mut(config_name).unwrap());
    }

    fn _lookup_values(
        &self,
        config_name: &str,
        section_name: &str,
        option_name: &str,
    ) -> Result<&Vec<String>> {
        match self._lookup_option(config_name, section_name, option_name) {
            Ok(Some(option)) => Ok(&option.values),
            Ok(None) => Err(Error::new(format!(
                "values of {}.{} not found",
                section_name, option_name
            ))),
            Err(err) => Err(err),
        }
    }

    fn _lookup_option(
        &self,
        config_name: &str,
        section_name: &str,
        option_name: &str,
    ) -> Result<Option<&UciOption>> {
        match self.configs.get(config_name) {
            Some(config) => match config.get(section_name) {
                Ok(Some(section)) => Ok(section.get(option_name)),
                Ok(None) => Ok(None),
                Err(err) => Err(err),
            },
            None => Ok(None),
        }
    }

    fn _set_option_with_type(
        &mut self,
        config_name: &str,
        section_name: &str,
        option_name: &str,
        opt_type: UciOptionType,
        values: Vec<String>,
    ) -> Result<()> {
        match self._ensure_config_loaded_mut(config_name) {
            Ok(cfg) => match cfg.get_mut(section_name) {
                Ok(Some(sec)) => match sec.get_mut(option_name) {
                    Some(opt) => Ok(opt.set_values(values)),
                    None => Ok(sec.add(UciOption::new(option_name.to_string(), opt_type, values))),
                },
                Ok(None) => Err(Error::new(format!("section '{}' not found", section_name))),
                Err(err) => Err(err),
            },

            Err(err) => Err(err),
        }
    }
}

pub trait Uci {
    fn add_section(
        &mut self,
        config_name: &str,
        section_type: &str,
        section_name: &str,
    ) -> Result<()>;
    fn commit(&mut self) -> Result<()>;
    fn del_option(
        &mut self,
        config_name: &str,
        section_name: &str,
        option_name: &str,
    ) -> Result<()>;
    fn del_section(&mut self, config_name: &str, section_type_name: &str) -> Result<()>;
    fn get_config(&self, config_name: &str) -> Option<&UciConfig>;
    fn get_dir(&self) -> &Path;
    fn get_option_value(
        &self,
        config_name: &str,
        section_type_name: &str,
        option_name: &str,
    ) -> Result<&Vec<String>>;
    fn get_option_last_value(
        &self,
        config_name: &str,
        section_type_name: &str,
        option_name: &str,
    ) -> Result<Option<String>>;
    fn get_option_bool_value(
        &self,
        config_name: &str,
        section_type_name: &str,
        option_name: &str,
    ) -> Result<bool>;
    fn get_sections(&mut self, config_name: &str, section_type: &str) -> Option<Vec<&UciSection>>;
    fn load_config(&mut self, name: &str) -> Result<()>;
    fn load_config_force(&mut self, name: &str) -> Result<()>;
    fn revert(&mut self, config_names: Vec<String>) -> Result<()>;
    fn set_config_package_name(&mut self, config_name: &str, package_name: &str) -> Result<()>;
    fn set_option_values(
        &mut self,
        config_names: &str,
        section_type_name: &str,
        option_name: &str,
        values: Vec<String>,
    ) -> Result<()>;
}

impl Uci for UciTree {
    fn load_config(&mut self, name: &str) -> Result<()> {
        if self.configs.contains_key(name) {
            return Err(Error::new(format!("{} already loaded", name)));
        };

        self._load_config(name)
    }

    fn load_config_force(&mut self, name: &str) -> Result<()> {
        self._load_config(name)
    }

    fn commit(&mut self) -> Result<()> {
        let _lock = self.lock.lock();

        if let Err(error) = self
            .configs
            .iter()
            .filter(|(_, config)| config.modified == true)
            .try_for_each(|(_, config)| -> Result<()> { self._save_config(config) })
        {
            return Err(error);
        };

        self.configs.iter_mut().for_each(|(_, cfg)| {
            cfg.modified = false;
        });

        Ok(())
    }

    fn revert(&mut self, config_names: Vec<String>) -> Result<()> {
        let _lock = self.lock.lock();

        for config_name in config_names {
            self.configs.remove(&config_name);
        }

        Ok(())
    }

    fn get_sections(&mut self, config_name: &str, section_type: &str) -> Option<Vec<&UciSection>> {
        if let Ok(cfg) = self._ensure_config_loaded(config_name) {
            return Some(
                cfg.sections
                    .iter()
                    .filter(|section| section.sec_type == section_type)
                    .collect(),
            );
        };

        None
    }

    fn get_option_value(
        &self,
        config_name: &str,
        section_type_name: &str,
        option_name: &str,
    ) -> Result<&Vec<String>> {
        if let Ok(values) = self._lookup_values(config_name, section_type_name, option_name) {
            return Ok(values);
        };

        match self._lookup_values(config_name, section_type_name, option_name) {
            Ok(values) => Ok(values),
            Err(err) => Err(err),
        }
    }

    fn get_option_last_value(
        &self,
        config_name: &str,
        section_type_name: &str,
        option_name: &str,
    ) -> Result<Option<String>> {
        match self.get_option_value(config_name, section_type_name, option_name) {
            Ok(values) => Ok(Some(values.last().unwrap().clone())),
            Err(err) => Err(err),
        }
    }

    fn get_option_bool_value(
        &self,
        config_name: &str,
        section_type_name: &str,
        option_name: &str,
    ) -> Result<bool> {
        match self.get_option_last_value(config_name, section_type_name, option_name) {
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
                _ => Ok(false),
            },
            Ok(None) => Ok(false),
            Err(err) => Err(err),
        }
    }

    fn set_option_values(
        &mut self,
        config_name: &str,
        section_type_name: &str,
        option_name: &str,
        values: Vec<String>,
    ) -> Result<()> {
        if values.len() > 1 {
            self._set_option_with_type(
                config_name,
                section_type_name,
                option_name,
                UciOptionType::TypeList,
                values,
            )
        } else {
            self._set_option_with_type(
                config_name,
                section_type_name,
                option_name,
                UciOptionType::TypeOption,
                values,
            )
        }
    }

    fn del_option(
        &mut self,
        config_name: &str,
        section_type_name: &str,
        option_name: &str,
    ) -> Result<()> {
        match self._ensure_config_loaded_mut(config_name) {
            Ok(cfg) => match cfg.get_mut(section_type_name) {
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
        section_type: &str,
        section_name: &str,
    ) -> Result<()> {
        let cfg_res = self._ensure_config_loaded_mut(config_name);

        let cfg = if cfg_res.is_err() {
            let mut cfg = UciConfig::new(&config_name);
            cfg.modified = true;
            self.configs.insert(config_name.to_string(), cfg);
            self.configs.get_mut(config_name).unwrap()
        } else {
            cfg_res.unwrap()
        };

        if section_name == "" {
            cfg.add(UciSection::new(
                section_type.to_string(),
                section_name.to_string(),
            ));
            cfg.modified = true;
            Ok(())
        } else {
            match cfg.get(section_name) {
                Ok(Some(sec)) => {
                    if sec.sec_type != section_type {
                        Err(Error::new(format!(
                            "type mismatch for {}.{}, got {}, want {}",
                            config_name, section_name, sec.sec_type, section_type
                        )))
                    } else {
                        Ok(())
                    }
                }
                _ => {
                    cfg.add(UciSection::new(
                        section_type.to_string(),
                        section_name.to_string(),
                    ));
                    cfg.modified = true;
                    Ok(())
                }
            }
        }
    }

    fn del_section(&mut self, config_name: &str, section_type_name: &str) -> Result<()> {
        match self._ensure_config_loaded_mut(config_name) {
            Ok(cfg) => {
                cfg.del(section_type_name);
                cfg.modified = true;
                Ok(())
            }
            Err(err) => Err(err),
        }
    }

    fn set_config_package_name(&mut self, config_name: &str, package_name: &str) -> Result<()> {
        match self.configs.get_mut(config_name) {
            Some(cfg) => {
                cfg.set_pkg_name(package_name.to_string());
                Ok(())
            }
            None => Err(Error::new(format!("config not found"))),
        }
    }

    fn get_config(&self, config_name: &str) -> Option<&UciConfig> {
        self.configs.get(config_name)
    }

    fn get_dir(&self) -> &Path {
        self.dir.as_ref()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_uci_load_config() -> Result<()> {
        let mut tree = UciTree::new("test_data");
        tree.load_config("uci_config")?;
        if let Some(config) = tree.get_config("uci_config") {
            assert_eq!(config.name, "uci_config");
        } else {
            panic!("load test_data/uci_config failed");
        };

        let err = tree.load_config("uci_config");
        assert!(err.is_err());

        Ok(())
    }

    #[test]
    fn test_uci_load_config_force() -> Result<()> {
        let mut tree = UciTree::new("test_data");
        tree.load_config_force("uci_config")?;
        if let Some(config) = tree.get_config("uci_config") {
            assert_eq!(config.name, "uci_config");
        } else {
            panic!("load test_data/uci_config failed");
        };

        if let Some(config) = tree.get_config("uci_config") {
            assert_eq!(config.name, "uci_config");
        } else {
            panic!("load test_data/uci_config failed");
        };

        Ok(())
    }

    #[test]
    fn test_uci_commit() -> Result<()> {
        let mut tree = UciTree::new("test_data");
        tree.load_config("uci_config")?;
        tree.add_section("uci_config", "section", "new_section")?;
        tree.set_option_values(
            "uci_config",
            "new_section",
            "abc",
            vec![format!("1"), format!("2"), format!("3")],
        )?;
        tree.commit()?;
        tree.load_config_force("uci_config")?;
        let opt_res = tree.get_option_value("uci_config", "new_section", "abc");
        assert!(opt_res.is_ok());
        tree.del_section("uci_config", "new_section")?;
        tree.commit()?;
        assert!(true);
        Ok(())
    }

    #[test]
    fn test_uci_revert() -> Result<()> {
        let mut tree = UciTree::new("test_data");
        tree.load_config("uci_config")?;
        tree.add_section("uci_config", "section", "new_section")?;
        tree.revert(vec!["uci_config".to_owned()])?;
        tree.load_config("uci_config")?;
        if let Some(config) = tree.get_config("uci_config") {
            assert_eq!(config.modified, false);
            Ok(())
        } else {
            panic!("revert test_data/uci_config failed");
        }
    }

    #[test]
    fn test_uci_get_sections() -> Result<()> {
        let mut tree = UciTree::new("test_data");
        tree.load_config("uci_config")?;
        if let Some(sections) = tree.get_sections("uci_config", "internal") {
            assert_eq!(sections.len(), 4);
            Ok(())
        } else {
            panic!("get sections of 'sec_type: internal' failed");
        }
    }

    #[test]
    fn test_uci_get_option_value() -> Result<()> {
        let mut tree = UciTree::new("test_data");
        tree.load_config("uci_config")?;
        if let Ok(values) = tree.get_option_value("uci_config", "main", "lang") {
            assert_eq!(values[0], "auto");
            Ok(())
        } else {
            panic!("get option value of 'uci_config.main.lang' failed");
        }
    }

    #[test]
    fn test_uci_get_last_value() -> Result<()> {
        let mut tree = UciTree::new("test_data");
        tree.load_config("uci_config")?;
        if let Ok(values) = tree.get_option_last_value("uci_config", "ntp", "server") {
            if (values.is_some()) {
                assert_eq!(values.unwrap(), "3.lede.pool.ntp.org");
            }
            Ok(())
        } else {
            panic!("get option last value of 'uci_config.ntp.server' failed");
        }
    }

    #[test]
    fn test_uci_get_bool_value() -> Result<()> {
        let mut tree = UciTree::new("test_data");
        tree.load_config("uci_config")?;
        if let Ok(values) = tree.get_option_bool_value("uci_config", "ccache", "enable") {
            assert_eq!(values, true);
            Ok(())
        } else {
            panic!("get option bool value of 'uci_config.ccache.enable' failed");
        }
    }

    #[test]
    fn test_uci_set_option_values() -> Result<()> {
        let mut tree = UciTree::new("test_data");
        tree.load_config("uci_config")?;
        if let Ok(_) = tree.set_option_values("uci_config", "main", "lang", vec![format!("en")]) {
            if let Ok(values) = tree.get_option_value("uci_config", "main", "lang") {
                assert_eq!(values[0], format!("en"));
            };
            Ok(())
        } else {
            panic!("set option values of 'uci_config.main.lang=en' failed");
        }
    }

    #[test]
    fn test_uci_del_option() -> Result<()> {
        let mut tree = UciTree::new("test_data");
        tree.load_config("uci_config")?;
        tree.del_option("uci_config", "main", "lang")?;
        if let Ok(_) = tree.get_option_value("uci_config", "main", "lang") {
            panic!("delete option of 'uci_config.main.lang' failed");
        } else {
            Ok(())
        }
    }

    #[test]
    fn test_uci_add_section() -> Result<()> {
        let mut tree = UciTree::new("test_data");
        tree.load_config("uci_config")?;
        tree.add_section("uci_config", "core", "setting")?;
        tree.set_option_values("uci_config", "setting", "off", vec![])?;
        if let Ok(_) = tree.get_option_value("uci_config", "setting", "off") {
            Ok(())
        } else {
            panic!("add section of 'uci_config.setting' failed");
        }
    }

    #[test]
    fn test_uci_del_section() -> Result<()> {
        let mut tree = UciTree::new("test_data");
        tree.load_config("uci_config")?;
        tree.del_section("uci_config", "main")?;
        if let Ok(_) = tree.get_option_value("uci_config", "main", "lang") {
            panic!("delete section of 'uci_config.main' failed");
        } else {
            Ok(())
        }
    }
}
