    use std::path::Path;

    use super::*;
    use crate::storage::uci::{parser::uci_parse, tree::uci_option::UciOption};
    use crate::utils::tempfile::TempFile;
    #[test]
    fn test_unmangle_section_name() {
        let test_cases = vec![
            (
                "",
                Err("implausible section selector: must be at least 5 characters long".to_string()),
            ),
            (
                "aa[0]",
                Err("invalid syntax: section selector must start with @ sign".to_string()),
            ),
            (
                "@@[0]",
                Err("invalid syntax: multiple @ signs found".to_string()),
            ),
            (
                "@@@@@@@@@@@",
                Err("invalid syntax: multiple @ signs found".to_string()),
            ),
            (
                "@[[0]",
                Err("invalid syntax: multiple open brackets found".to_string()),
            ),
            (
                "@][0]",
                Err("invalid syntax: multiple closed brackets found".to_string()),
            ),
            (
                "@aa0]",
                Err("invalid syntax: section selector must have format '@type[index]'".to_string()),
            ),
            (
                "@a[b]",
                Err("invalid syntax: index must be numeric: invalid digit found in string".to_string()),
            ),
            ("@a[0]", Ok(("a".to_string(), 0))),
            ("@a[4223]", Ok(("a".to_string(), 4223))),
            ("@a[-1]", Ok(("a".to_string(), -1))),
            ("@abcdEFGHijkl[-255]", Ok(("abcdEFGHijkl".to_string(), -255))),
            (
                "@abcdEFGHijkl[0xff]",
                Err("invalid syntax: index must be numeric: invalid digit found in string".to_string()),
            ),
        ];

        for (name, expected) in test_cases {
            match unmangle_section_name(name) {
                Ok((typ, idx)) => {
                    assert_eq!(Ok((typ, idx)), expected);
                }
                Err(err) => {
                    assert_eq!(Err(err.message), expected);
                }
            }
        }
    }

    #[test]
    fn test_config_get() {
        let config = uci_parse("unnamed","\npackage 'abc'\nconfig foo named\n\toption pos '0'\n\toption unnamed '0'\n\tlist list 0\n\nconfig foo\n\toption pos '1'\n\toption unnamed '1'\n\tlist list 10\n\nconfig foo\n\toption pos '2'\n\toption unnamed '1'\n\tlist list 20\n\nconfig foo named\n\toption pos '3'\n\toption unnamed '0'\n\tlist list 30\n".to_string());

        assert!(config.is_ok());

        let test_cases = vec![
            UciSection {
                name: "named".to_string(),
                sec_type: "foo".to_string(),
                options: vec![
                    UciOption::new(
                        "pos".to_string(),
                        UciOptionType::TypeOption,
                        vec![format!("3")],
                    ),
                    UciOption::new(
                        "unnamed".to_string(),
                        UciOptionType::TypeOption,
                        vec![format!("0")],
                    ),
                    UciOption::new(
                        "list".to_string(),
                        UciOptionType::TypeList,
                        vec![format!("0"), format!("30")],
                    ),
                ],
            },
            UciSection {
                name: "@foo[0]".to_string(),
                sec_type: "foo".to_string(),
                options: vec![
                    UciOption::new(
                        "pos".to_string(),
                        UciOptionType::TypeOption,
                        vec![format!("3")],
                    ),
                    UciOption::new(
                        "unnamed".to_string(),
                        UciOptionType::TypeOption,
                        vec![format!("0")],
                    ),
                    UciOption::new(
                        "list".to_string(),
                        UciOptionType::TypeList,
                        vec![format!("0"), format!("30")],
                    ),
                ],
            },
            UciSection {
                name: "@foo[1]".to_string(),
                sec_type: "foo".to_string(),
                options: vec![
                    UciOption::new(
                        "pos".to_string(),
                        UciOptionType::TypeOption,
                        vec![format!("1")],
                    ),
                    UciOption::new(
                        "unnamed".to_string(),
                        UciOptionType::TypeOption,
                        vec![format!("1")],
                    ),
                    UciOption::new(
                        "list".to_string(),
                        UciOptionType::TypeList,
                        vec![format!("10")],
                    ),
                ],
            },
            UciSection {
                name: "@foo[2]".to_string(),
                sec_type: "foo".to_string(),
                options: vec![
                    UciOption::new(
                        "pos".to_string(),
                        UciOptionType::TypeOption,
                        vec![format!("2")],
                    ),
                    UciOption::new(
                        "unnamed".to_string(),
                        UciOptionType::TypeOption,
                        vec![format!("1")],
                    ),
                    UciOption::new(
                        "list".to_string(),
                        UciOptionType::TypeList,
                        vec![format!("20")],
                    ),
                ],
            },
            UciSection {
                name: "@foo[-3]".to_string(),
                sec_type: "foo".to_string(),
                options: vec![
                    UciOption::new(
                        "pos".to_string(),
                        UciOptionType::TypeOption,
                        vec![format!("3")],
                    ),
                    UciOption::new(
                        "unnamed".to_string(),
                        UciOptionType::TypeOption,
                        vec![format!("0")],
                    ),
                    UciOption::new(
                        "list".to_string(),
                        UciOptionType::TypeList,
                        vec![format!("0"), format!("30")],
                    ),
                ],
            },
            UciSection {
                name: "@foo[-2]".to_string(),
                sec_type: "foo".to_string(),
                options: vec![
                    UciOption::new(
                        "pos".to_string(),
                        UciOptionType::TypeOption,
                        vec![format!("1")],
                    ),
                    UciOption::new(
                        "unnamed".to_string(),
                        UciOptionType::TypeOption,
                        vec![format!("1")],
                    ),
                    UciOption::new(
                        "list".to_string(),
                        UciOptionType::TypeList,
                        vec![format!("10")],
                    ),
                ],
            },
            UciSection {
                name: "@foo[-1]".to_string(),
                sec_type: "foo".to_string(),
                options: vec![
                    UciOption::new(
                        "pos".to_string(),
                        UciOptionType::TypeOption,
                        vec![format!("2")],
                    ),
                    UciOption::new(
                        "unnamed".to_string(),
                        UciOptionType::TypeOption,
                        vec![format!("1")],
                    ),
                    UciOption::new(
                        "list".to_string(),
                        UciOptionType::TypeList,
                        vec![format!("20")],
                    ),
                ],
            },
        ];

        for expected_sec in test_cases {
            let sec = config.as_ref().unwrap().get(&expected_sec.name);
            assert!(sec.is_ok());
            assert_eq!(
                sec.as_ref().unwrap().unwrap().sec_type,
                expected_sec.sec_type
            );
            for expected_opt in expected_sec.options {
                let opt = sec.as_ref().unwrap().unwrap().get(&expected_opt.name);
                assert!(opt.is_some());
                assert_eq!(opt.unwrap().values, expected_opt.values);
            }
        }
    }

    #[test]
    fn test_config_del() {
        let test_cases = vec![
            (
                UciConfig {
                    name: "test_config".to_owned(),
                    pkg_name: "test_config".to_owned(),
                    sections: vec![UciSection {
                        name: "named".to_string(),
                        sec_type: "foo".to_string(),
                        options: vec![],
                    }],
                    modified: false,
                },
                "named",
                None,
            ),
            (
                UciConfig {
                    name: "test_config".to_owned(),
                    pkg_name: "test_config".to_owned(),
                    sections: vec![UciSection {
                        name: "".to_string(),
                        sec_type: "foo".to_string(),
                        options: vec![],
                    }],
                    modified: false,
                },
                "@foo[0]",
                None,
            ),
        ];

        for (mut cfg, del_name, expected) in test_cases {
            cfg.del(del_name);
            if let Ok(sec) = cfg.get(del_name) {
                assert_eq!(sec, expected);
            };
        }
    }

    #[test]
    fn test_config_write_in() {
        if let Ok(mut tempfile) = TempFile::new("test_temp") {
            let config = UciConfig {
                name: "test_config".to_owned(),
                pkg_name: "test_config".to_owned(),
                sections: vec![UciSection {
                    name: "named".to_string(),
                    sec_type: "foo".to_string(),
                    options: vec![
                        UciOption::new(
                            "pos1".to_string(),
                            UciOptionType::TypeOption,
                            vec![format!("3")],
                        ),
                        UciOption::new(
                            "pos2".to_string(),
                            UciOptionType::TypeOption,
                            vec![format!("3")],
                        ),
                        UciOption::new(
                            "pos3".to_string(),
                            UciOptionType::TypeList,
                            vec![format!("3"), format!("5")],
                        ),
                    ],
                }],
                modified: false,
            };
            match config.write_in(&mut tempfile) {
                Ok(()) => match tempfile.persist(Path::new("test_temp/test_config")) {
                    Ok(()) => {
                    }
                    Err(err) => {
                        panic!("{:?}", err)
                    }
                },
                Err(err) => panic!("{:?}", err),
            };
        };
    }