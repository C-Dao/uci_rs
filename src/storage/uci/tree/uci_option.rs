use std::collections::HashSet;

#[derive(PartialEq, Clone, Debug)]
pub struct UciOption {
    pub name: String,
    pub values: Vec<String>,
    pub opt_type: UciOptionType,
}

#[derive(PartialEq, Clone, Debug)]
pub enum UciOptionType {
    TypeOption,
    TypeList,
}

impl UciOption {
    pub fn new(name: String, opt_type: UciOptionType, values: Vec<String>) -> UciOption {
        UciOption {
            name: name,
            opt_type: opt_type,
            values: values,
        }
    }

    pub fn set_values(&mut self, values: Vec<String>) {
        self.values = values;
    }

    pub fn merge_values(&mut self, values: Vec<String>) {
        match self.opt_type {
            UciOptionType::TypeOption => {
                self.set_values(values);
            }
            UciOptionType::TypeList => {
                let set: HashSet<String> =
                    HashSet::from_iter([values, self.values.clone()].concat().into_iter());

                self.values = set.into_iter().collect();
            }
        }
    }
}
