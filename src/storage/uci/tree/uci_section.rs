use super::uci_option::UciOption;

#[derive(PartialEq, Clone, Debug)]
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
            if opt.name == option.name && opt.opt_type == option.opt_type {
                opt.merge_values(option.values);
                return;
            } else if opt.name == option.name && opt.opt_type != option.opt_type {
                opt.set_type(option.opt_type);
                opt.set_values(option.values);
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

#[cfg(test)]
mod test {
    use super::*;
    use crate::storage::uci::tree::uci_option::UciOptionType;

    #[test]
    fn test_section_merge() {
        let test_cases = vec![
            (
                UciSection {
                    name: format!("named"),
                    sec_type: format!("foo"),
                    options: vec![UciOption::new(
                        format!("pos"),
                        UciOptionType::TypeOption,
                        vec![format!("3")],
                    )],
                },
                UciOption::new(
                    format!("pos"),
                    UciOptionType::TypeOption,
                    vec![format!("14")],
                ),
                UciSection {
                    name: format!("named"),
                    sec_type: format!("foo"),
                    options: vec![UciOption::new(
                        format!("pos"),
                        UciOptionType::TypeOption,
                        vec![format!("14")],
                    )],
                },
            ),
            (
                UciSection {
                    name: format!("named"),
                    sec_type: format!("foo"),
                    options: vec![UciOption::new(
                        format!("pos"),
                        UciOptionType::TypeOption,
                        vec![format!("3")],
                    )],
                },
                UciOption::new(
                    format!("pos"),
                    UciOptionType::TypeList,
                    vec![format!("14"), format!("3")],
                ),
                UciSection {
                    name: format!("named"),
                    sec_type: format!("foo"),
                    options: vec![UciOption::new(
                        format!("pos"),
                        UciOptionType::TypeList,
                        vec![format!("14"), format!("3")],
                    )],
                },
            ),
            (
                UciSection {
                    name: format!("named"),
                    sec_type: format!("foo"),
                    options: vec![UciOption::new(
                        format!("pos"),
                        UciOptionType::TypeList,
                        vec![format!("3"), format!("5")],
                    )],
                },
                UciOption::new(
                    format!("pos"),
                    UciOptionType::TypeOption,
                    vec![format!("14")],
                ),
                UciSection {
                    name: format!("named"),
                    sec_type: format!("foo"),
                    options: vec![UciOption::new(
                        format!("pos"),
                        UciOptionType::TypeOption,
                        vec![format!("14")],
                    )],
                },
            ),
            (
                UciSection {
                    name: format!("named"),
                    sec_type: format!("foo"),
                    options: vec![UciOption::new(
                        format!("pos"),
                        UciOptionType::TypeList,
                        vec![format!("3"), format!("5")],
                    )],
                },
                UciOption::new(format!("pos"), UciOptionType::TypeList, vec![format!("14")]),
                UciSection {
                    name: format!("named"),
                    sec_type: format!("foo"),
                    options: vec![UciOption::new(
                        format!("pos"),
                        UciOptionType::TypeList,
                        vec![format!("3"), format!("5"), format!("14")],
                    )],
                },
            ),
        ];

        for (mut sec, val, expected) in test_cases {
            sec.merge(val);
            assert_eq!(sec, expected);
        }
    }
    #[test]
    fn test_section_del() {
        let test_cases = vec![
            (
                UciSection {
                    name: format!("@foo[-1]"),
                    sec_type: format!("foo"),
                    options: vec![UciOption::new(
                        format!("pos"),
                        UciOptionType::TypeOption,
                        vec![format!("2")],
                    )],
                },
                "pos",
                None,
            ),
            (
                UciSection {
                    name: format!("@foo[-1]"),
                    sec_type: format!("foo"),
                    options: vec![UciOption::new(
                        format!("list"),
                        UciOptionType::TypeList,
                        vec![format!("20")],
                    )],
                },
                "list",
                None,
            ),
        ];

        for (mut sec, del_name, expected) in test_cases {
            sec.del(del_name);
            assert_eq!(sec.get(del_name), expected);
        }
    }
}
