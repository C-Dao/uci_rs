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
            name,
            opt_type,
            values,
        }
    }

    pub fn set_values(&mut self, values: Vec<String>) {
        self.values = values;
    }

    pub fn set_type(&mut self, typ: UciOptionType) {
        self.opt_type = typ;
    }


    pub fn merge_values(&mut self, values: Vec<String>) {
        match self.opt_type {
            UciOptionType::TypeOption => {
                self.set_values(values);
            }
            UciOptionType::TypeList => {
                let set: HashSet<String> = HashSet::from_iter(self.values.clone().into_iter());

                for v in values {
                    if set.contains(&v) {
                        continue;
                    } else {
                        self.values.push(v);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod test;
