
    use super::*;
    #[test]
    fn test_option_merge_values() {
        let test_cases = vec![
            (
                UciOption::new(
                    "pos",
                    UciOptionType::TypeOption,
                    vec![format!("3")],
                ),
                vec![format!("5")],
                vec![format!("5")],
            ),
            (
                UciOption::new("pos", UciOptionType::TypeList, vec![format!("3")]),
                vec![format!("5")],
                vec![format!("3"), format!("5")],
            ),
            (
                UciOption::new("pos", UciOptionType::TypeList, vec![format!("3")]),
                vec![],
                vec![format!("3")],
            ),
            (
                UciOption::new("pos", UciOptionType::TypeOption, vec![format!("3")]),
                vec![],
                vec![],
            ),
        ];

        for (mut opt, val, expected) in test_cases {
            opt.merge_values(val);
            assert_eq!(opt.values, expected);
        }
    }