use std::{collections::VecDeque, vec};

use crate::utils::{Error, Result};

use self::{
    lexer::Lexer,
    token::{ScanTokenType, Token, TokenItem, TokenItemType},
};

use super::tree::{
    uci_config::UciConfig,
    uci_option::{UciOption, UciOptionType},
    uci_section::UciSection,
};

pub mod lexer;
pub mod token;

pub struct Scanner {
    lexer: Lexer,
    state: Option<ScannerState>,
    last: Option<TokenItem>,
    curr: Vec<TokenItem>,
    tokens: Option<VecDeque<Token>>,
}

trait ScannerStateMachine {
    fn action(&mut self) -> Option<ScannerState>;
    fn scan_start(&mut self) -> Option<ScannerState>;
    fn scan_package(&mut self) -> Option<ScannerState>;
    fn scan_section(&mut self) -> Option<ScannerState>;
    fn scan_option(&mut self) -> Option<ScannerState>;
    fn scan_option_name(&mut self) -> Option<ScannerState>;
    fn scan_list_name(&mut self) -> Option<ScannerState>;
    fn scan_list_value(&mut self) -> Option<ScannerState>;
    fn scan_option_value(&mut self) -> Option<ScannerState>;
}

#[derive(Debug)]
enum ScannerState {
    ScanStart,
    ScanPackage,
    ScanSection,
    ScanOption,
    ScanOptionName,
    ScanOptionValue,
    ScanListName,
    ScanListValue,
}

impl Scanner {
    fn new(name: &str, input: String) -> Self {
        Scanner {
            lexer: Lexer::new(&name, input),
            state: Some(ScannerState::ScanStart),
            curr: vec![],
            tokens: Some(VecDeque::new()),
            last: None,
        }
    }

    fn eof(&self) -> Option<Token> {
        None
    }

    fn stop(&mut self) -> Option<Token> {
        let mut tok = self.eof();
        if self.tokens.is_none() {
            return tok;
        } else {
            self.lexer.stop();
            tok = self.tokens.as_mut().unwrap().pop_front();
            self.tokens = None;
            return tok;
        }
    }

    fn next_item(&mut self) -> TokenItem {
        if let Some(it) = self.last.take() {
            it
        } else {
            self.lexer.next_item()
        }
    }

    fn peek(&mut self) -> TokenItem {
        let it = self.next_item();
        self.backup(&it);
        it
    }

    fn backup(&mut self, it: &TokenItem) {
        self.last = Some((*it).clone());
    }

    fn accept_once(&mut self, it: TokenItemType) -> bool {
        let tok = self.next_item();
        if tok.typ == it {
            self.curr.push(tok);
            return true;
        } else {
            self.backup(&tok);
            return false;
        }
    }

    fn emit(&mut self, typ: ScanTokenType) {
        self.tokens.as_mut().unwrap().push_back(Token {
            typ: typ,
            items: self.curr.clone(),
        });
        self.curr = vec![];
    }

    fn emit_error(&mut self, error: &str) -> Option<ScannerState> {
        self.tokens.as_mut().unwrap().push_back(Token {
            typ: ScanTokenType::TokenError,
            items: vec![TokenItem {
                typ: TokenItemType::TokenError,
                val: error.to_owned(),
                pos: 0,
            }],
        });
        return None;
    }
}

impl Iterator for Scanner {
    type Item = Token;
    fn next(&mut self) -> Option<Self::Item> {
        while self.state.is_some() {
            if self.tokens.is_none() {
                return None;
            } else if let Some(token) = self.tokens.as_mut().unwrap().pop_front() {
                return Some(token);
            } else {
                self.state = self.action();
            }
        }
        self.stop()
    }
}

impl ScannerStateMachine for Scanner {
    fn action(&mut self) -> Option<ScannerState> {
        match self.state {
            Some(ScannerState::ScanListName) => self.scan_list_name(),
            Some(ScannerState::ScanListValue) => self.scan_list_value(),
            Some(ScannerState::ScanOption) => self.scan_option(),
            Some(ScannerState::ScanOptionName) => self.scan_option_name(),
            Some(ScannerState::ScanOptionValue) => self.scan_option_value(),
            Some(ScannerState::ScanPackage) => self.scan_package(),
            Some(ScannerState::ScanSection) => self.scan_section(),
            Some(ScannerState::ScanStart) => self.scan_start(),
            None => None,
        }
    }
    fn scan_start(&mut self) -> Option<ScannerState> {
        match self.next_item() {
            it if it.typ == TokenItemType::TokenPackage => Some(ScannerState::ScanPackage),
            it if it.typ == TokenItemType::TokenConfig => Some(ScannerState::ScanSection),
            it if it.typ == TokenItemType::TokenError => self.emit_error(&it.val),
            it if it.typ == TokenItemType::TokenEOF => None,
            _ => self.emit_error("expected package or config token"),
        }
    }

    fn scan_package(&mut self) -> Option<ScannerState> {
        match self.next_item() {
            it if it.typ == TokenItemType::TokenString => {
                self.curr.push(it);
                self.emit(ScanTokenType::TokenPackage);
                Some(ScannerState::ScanStart)
            }
            it if it.typ == TokenItemType::TokenError => self.emit_error(&it.val),
            _ => self.emit_error("expected string value while parsing package"),
        }
    }

    fn scan_section(&mut self) -> Option<ScannerState> {
        match self.next_item() {
            it if it.typ == TokenItemType::TokenIdent => {
                self.curr.push(it);
                let tok = self.peek();
                if tok.typ == TokenItemType::TokenString {
                    self.accept_once(TokenItemType::TokenString);
                };
                self.emit(ScanTokenType::TokenSection);
                Some(ScannerState::ScanOption)
            }
            it if it.typ == TokenItemType::TokenError => self.emit_error(&it.val),
            _ => self.emit_error("expected identifier while parsing config section"),
        }
    }

    fn scan_option(&mut self) -> Option<ScannerState> {
        match self.next_item() {
            it if it.typ == TokenItemType::TokenOption => Some(ScannerState::ScanOptionName),
            it if it.typ == TokenItemType::TokenList => Some(ScannerState::ScanListName),
            it if it.typ == TokenItemType::TokenError => self.emit_error(&it.val),
            it => {
                self.backup(&it);
                Some(ScannerState::ScanStart)
            }
        }
    }

    fn scan_option_name(&mut self) -> Option<ScannerState> {
        if self.accept_once(TokenItemType::TokenIdent) {
            Some(ScannerState::ScanOptionValue)
        } else {
            self.emit_error("expected option name")
        }
    }

    fn scan_list_name(&mut self) -> Option<ScannerState> {
        if self.accept_once(TokenItemType::TokenIdent) {
            Some(ScannerState::ScanListValue)
        } else {
            self.emit_error("expected option name")
        }
    }

    fn scan_option_value(&mut self) -> Option<ScannerState> {
        match self.next_item() {
            it if it.typ == TokenItemType::TokenString => {
                self.curr.push(it);
                self.emit(ScanTokenType::TokenOption);
                Some(ScannerState::ScanOption)
            }
            it if it.typ == TokenItemType::TokenError => self.emit_error(&it.val),
            _ => self.emit_error("expected option value"),
        }
    }

    fn scan_list_value(&mut self) -> Option<ScannerState> {
        match self.next_item() {
            it if it.typ == TokenItemType::TokenString => {
                self.curr.push(it);
                self.emit(ScanTokenType::TokenList);
                Some(ScannerState::ScanOption)
            }
            it if it.typ == TokenItemType::TokenError => self.emit_error(&it.val),
            _ => self.emit_error("expected option value"),
        }
    }
}

pub fn uci_parse(name: &str, input: String) -> Result<UciConfig> {
    let mut scanner = Scanner::new(name, input);
    match scanner.try_fold(
        (UciConfig::new(name), None),
        |(mut cfg, mut sec): (UciConfig, Option<UciSection>),
         tok: Token|
         -> Result<(UciConfig, Option<UciSection>)> {
            match tok.typ {
                ScanTokenType::TokenError => {
                    return Err(Error::new(format!("parse error: {}", tok.items[0].val)));
                }
                ScanTokenType::TokenPackage => {
                    return Err(Error::new(
                        "UCI packages syntax are not yet supported".to_string(),
                    ));
                }
                ScanTokenType::TokenSection => {
                    let sec_typ = tok.items[0].val.to_string();
                    if tok.items.len() == 2 {
                        sec = Some(
                            cfg.merge(UciSection::new(sec_typ, tok.items[1].val.to_string()))
                                .clone(),
                        );
                    } else {
                        sec = Some(cfg.add(UciSection::new(sec_typ, "".to_string())).clone());
                    }
                }
                ScanTokenType::TokenOption => {
                    let name = tok.items[0].val.to_string();
                    let val = tok.items[1].val.to_string();

                    if let Some(opt) = sec.as_mut().unwrap().get_mut(&name) {
                        opt.set_values(vec![val]);
                    } else {
                        sec.as_mut().map(|s| {
                            s.add(UciOption::new(name, UciOptionType::TypeOption, vec![val]))
                        });
                    };
                }
                ScanTokenType::TokenList => {
                    let name = tok.items[0].val.to_string();
                    let val = tok.items[1].val.to_string();

                    if let Some(opt) = sec.as_mut().unwrap().get_mut(&name) {
                        opt.merge_values(vec![val]);
                    } else {
                        sec.as_mut().map(|s| {
                            s.add(UciOption::new(name, UciOptionType::TypeList, vec![val]))
                        });
                    };
                }
                ScanTokenType::TokenEOF => {}
            };
            Ok((cfg, sec))
        },
    ) {
        Ok((cfg, _)) => Ok(cfg),
        Err(err) => {
            scanner.stop();
            Err(err)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parser() {
        let test_cases = vec![
            (
                "empty1",
                format!(""),
                vec![]
            ),
            (
                "empty2",
                format!("  \n\t\n\n \n "),
                vec![]
            ),
            (
                "simple",
                format!("config sectiontype 'sectionname' \n\t option optionname 'optionvalue'\n"),
                vec![
                    Token {
                        typ: ScanTokenType::TokenSection,
                        items: vec![
                            TokenItem {
                                typ: TokenItemType::TokenIdent,
                                val: format!("sectiontype"),
                                pos: 0,
                            },
                            TokenItem {
                                typ: TokenItemType::TokenString,
                                val: format!("sectionname"),
                                pos: 0,
                            },
                        ],
                    },
                    Token {
                        typ: ScanTokenType::TokenOption,
                        items: vec![
                            TokenItem {
                                typ: TokenItemType::TokenIdent,
                                val: format!("optionname"),
                                pos: 0,
                            },
                            TokenItem {
                                typ: TokenItemType::TokenString,
                                val: format!("optionvalue"),
                                pos: 0,
                            },
                        ],
                    },
                ],
            ),
            (
                "export",
                format!("package \"pkgname\"\n config empty \n config squoted 'sqname'\n config dquoted \"dqname\"\n config multiline 'line1\\\n\tline2'\n"),
                vec![
                    Token {
                        typ: ScanTokenType::TokenPackage,
                        items: vec![
                            TokenItem {
                                typ: TokenItemType::TokenString,
                                val: format!("pkgname"), pos: 0
                            },
                        ]
                    },
                    Token {
                        typ: ScanTokenType::TokenSection,
                        items: vec![
                            TokenItem {
                                typ: TokenItemType::TokenIdent,
                                val: format!("empty"), pos: 0
                            },
                        ]
                    },
                    Token {
                        typ:ScanTokenType::TokenSection,
                        items:vec![
                            TokenItem {
                                typ: TokenItemType::TokenIdent,
                                val: format!("squoted"),
                                pos: 0
                            },
                            TokenItem {
                                typ: TokenItemType::TokenString,
                                val: format!("sqname"),
                                pos: 0
                            }
                        ]
                    },
                    Token {
                        typ: ScanTokenType::TokenSection,
                        items: vec![
                            TokenItem {
                                typ: TokenItemType::TokenIdent,
                                val: format!("dquoted"),
                                pos: 0
                            },
                            TokenItem {
                                typ: TokenItemType::TokenString,
                                val: format!("dqname"),
                                pos: 0
                            }
                        ]
                    },
                    Token {
                        typ: ScanTokenType::TokenSection,
                        items: vec![
                            TokenItem {
                                typ: TokenItemType::TokenIdent,
                                val: format!("multiline"),
                                pos: 0
                            },
                            TokenItem {
                                typ: TokenItemType::TokenString,
                                val: format!("line1\\\n\tline2"),
                                pos: 0
                            },
                        ]
                    },
                ]
            ),
            (
                "unquoted",
                format!("config foo bar\noption answer 42\n"),
                vec![
                    Token {
                        typ: ScanTokenType::TokenSection,
                        items: vec![
                            TokenItem {
                                typ: TokenItemType::TokenIdent,
                                val: format!("foo"),
                                pos: 0
                            },
                            TokenItem {
                                typ: TokenItemType::TokenString,
                                val: format!("bar"),
                                pos: 0
                            },
                        ]
                    },
                    Token {
                        typ: ScanTokenType::TokenOption,
                        items: vec![
                            TokenItem {
                                typ: TokenItemType::TokenIdent,
                                val: format!("answer"),
                                pos: 0
                            },
                            TokenItem {
                                typ: TokenItemType::TokenString,
                                val: format!("42"),
                                pos: 0
                            }
                        ]
                    },
                ]
            ),
            (
                "unnamed",
                format!("\nconfig foo named\n\toption pos '0'\n\toption unnamed '0'\n\tlist list 0\n\nconfig foo\n\toption pos '1'\n\toption unnamed '1'\n\tlist list 10\n\nconfig foo\n\toption pos '2'\n\toption unnamed '1'\n\tlist list 20\n\nconfig foo named\n\toption pos '3'\n\toption unnamed '0'\n\tlist list 30\n"),
                vec![
                    Token {
                        typ: ScanTokenType::TokenSection,
                        items: vec![
                            TokenItem {
                                typ: TokenItemType::TokenIdent,
                                val: format!("foo"),
                                pos: 0
                            },
                            TokenItem {
                                typ: TokenItemType::TokenString,
                                val: format!("named"),
                                pos: 0
                            }
                        ]
                    },
                    Token {
                        typ: ScanTokenType::TokenOption,
                        items: vec![
                            TokenItem {
                                typ: TokenItemType::TokenIdent,
                                val: format!("pos"),
                                pos: 0
                            },
                            TokenItem{
                                typ: TokenItemType::TokenString,
                                val: format!("0"),
                                pos: 0
                            },
                        ]
                    },
                    Token {
                        typ: ScanTokenType::TokenOption,
                        items: vec![
                            TokenItem {
                                typ: TokenItemType::TokenIdent,
                                val: format!("unnamed"),
                                pos: 0
                            },
                            TokenItem {
                                typ: TokenItemType::TokenString,
                                val: format!("0"),
                                pos: 0
                            },
                        ]
                    },
                    Token {
                        typ: ScanTokenType::TokenList,
                        items: vec![
                            TokenItem {
                                typ: TokenItemType::TokenIdent,
                                val: format!("list"),
                                pos: 0
                            },
                            TokenItem {
                                typ: TokenItemType::TokenString,
                                val: format!("0"),
                                pos: 0
                            },
                        ]
                    },
                    Token {
                        typ: ScanTokenType::TokenSection,
                        items: vec![
                            TokenItem {
                                typ: TokenItemType::TokenIdent,
                                val: format!("foo"),
                                pos: 0
                            }
                        ]
                    },
                    Token {
                        typ: ScanTokenType::TokenOption,
                        items: vec![
                            TokenItem {
                                typ: TokenItemType::TokenIdent,
                                val: format!("pos"),
                                pos: 0
                            },
                            TokenItem {
                                typ: TokenItemType::TokenString,
                                val: format!("1"),
                                pos: 0
                            },
                        ]
                    },
                    Token {
                        typ: ScanTokenType::TokenOption,
                        items: vec![
                            TokenItem {
                                typ: TokenItemType::TokenIdent,
                                val: format!("unnamed"),
                                pos: 0
                            },
                            TokenItem {
                                typ: TokenItemType::TokenString,
                                val: format!("1"),
                                pos: 0
                            },
                        ]
                    },
                    Token {
                        typ: ScanTokenType::TokenList,
                        items: vec![
                            TokenItem {
                                typ: TokenItemType::TokenIdent,
                                val: format!("list"), pos: 0
                            },
                            TokenItem {
                                typ: TokenItemType::TokenString,
                                val: format!("10"),
                                pos: 0
                            },
                        ]
                    },
                    Token {
                        typ: ScanTokenType::TokenSection,
                        items: vec![
                            TokenItem {
                                typ: TokenItemType::TokenIdent,
                                val: format!("foo"),
                                pos: 0
                            },
                        ]
                    },
                    Token {
                        typ: ScanTokenType::TokenOption,
                        items: vec![
                            TokenItem {
                                typ: TokenItemType::TokenIdent,
                                val: format!("pos"),
                                pos: 0
                            },
                            TokenItem {
                                typ: TokenItemType::TokenString,
                                val: format!("2"),
                                pos: 0
                            },
                        ]
                    },
                    Token {
                        typ: ScanTokenType::TokenOption,
                        items: vec![
                            TokenItem {
                                typ: TokenItemType::TokenIdent,
                                val: format!("unnamed"),
                                pos: 0
                            },
                            TokenItem {
                                typ: TokenItemType::TokenString,
                                val: format!("1"),
                                pos: 0
                            },
                        ]
                    },
                    Token {
                        typ: ScanTokenType::TokenList,
                        items: vec![
                            TokenItem {
                                typ: TokenItemType::TokenIdent,
                                val: format!("list"),
                                pos: 0
                            },
                            TokenItem {
                                typ: TokenItemType::TokenString,
                                val: format!("20"),
                                pos: 0
                            },
                        ]
                    },
                    Token {
                        typ: ScanTokenType::TokenSection,
                        items: vec![
                            TokenItem {
                                typ: TokenItemType::TokenIdent,
                                val: format!("foo"),
                                pos: 0
                            },
                            TokenItem {
                                typ: TokenItemType::TokenString,
                                val: format!("named"),
                                pos: 0
                            },
                        ]
                    },
                    Token {
                        typ: ScanTokenType::TokenOption,
                        items: vec![
                            TokenItem {
                                typ: TokenItemType::TokenIdent,
                                val: format!("pos"),
                                pos: 0
                            },
                            TokenItem {
                                typ: TokenItemType::TokenString,
                                val: format!("3"),
                                pos: 0
                            },
                        ]
                    },
                    Token {
                        typ: ScanTokenType::TokenOption,
                        items: vec![
                            TokenItem {
                                typ: TokenItemType::TokenIdent,
                                val: format!("unnamed"),
                                pos: 0
                            },
                            TokenItem {
                                typ: TokenItemType::TokenString,
                                val: format!("0"),
                                pos: 0
                            },
                        ]
                    },
                    Token {
                        typ: ScanTokenType::TokenList,
                        items: vec![
                            TokenItem {
                                typ: TokenItemType::TokenIdent,
                                val: format!("list"),
                                pos: 0
                            },
                            TokenItem {
                                typ: TokenItemType::TokenString,
                                val: format!("30"),
                                pos: 0
                            },
                        ]
                    },
                ]
            ),
            (
                "hyphenated",
                format!("\nconfig wifi-device wl0\n\toption type 'broadcom'\n\toption channel '6'\n\nconfig wifi-iface wifi0\n\toption device 'wl0'\n\toption mode 'ap'\n"),
                vec![
                    Token {
                        typ: ScanTokenType::TokenSection,
                        items: vec![
                            TokenItem {
                                typ: TokenItemType::TokenIdent,
                                val: format!("wifi-device"),
                                pos: 0
                            },
                            TokenItem {
                                typ: TokenItemType::TokenString,
                                val: format!("wl0"),
                                pos: 0
                            },
                        ]
                    },
                    Token {
                        typ: ScanTokenType::TokenOption,
                        items: vec![
                            TokenItem {
                                typ: TokenItemType::TokenIdent,
                                val: format!("type"),
                                pos: 0
                            },
                            TokenItem {
                                typ: TokenItemType::TokenString,
                                val: format!("broadcom"),
                                pos: 0
                            },
                        ]
                    },
                    Token {
                        typ: ScanTokenType::TokenOption,
                        items: vec![
                            TokenItem {
                                typ: TokenItemType::TokenIdent,
                                val: format!("channel"),
                                pos: 0
                            },
                            TokenItem {
                                typ: TokenItemType::TokenString,
                                val: format!("6"),
                                pos: 0
                            },
                        ]
                    },
                    Token{
                        typ: ScanTokenType::TokenSection,
                        items: vec![
                            TokenItem {
                                typ: TokenItemType::TokenIdent,
                                val: format!("wifi-iface"),
                                pos: 0
                            },
                            TokenItem {
                                typ: TokenItemType::TokenString,
                                val: format!("wifi0"),
                                pos: 0
                            },
                        ]
                    },
                    Token {
                        typ: ScanTokenType::TokenOption,
                        items: vec![
                            TokenItem {
                                typ: TokenItemType::TokenIdent,
                                val: format!("device"),
                                pos: 0
                            },
                            TokenItem {
                                typ: TokenItemType::TokenString,
                                val: format!("wl0"),
                                pos: 0
                            },
                        ]
                    },
                    Token {
                        typ: ScanTokenType::TokenOption,
                        items: vec![
                            TokenItem {
                                typ: TokenItemType::TokenIdent,
                                val: format!("mode"),
                                pos: 0
                            },
                            TokenItem {
                                typ: TokenItemType::TokenString,
                                val: format!("ap"),
                                pos: 0
                            }
                        ]
                    },
                ]
            ),
            (
                "commented",
                format!("\n# heading\n\n# another heading\nconfig foo\n\toption opt1 1\n\t# option opt1 2\n\toption opt2 3 # baa\n\toption opt3 hello\n\n# a comment block spanning\n# multiple lines, surrounded\n# by empty lines\n\n# eof\n"),
                vec![
                    Token {
                        typ: ScanTokenType::TokenSection,
                        items: vec![
                            TokenItem {
                                typ: TokenItemType::TokenIdent,
                                val: format!("foo"),
                                pos: 0
                            },
                        ]
                    },
                    Token {
                        typ: ScanTokenType::TokenOption,
                        items: vec![
                            TokenItem {
                                typ: TokenItemType::TokenIdent,
                                val: format!("opt1"),
                                pos: 0
                            },
                            TokenItem {
                                typ: TokenItemType::TokenString,
                                val: format!("1"),
                                pos: 0
                            },
                        ]
                    },
                    Token {
                        typ: ScanTokenType::TokenOption,
                        items: vec![
                            TokenItem {
                                typ: TokenItemType::TokenIdent,
                                val: format!("opt2"),
                                pos: 0
                            },
                            TokenItem {
                                typ: TokenItemType::TokenString,
                                val: format!("3"),
                                pos: 0
                            },
                        ]
                    },
                    Token {
                        typ: ScanTokenType::TokenOption,
                        items: vec![
                            TokenItem {
                                typ: TokenItemType::TokenIdent,
                                val: format!("opt3"),
                                pos: 0
                            },
                            TokenItem {
                                typ: TokenItemType::TokenString,
                                val: format!("hello"),
                                pos: 0
                            }
                        ]
                    },
                ]
            ),
            (
                "invalid",
                format!("\n<?xml version=\"1.0\">\n<error message=\"not a UCI file\" />\n"),
                vec![
                    Token{
                        typ: ScanTokenType::TokenError,
                        items: vec![
                            TokenItem {
                                typ: TokenItemType::TokenError,
                                val: format!("config: invalid, expected keyword (package, config, option, list) or eof"),
                                pos: 0
                            }
                        ]
                    },
                ],
            ),
            (
                "pkg invalid",
                format!("\n package\n"),
                vec![
                    Token {
                        typ: ScanTokenType::TokenError,
                        items: vec![
                            TokenItem {
                                typ: TokenItemType::TokenError,
                                val: format!("config: pkg invalid, incomplete package name"),
                                pos: 0
                            },
                        ]
                    }
                ],
            ),
            (
                "unterminated quoted string",
                format!("\nconfig foo \"bar\n"),
                vec![
                    Token {
                        typ: ScanTokenType::TokenSection,
                        items: vec![
                            TokenItem {
                                typ: TokenItemType::TokenIdent,
                                val: format!("foo"),
                                pos: 0
                            },
                        ]
                    },
                    Token {
                        typ: ScanTokenType::TokenError,
                        items: vec![
                            TokenItem {
                                typ: TokenItemType::TokenError,
                                val: format!("config: unterminated quoted string, unterminated quoted string"),
                                pos: 0
                            }
                        ]
                    }
                ]
            ),
            (
                "unterminated unquoted string",
                format!("\nconfig foo\n\toption opt opt\\\n"),
                vec![
                    Token {
                        typ: ScanTokenType::TokenSection,
                        items: vec![
                            TokenItem {
                                typ: TokenItemType::TokenIdent,
                                val: format!("foo"),
                                pos: 0
                            },
                        ]
                    },
                    Token {
                        typ: ScanTokenType::TokenError,
                        items: vec![
                            TokenItem {
                                typ: TokenItemType::TokenError,
                                val: format!("config: unterminated unquoted string, unterminated unquoted string"),
                                pos: 0
                            },
                        ]
                    }
                ]
            ),
        ];

        for test_case in test_cases {
            let (name, input, expected) = test_case;
            let mut idx = 0;
            let scanner = Scanner::new(name, input);
            for token in scanner {
                assert_eq!(token.typ, expected[idx].typ);
                token
                    .items
                    .iter()
                    .zip(&expected[idx].items)
                    .for_each(|(t1, t2)| {
                        assert_eq!(t1.typ, t2.typ);
                        assert_eq!(t1.val, t2.val);
                    });
                idx += 1;
            }

            assert_eq!(expected.len(), idx);
        }
    }
}
