mod error;
mod uci_config;
mod uci_option;
mod uci_section;

use std::collections::HashMap;
use error::Error;
use uci_config::UciConfig;

pub struct UciTree {
    dir: String,
    configs: HashMap<String, UciConfig>,
}


pub trait Uci {
    fn load_config(&mut self, name: String, force_reload: bool) -> Result<(), Error>;
    fn unload_config(&mut self, name: String) -> Result<(), Error>;
    fn free_config(&mut self) -> Result<(), Error>;
    fn commit(&mut self) -> Result<(), Error>;
    fn revert(&mut self, configs: Vec<String>) -> Result<(), Error>;
    fn get_sections(&mut self, configs: Vec<String>, sec_type: String) -> (Vec<String>, bool);
    fn add_section(&mut self, configs: Vec<String>, section: String, sec_type: String);
    fn get(&mut self, configs: Vec<String>, section: String, option: String)
        -> (Vec<String>, bool);
    fn get_last(&mut self, configs: Vec<String>, section: String, option: String)
        -> (String, bool);
    fn get_bool(&mut self, configs: Vec<String>, section: String, option: String) -> (bool, bool);
    fn set(
        &mut self,
        configs: Vec<String>,
        section: String,
        option: String,
        values: Vec<String>,
    ) -> bool;
    fn del(&mut self, configs: Vec<String>, section: String, option: String) -> bool;
    fn del_section(&mut self, configs:Vec<String>, section:String);
}
