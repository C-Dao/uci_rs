    use super::*;
    use crate::storage::uci::tree::uci_option::UciOptionType;

    #[test]
    fn test_section_merge() {
        let test_cases = vec![
            (
                UciSection {
                    name: "named".to_string(),
                    sec_type: "foo".to_string(),
                    options: vec![UciOption::new(
                        "pos".to_string(),
                        UciOptionType::TypeOption,
                        vec![format!("3")],
                    )],
                },
                UciOption::new(
                    "pos".to_string(),
                    UciOptionType::TypeOption,
                    vec![format!("14")],
                ),
                UciSection {
                    name: "named".to_string(),
                    sec_type: "foo".to_string(),
                    options: vec![UciOption::new(
                        "pos".to_string(),
                        UciOptionType::TypeOption,
                        vec![format!("14")],
                    )],
                },
            ),
            (
                UciSection {
                    name: "named".to_string(),
                    sec_type: "foo".to_string(),
                    options: vec![UciOption::new(
                        "pos".to_string(),
                        UciOptionType::TypeOption,
                        vec![format!("3")],
                    )],
                },
                UciOption::new(
                    "pos".to_string(),
                    UciOptionType::TypeList,
                    vec![format!("14"), format!("3")],
                ),
                UciSection {
                    name: "named".to_string(),
                    sec_type: "foo".to_string(),
                    options: vec![UciOption::new(
                        "pos".to_string(),
                        UciOptionType::TypeList,
                        vec![format!("14"), format!("3")],
                    )],
                },
            ),
            (
                UciSection {
                    name: "named".to_string(),
                    sec_type: "foo".to_string(),
                    options: vec![UciOption::new(
                        "pos".to_string(),
                        UciOptionType::TypeList,
                        vec![format!("3"), format!("5")],
                    )],
                },
                UciOption::new(
                    "pos".to_string(),
                    UciOptionType::TypeOption,
                    vec![format!("14")],
                ),
                UciSection {
                    name: "named".to_string(),
                    sec_type: "foo".to_string(),
                    options: vec![UciOption::new(
                        "pos".to_string(),
                        UciOptionType::TypeOption,
                        vec![format!("14")],
                    )],
                },
            ),
            (
                UciSection {
                    name: "named".to_string(),
                    sec_type: "foo".to_string(),
                    options: vec![UciOption::new(
                        "pos".to_string(),
                        UciOptionType::TypeList,
                        vec![format!("3"), format!("5")],
                    )],
                },
                UciOption::new("pos".to_string(), UciOptionType::TypeList, vec![format!("14")]),
                UciSection {
                    name: "named".to_string(),
                    sec_type: "foo".to_string(),
                    options: vec![UciOption::new(
                        "pos".to_string(),
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
                    name: "@foo[-1]".to_string(),
                    sec_type: "foo".to_string(),
                    options: vec![UciOption::new(
                        "pos".to_string(),
                        UciOptionType::TypeOption,
                        vec![format!("2")],
                    )],
                },
                "pos",
                None,
            ),
            (
                UciSection {
                    name: "@foo[-1]".to_string(),
                    sec_type: "foo".to_string(),
                    options: vec![UciOption::new(
                        "list".to_string(),
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