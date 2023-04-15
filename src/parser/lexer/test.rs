use super::*;

#[test]
fn test_lexer() {
        let test_cases = vec![
            (
                "empty1", 
                String::new(), 
                vec![]
            ),
            (
                "empty2", 
                "  \n\t\n\n \n ".to_string(), 
                vec![]
            ),
            (
                "simple",
                "config sectiontype 'sectionname' \n\t option optionname 'optionvalue'\n".to_string(),
                vec![
                    TokenItem {
                        typ: TokenItemType::Config,
                        val: "config".to_string(),
                        pos: 0,
                     },
                    TokenItem {
                        typ: TokenItemType::Ident,
                        val: "sectiontype".to_string(),
                        pos: 0,
                    },
                    TokenItem {
                        typ: TokenItemType::String,
                        val: "sectionname".to_string(),
                        pos: 0,
                    },
                    TokenItem {
                        typ: TokenItemType::Option,
                        val: "option".to_string(),
                        pos: 0,
                    },
                    TokenItem {
                        typ: TokenItemType::Ident,
                        val: "optionname".to_string(),
                        pos: 0,
                    },
                    TokenItem {
                        typ: TokenItemType::String,
                        val: "optionvalue".to_string(),
                        pos: 0,
                    },
                ],
            ),
            (
                "export", 
                "package \"pkgname\"\n config empty \n config squoted 'sqname'\n config dquoted \"dqname\"\n config multiline 'line1\\\n\tline2'\n".to_string(),
                vec![
                    TokenItem {
                        typ: TokenItemType::Package, 
                        val: "package".to_string(), 
                        pos: 0
                    },
                    TokenItem {
                        typ: TokenItemType::String, 
                        val: "pkgname".to_string(), 
                        pos: 0
                    },
                    TokenItem {
                        typ: TokenItemType::Config, 
                        val: "config".to_string(), 
                        pos: 0
                    },
                    TokenItem {
                        typ: TokenItemType::Ident, 
                        val: "empty".to_string(), 
                        pos: 0
                    },
                    TokenItem {
                        typ: TokenItemType::Config, 
                        val: "config".to_string(), 
                        pos: 0
                    },
                    TokenItem {
                        typ: TokenItemType::Ident, 
                        val: "squoted".to_string(), 
                        pos: 0
                    },
                    TokenItem {
                        typ: TokenItemType::String, 
                        val: "sqname".to_string(), 
                        pos: 0
                    },
                    TokenItem {
                        typ: TokenItemType::Config, 
                        val: "config".to_string(), 
                        pos: 0
                    },
                    TokenItem {
                        typ: TokenItemType::Ident, 
                        val: "dquoted".to_string(), 
                        pos: 0
                    },
                    TokenItem {
                        typ: TokenItemType::String, 
                        val: "dqname".to_string(), 
                        pos: 0
                    },
                    TokenItem {
                        typ: TokenItemType::Config, 
                        val: "config".to_string(), 
                        pos: 0
                    },
                    TokenItem {
                        typ: TokenItemType::Ident, 
                        val: "multiline".to_string(), 
                        pos: 0
                    },
                    TokenItem {
                        typ: TokenItemType::String, 
                        val: "line1\\\n\tline2".to_string(), 
                        pos: 0
                    },
                ]
            ),
            (
                "unquoted", 
                "config foo bar\noption answer 42\n".to_string(),
            	vec![
                    TokenItem {
                        typ: TokenItemType::Config, 
                        val: "config".to_string(), 
                        pos: 0
                    },
                    TokenItem {
                        typ: TokenItemType::Ident, 
                        val: "foo".to_string(), 
                        pos: 0
                    },
                    TokenItem {
                        typ: TokenItemType::String, 
                        val: "bar".to_string(), 
                        pos: 0
                    },
                    TokenItem {
                        typ: TokenItemType::Option, 
                        val: "option".to_string(), 
                        pos: 0
                    },
                    TokenItem {
                        typ: TokenItemType::Ident, 
                        val: "answer".to_string(), 
                        pos: 0
                    },
                    TokenItem {
                        typ: TokenItemType::String, 
                        val: "42".to_string(), 
                        pos: 0
                    },
                ]
            ),
            (
                "unnamed", 
                "\nconfig foo named\n\toption pos '0'\n\toption unnamed '0'\n\tlist list 0\n\nconfig foo\n\toption pos '1'\n\toption unnamed '1'\n\tlist list 10\n\nconfig foo\n\toption pos '2'\n\toption unnamed '1'\n\tlist list 20\n\nconfig foo named\n\toption pos '3'\n\toption unnamed '0'\n\tlist list 30\n".to_string(), 
                vec![
                    TokenItem {
                        typ: TokenItemType::Config, 
                        val: "config".to_string(), 
                        pos: 0
                    },
                    TokenItem {
                        typ: TokenItemType::Ident, 
                        val: "foo".to_string(), 
                        pos: 0
                    },
                    TokenItem {
                        typ: TokenItemType::String, 
                        val: "named".to_string(), 
                        pos: 0
                    },
                    TokenItem {
                        typ: TokenItemType::Option, 
                        val: "option".to_string(), 
                        pos: 0
                    },
                    TokenItem {
                        typ: TokenItemType::Ident, 
                        val: "pos".to_string(), 
                        pos: 0
                    },
                    TokenItem {
                        typ: TokenItemType::String, 
                        val: "0".to_string(), 
                        pos: 0
                    },
                    TokenItem {
                        typ: TokenItemType::Option, 
                        val: "option".to_string(), 
                        pos: 0
                    },
                    TokenItem {
                        typ: TokenItemType::Ident, 
                        val: "unnamed".to_string(), 
                        pos: 0
                    },
                    TokenItem {
                        typ: TokenItemType::String, 
                        val: "0".to_string(), 
                        pos: 0
                    },
                    TokenItem {
                        typ: TokenItemType::List, 
                        val: "list".to_string(), 
                        pos: 0
                    },
                    TokenItem {
                        typ: TokenItemType::Ident, 
                        val: "list".to_string(), 
                        pos: 0
                    },
                    TokenItem {
                        typ: TokenItemType::String, 
                        val: "0".to_string(), 
                        pos: 0
                    },

                    TokenItem {
                        typ: TokenItemType::Config, 
                        val: "config".to_string(), 
                        pos: 0
                    },
                    TokenItem {
                        typ: TokenItemType::Ident, 
                        val: "foo".to_string(), 
                        pos: 0
                    },
                    TokenItem {
                        typ: TokenItemType::Option, 
                        val: "option".to_string(), 
                        pos: 0
                    },
                    TokenItem {
                        typ: TokenItemType::Ident, 
                        val: "pos".to_string(), 
                        pos: 0
                    },
                    TokenItem {
                        typ: TokenItemType::String, 
                        val: "1".to_string(), 
                        pos: 0
                    },
                    TokenItem {
                        typ: TokenItemType::Option, 
                        val: "option".to_string(), 
                        pos: 0
                    },
                    TokenItem {
                        typ: TokenItemType::Ident, 
                        val: "unnamed".to_string(), 
                        pos: 0
                    },
                    TokenItem {
                        typ: TokenItemType::String, 
                        val: "1".to_string(), 
                        pos: 0
                    },
                    TokenItem {
                        typ: TokenItemType::List, 
                        val: "list".to_string(), 
                        pos: 0
                    },
                    TokenItem {
                        typ: TokenItemType::Ident, 
                        val: "list".to_string(), 
                        pos: 0
                    },
                    TokenItem {
                        typ: TokenItemType::String, 
                        val: "10".to_string(), 
                        pos: 0
                    },

                    TokenItem {
                        typ: TokenItemType::Config, 
                        val: "config".to_string(), 
                        pos: 0
                    },
                    TokenItem {
                        typ: TokenItemType::Ident, 
                        val: "foo".to_string(), 
                        pos: 0
                    },
                    TokenItem {
                        typ: TokenItemType::Option, 
                        val: "option".to_string(), 
                        pos: 0
                    },
                    TokenItem {
                        typ: TokenItemType::Ident, 
                        val: "pos".to_string(), 
                        pos: 0
                    },
                    TokenItem {
                        typ: TokenItemType::String, 
                        val: "2".to_string(), 
                        pos: 0
                    },
                    TokenItem {
                        typ: TokenItemType::Option, 
                        val: "option".to_string(), 
                        pos: 0
                    },
                    TokenItem {
                        typ: TokenItemType::Ident, 
                        val: "unnamed".to_string(), 
                        pos: 0
                    },
                    TokenItem {
                        typ: TokenItemType::String, 
                        val: "1".to_string(), 
                        pos: 0
                    },
                    TokenItem {
                        typ: TokenItemType::List, 
                        val: "list".to_string(), 
                        pos: 0
                    },
                    TokenItem {
                        typ: TokenItemType::Ident, 
                        val: "list".to_string(), 
                        pos: 0
                    },
                    TokenItem {
                        typ: TokenItemType::String, 
                        val: "20".to_string(), 
                        pos: 0
                    },

                    TokenItem {
                        typ: TokenItemType::Config, 
                        val: "config".to_string(), 
                        pos: 0
                    },
                    TokenItem {
                        typ: TokenItemType::Ident, 
                        val: "foo".to_string(), 
                        pos: 0
                    },
                    TokenItem {
                        typ: TokenItemType::String, 
                        val: "named".to_string(), 
                        pos: 0
                    },
                    TokenItem {
                        typ: TokenItemType::Option, 
                        val: "option".to_string(), 
                        pos: 0
                    },
                    TokenItem {
                        typ: TokenItemType::Ident, 
                        val: "pos".to_string(), 
                        pos: 0
                    },
                    TokenItem {
                        typ: TokenItemType::String, 
                        val: "3".to_string(), 
                        pos: 0
                    },
                    TokenItem {
                        typ: TokenItemType::Option, 
                        val: "option".to_string(), 
                        pos: 0
                    },
                    TokenItem {
                        typ: TokenItemType::Ident, 
                        val: "unnamed".to_string(), 
                        pos: 0
                    },
                    TokenItem {
                        typ: TokenItemType::String, 
                        val: "0".to_string(), 
                        pos: 0
                    },
                    TokenItem {
                        typ: TokenItemType::List, 
                        val: "list".to_string(), 
                        pos: 0
                    },
                    TokenItem {
                        typ: TokenItemType::Ident, 
                        val: "list".to_string(), 
                        pos: 0
                    },
                    TokenItem {
                        typ: TokenItemType::String, 
                        val: "30".to_string(), 
                        pos: 0
                    },
                ]
            ),
            (
                "hyphenated", 
                "\nconfig wifi-device wl0\n\toption type 'broadcom'\n\toption channel '6'\n\nconfig wifi-iface wifi0\n\toption device 'wl0'\n\toption mode 'ap'\n".to_string(),
                vec![
            	    TokenItem {
                        typ: TokenItemType::Config, 
                        val: "config".to_string(), 
                        pos: 0
                    },
                    TokenItem {
                        typ: TokenItemType::Ident, 
                        val: "wifi-device".to_string(), 
                        pos: 0
                    },
                    TokenItem {
                        typ: TokenItemType::String, 
                        val: "wl0".to_string(), 
                        pos: 0
                    },
                    TokenItem {
                        typ: TokenItemType::Option, 
                        val: "option".to_string(), 
                        pos: 0
                    },
                    TokenItem {
                        typ: TokenItemType::Ident, 
                        val: "type".to_string(), 
                        pos: 0
                    },
                    TokenItem {
                        typ: TokenItemType::String, 
                        val: "broadcom".to_string(), 
                        pos: 0
                    },
                    TokenItem {
                        typ: TokenItemType::Option, 
                        val: "option".to_string(), 
                        pos: 0
                    },
                    TokenItem {
                        typ: TokenItemType::Ident, 
                        val: "channel".to_string(), 
                        pos: 0
                    },
                    TokenItem {
                        typ: TokenItemType::String, 
                        val: "6".to_string(), 
                        pos: 0
                    },
                    TokenItem {
                        typ: TokenItemType::Config, 
                        val: "config".to_string(), 
                        pos: 0
                    },
                    TokenItem {
                        typ: TokenItemType::Ident, 
                        val: "wifi-iface".to_string(), 
                        pos: 0
                    },
                    TokenItem {
                        typ: TokenItemType::String, 
                        val: "wifi0".to_string(), 
                        pos: 0
                    },
                    TokenItem {
                        typ: TokenItemType::Option, 
                        val: "option".to_string(), 
                        pos: 0
                    },
                    TokenItem {
                        typ: TokenItemType::Ident, 
                        val: "device".to_string(), 
                        pos: 0
                    },
                    TokenItem {
                        typ: TokenItemType::String, 
                        val: "wl0".to_string(), 
                        pos: 0
                    },
                    TokenItem {
                        typ: TokenItemType::Option, 
                        val: "option".to_string(), 
                        pos: 0
                    },
                    TokenItem {
                        typ: TokenItemType::Ident, 
                        val: "mode".to_string(), 
                        pos: 0
                    },
                    TokenItem {
                        typ: TokenItemType::String, 
                        val: "ap".to_string(), 
                        pos: 0
                    }
                ]
            ),
            (
                "commented", 
                "\n# heading\n\n# another heading\nconfig foo\n\toption opt1 1\n\t# option opt1 2\n\toption opt2 3 # baa\n\toption opt3 hello\n\n# a comment block spanning\n# multiple lines, surrounded\n# by empty lines\n\n# eof\n".to_string(), 
                vec![
                    TokenItem {
                        typ: TokenItemType::Config, 
                        val: "config".to_string(), 
                        pos: 0
                    },
                    TokenItem {
                        typ: TokenItemType::Ident, 
                        val: "foo".to_string(), 
                        pos: 0
                    },
                    TokenItem {
                        typ: TokenItemType::Option, 
                        val: "option".to_string(), 
                        pos: 0
                    },
                    TokenItem {
                        typ: TokenItemType::Ident, 
                        val: "opt1".to_string(), 
                        pos: 0
                    },
                    TokenItem {
                        typ: TokenItemType::String, 
                        val: "1".to_string(), 
                        pos: 0
                    },
                    TokenItem {
                        typ: TokenItemType::Option, 
                        val: "option".to_string(), 
                        pos: 0
                    },
                    TokenItem {
                        typ: TokenItemType::Ident, 
                        val: "opt2".to_string(), 
                        pos: 0
                    },
                    TokenItem {
                        typ: TokenItemType::String, 
                        val: "3".to_string(), 
                        pos: 0
                    },
                    TokenItem {
                        typ: TokenItemType::Option, 
                        val: "option".to_string(), 
                        pos: 0
                    },
                    TokenItem {
                        typ: TokenItemType::Ident, 
                        val: "opt3".to_string(), 
                        pos: 0
                    },
                    TokenItem {
                        typ: TokenItemType::String, 
                        val: "hello".to_string(), 
                        pos: 0
                    }
                ]
            ),
            (
                "invalid", 
                "\n<?xml version=\"1.0\">\n<error message=\"not a UCI file\" />\n".to_string(), 
                vec![
                    TokenItem {
                        typ: TokenItemType::Error, 
                        val: "config: invalid, expected keyword (package, config, option, list) or eof".to_string(), 
                        pos: 0
                    }
                ],
            ),
            (
                "pkg invalid", 
                "\n package\n".to_string(), 
                vec![
                    TokenItem {
                        typ: TokenItemType::Package, 
                        val: "package".to_string(), 
                        pos: 0
                    },
                    TokenItem {
                        typ: TokenItemType::Error, 
                        val: "config: pkg invalid, incomplete package name".to_string(), 
                        pos: 0
                    },
                ],
            ),
            (
                "unterminated quoted string", 
                "\nconfig foo \"bar\n".to_string(), 
                vec![
            		TokenItem {
                        typ: TokenItemType::Config, 
                        val: "config".to_string(), 
                        pos: 0
                    },
                    TokenItem {
                        typ: TokenItemType::Ident, 
                        val: "foo".to_string(), 
                        pos: 0
                    },
                    TokenItem {
                        typ: TokenItemType::Error, 
                        val: "config: unterminated quoted string, unterminated quoted string".to_string(), 
                        pos: 0
                    },
            	]
            ),
            (
                "unterminated unquoted string", 
                "\nconfig foo\n\toption opt opt\\\n".to_string(),  
                vec![
            		TokenItem {
                        typ: TokenItemType::Config, 
                        val: "config".to_string(), 
                        pos: 0
                    },
                    TokenItem {
                        typ: TokenItemType::Ident, 
                        val: "foo".to_string(), 
                        pos: 0
                    },
                    TokenItem {
                        typ: TokenItemType::Option, 
                        val: "option".to_string(), 
                        pos: 0
                    },
                    TokenItem {
                        typ: TokenItemType::Ident, 
                        val: "opt".to_string(), 
                        pos: 0
                    },
                    TokenItem {
                        typ: TokenItemType::Error, 
                        val: "config: unterminated unquoted string, unterminated unquoted string".to_string(), 
                        pos: 0
                    },
            	]
            ),
        ];

        for test_case in test_cases {
            let (name, input, expected) = test_case;
            let mut lex = Lexer::new(name, input);
            let mut idx = 0;
            loop {
                let item = lex.next_item();
                if item.typ == TokenItemType::Eof {
                    break;
                };
                assert_eq!(item.typ, expected[idx].typ);
                assert_eq!(item.val, expected[idx].val);
                idx += 1;
            }

            assert_eq!(expected.len(), idx);
        }
    }
