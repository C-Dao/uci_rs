mod uci_config;
mod uci_option;
mod uci_section;

use crate::utils::Error;
use std::collections::HashMap;
use std::fmt::format;
use std::fs::{self, File};
use std::io::prelude::*;
use std::path::Path;
use std::sync::Mutex;
use smol::fs::unix::PermissionsExt;
use tempfile::NamedTempFile;

use uci_config::UciConfig;
use uci_option::UciOption;
use uci_section::UciSection;

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

    fn _load_config(&self, name: String) -> Result<(), Error> {
        let file = File::open(self.dir.join(name))?;
        let mut string_buffer = String::new();

        file.read_to_string(&mut string_buffer)?;

        let cfg = todo!("parse from string_buffer");

        self.configs.insert(name, cfg);

        Ok(())
    }

    fn _save_config(&self, mut config: &mut UciConfig) -> Result<(), Error> {
        let mut temp_file = NamedTempFile::new_in(self.dir)?;
        match config.write_in(&mut temp_file){
            Ok(()) =>{
                let mut perms = temp_file.as_file().metadata()?.permissions();
                perms.set_mode(0o644);
                temp_file.as_file_mut().set_permissions(perms)?;
                temp_file.as_file_mut().sync_all()?;
                temp_file.persist(self.dir.join(config.name))?;
                temp_file.close()?;
                config.modified = false;
                Ok(())
            },
            Err(err)=>{
                temp_file.close()?;
                Err(err)
            }
        }
    }

    fn _ensure_config_loaded(&self, config_name: String) -> Result<Option<&UciConfig>, Error> {
        if let Some(cfg) = self.configs.get(&config_name) {
            return Ok(Some(cfg));
        };

        if self.load_config(config_name).is_ok() {
            let cfg = self.configs.get(&config_name);
            return Ok(cfg);
        }

        Ok(None)
    }

    fn _lookup_values(
        &self,
        config_name: String,
        section_name: String,
        option_name: String,
    ) -> Option<Vec<String>> {
        match self._lookup_option(config_name, section_name, option_name) {
            Some(option) => Some(option.values),
            None => None,
        }
    }

    fn _lookup_option(
        &self,
        config_name: String,
        section_name: String,
        option_name: String,
    ) -> Option<UciOption> {
        match self.configs.get(&config_name) {
            Some(config) => match config.get(section_name) {
                Some(section) => section.get(option_name),
                None => None,
            },
            None => None,
        }
    }
}

pub trait Uci {
    fn load_config(&mut self, name: String) -> Result<(), Error>;
    fn load_config_force(&mut self, name: String) -> Result<(), Error>;
    fn unload_config(&mut self, name: String) -> Result<(), Error>;
    fn free_config(&mut self) -> Result<(), Error>;
    fn commit(&mut self) -> Result<(), Error>;
    fn revert(&mut self, config_names: Vec<String>) -> Result<(), Error>;
    fn get_sections(
        &mut self,
        config_name: String,
        sec_type: String,
    ) -> Result<Option<Vec<String>>, Error>;
    fn add_section(&mut self, config_names: Vec<String>, section_name: String, sec_type: String);
    fn get(
        &mut self,
        config_name: String,
        section_name: String,
        option_name: String,
    ) -> Result<Option<Vec<String>>, Error>;
    fn get_last(
        &mut self,
        config_names: Vec<String>,
        section_name: String,
        option_name: String,
    ) -> Result<Option<String>, Error>;
    fn get_bool(
        &mut self,
        config_names: Vec<String>,
        section_name: String,
        option_name: String,
    ) -> Result<Option<bool>, Error>;
    fn set(
        &mut self,
        config_names: Vec<String>,
        section_name: String,
        option_name: String,
        values: Vec<String>,
    ) -> Result<(), Error>;
    fn del(
        &mut self,
        config_names: Vec<String>,
        section_name: String,
        option_name: String,
    ) -> Result<(), Error>;
    fn del_section(&mut self, config_names: Vec<String>, section_name: String) -> Result<(), Error>;
}

impl Uci for UciTree {
    fn load_config(&mut self, name: String) -> Result<(), Error> {
        let lock = self.lock.lock();

        if let Some(config) = self.configs.get(&name) {
            return Err(Error::new(format!("{} already loaded", name)));
        };

        self._load_config(name);
        Ok(())
    }

    fn load_config_force(&mut self, name: String) -> Result<(), Error> {
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
        config_name: String,
        sec_type: String,
    ) -> Result<Option<Vec<String>>, Error> {
        if let Ok(Some(cfg)) = self._ensure_config_loaded(config_name) {
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

    fn get(&mut self, config_name: String, section_name: String, option_name: String) -> Result<Option<Vec<String>>, Error> {
        let lock = self.lock.lock();
        let values = self._lookup_values(config_name, section_name, option_name);
        if values.is_some() {
            return Ok(values);
        }

        match self.load_config(config_name) {
            Ok(_) => Ok(self._lookup_values(config_name, section_name, option_name)),
            Err(err) => Err(err),
        }
    }
}
