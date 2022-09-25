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

    pub fn add(&self, option: UciOption) {
        self.options.push(option);
    }

    pub fn merge(&self, option: UciOption) {
        for opt in self.options {
            if opt.name == option.name {
                opt.merge_values(option.values);
            }
        }

        self.add(option);
    }

    pub fn del(&self, name: String) -> bool {
        let mut idx = 0;

        for opt in self.options {
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

    pub fn get(&self, name: String) -> Option<&UciOption> {
        self.options.iter().find(|opt| opt.name == name)
    }
}
