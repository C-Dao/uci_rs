use super::uci_option::UciOption;

#[derive(PartialEq)]
pub struct UciSection {
    pub name: String,
    pub sec_type: String,
    pub options: Vec<UciOption>,
}

impl UciSection {
    pub fn new(sec_type: String, name: String) -> UciSection {
        UciSection {
            name: name,
            sec_type: sec_type,
            options: Vec::new(),
        }
    }

    pub fn add(&mut self, option: UciOption) {
        self.options.push(option);
    }

    pub fn merge(&mut self, option: UciOption) {
        for opt in self.options.iter_mut() {
            if opt.name == option.name {
                opt.merge_values(option.values);
                return;
            }
        }
        self.add(option);
    }

    pub fn del(&mut self, name: &str) -> bool {
        let mut idx = 0;

        for opt in self.options.iter() {
            if opt.name == name {
                break;
            }
            idx += 1;
        }

        if idx == self.options.len() {
            return false;
        }

        self.options.remove(idx);

        true
    }

    pub fn get(&self, name: &str) -> Option<&UciOption> {
        self.options.iter().find(|opt| opt.name == name)
    }

    pub fn get_mut(&mut self, name: &str) -> Option<&mut UciOption> {
        self.options.iter_mut().find(|opt| opt.name == name)
    }
}
