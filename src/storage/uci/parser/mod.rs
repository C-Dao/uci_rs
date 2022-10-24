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
    Start,
    Package,
    Section,
    Option,
    OptionName,
    OptionValue,
    ListName,
    ListValue,
}

impl Scanner {
    fn new(name: &str, input: String) -> Self {
        Scanner {
            lexer: Lexer::new(name, input),
            state: Some(ScannerState::Start),
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
            tok
        } else {
            self.lexer.stop();
            tok = self.tokens.as_mut().unwrap().pop_front();
            self.tokens = None;
            tok
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
            true
        } else {
            self.backup(&tok);
            false
        }
    }

    fn emit(&mut self, typ: ScanTokenType) {
        self.tokens.as_mut().unwrap().push_back(Token {
            typ,
            items: self.curr.clone(),
        });
        self.curr = vec![];
    }

    fn emit_error(&mut self, error: &str) -> Option<ScannerState> {
        self.tokens.as_mut().unwrap().push_back(Token {
            typ: ScanTokenType::Error,
            items: vec![TokenItem {
                typ: TokenItemType::Error,
                val: error.to_owned(),
                pos: 0,
            }],
        });
        None
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
            Some(ScannerState::ListName) => self.scan_list_name(),
            Some(ScannerState::ListValue) => self.scan_list_value(),
            Some(ScannerState::Option) => self.scan_option(),
            Some(ScannerState::OptionName) => self.scan_option_name(),
            Some(ScannerState::OptionValue) => self.scan_option_value(),
            Some(ScannerState::Package) => self.scan_package(),
            Some(ScannerState::Section) => self.scan_section(),
            Some(ScannerState::Start) => self.scan_start(),
            None => None,
        }
    }
    fn scan_start(&mut self) -> Option<ScannerState> {
        match self.next_item() {
            it if it.typ == TokenItemType::Package => Some(ScannerState::Package),
            it if it.typ == TokenItemType::Config => Some(ScannerState::Section),
            it if it.typ == TokenItemType::Error => self.emit_error(&it.val),
            it if it.typ == TokenItemType::EOF => None,
            _ => self.emit_error("expected package or config token"),
        }
    }

    fn scan_package(&mut self) -> Option<ScannerState> {
        match self.next_item() {
            it if it.typ == TokenItemType::String => {
                self.curr.push(it);
                self.emit(ScanTokenType::Package);
                Some(ScannerState::Start)
            }
            it if it.typ == TokenItemType::Error => self.emit_error(&it.val),
            _ => self.emit_error("expected string value while parsing package"),
        }
    }

    fn scan_section(&mut self) -> Option<ScannerState> {
        match self.next_item() {
            it if it.typ == TokenItemType::Ident => {
                self.curr.push(it);
                let tok = self.peek();
                if tok.typ == TokenItemType::String {
                    self.accept_once(TokenItemType::String);
                };
                self.emit(ScanTokenType::Section);
                Some(ScannerState::Option)
            }
            it if it.typ == TokenItemType::Error => self.emit_error(&it.val),
            _ => self.emit_error("expected identifier while parsing config section"),
        }
    }

    fn scan_option(&mut self) -> Option<ScannerState> {
        match self.next_item() {
            it if it.typ == TokenItemType::Option => Some(ScannerState::OptionName),
            it if it.typ == TokenItemType::List => Some(ScannerState::ListName),
            it if it.typ == TokenItemType::Error => self.emit_error(&it.val),
            it => {
                self.backup(&it);
                Some(ScannerState::Start)
            }
        }
    }

    fn scan_option_name(&mut self) -> Option<ScannerState> {
        if self.accept_once(TokenItemType::Ident) {
            Some(ScannerState::OptionValue)
        } else {
            self.emit_error("expected option name")
        }
    }

    fn scan_list_name(&mut self) -> Option<ScannerState> {
        if self.accept_once(TokenItemType::Ident) {
            Some(ScannerState::ListValue)
        } else {
            self.emit_error("expected option name")
        }
    }

    fn scan_option_value(&mut self) -> Option<ScannerState> {
        match self.next_item() {
            it if it.typ == TokenItemType::String => {
                self.curr.push(it);
                self.emit(ScanTokenType::Option);
                Some(ScannerState::Option)
            }
            it if it.typ == TokenItemType::Error => self.emit_error(&it.val),
            _ => self.emit_error("expected option value"),
        }
    }

    fn scan_list_value(&mut self) -> Option<ScannerState> {
        match self.next_item() {
            it if it.typ == TokenItemType::String => {
                self.curr.push(it);
                self.emit(ScanTokenType::List);
                Some(ScannerState::Option)
            }
            it if it.typ == TokenItemType::Error => self.emit_error(&it.val),
            _ => self.emit_error("expected option value"),
        }
    }
}

pub fn uci_parse(name: &str, input: String) -> Result<UciConfig> {
    let mut scanner = Scanner::new(name, input);
    let mut cfg = UciConfig::new(name);
    let mut sec: Option<UciSection> = None;
    match scanner.try_for_each(|tok: Token| -> Result<()> {
        match tok.typ {
            ScanTokenType::Error => {
                return Err(Error::new(format!("parse error: {}", tok.items[0].val)));
            }
            ScanTokenType::Package => {
                let pkg_name = tok.items[0].val.to_string();
                cfg.set_pkg_name(pkg_name);
            }
            ScanTokenType::Section => {
                if sec.is_some() {
                    if let Some(s) =  sec.as_ref(){
                        if s.sec_type != String::new() && s.name != String::new() {
                            cfg.merge(s.clone());
                        } else {
                            cfg.add(s.clone());
                        }
                    };
                };
                if tok.items.len() == 2 {
                    let sec_typ = tok.items[0].val.to_string();
                    let name = tok.items[1].val.to_string();
                    sec = Some(UciSection::new(sec_typ, name));
                } else {
                    let sec_typ = tok.items[0].val.to_string();
                    sec = Some(UciSection::new(sec_typ, "".to_string()));
                }
            }
            ScanTokenType::Option => {
                let name = tok.items[0].val.to_string();
                let val = tok.items[1].val.to_string();

                if let Some(opt) = sec.as_mut().unwrap().get_mut(&name) {
                    opt.set_values(vec![val]);
                } else if let Some(s) = sec.as_mut() { s.add(UciOption::new(name, UciOptionType::TypeOption, vec![val])) };
            }
            ScanTokenType::List => {
                let name = tok.items[0].val.to_string();
                let val = tok.items[1].val.to_string();

                if let Some(opt) = sec.as_mut().unwrap().get_mut(&name) {
                    opt.merge_values(vec![val]);
                } else if let Some(s) = sec.as_mut() { s.add(UciOption::new(name, UciOptionType::TypeList, vec![val])) };
            }
        };
        Ok(())
    }) {
        Ok(_) => {
            if sec.is_some() {
                if let Some(s) = sec.as_ref() {
                    if s.sec_type != String::new() && s.name != String::new() {
                        cfg.merge(s.clone());
                    } else {
                        cfg.add(s.clone());
                    }
            };
            };
            Ok(cfg)
        }
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
    fn test_parser() {
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
                "empty option",
                "config sectiontype 'sectionname' \n\t option optionname ''\n".to_string(),
                vec![
                    Token {
                        typ: ScanTokenType::Section,
                        items: vec![
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
                        ],
                    },
                    Token {
                        typ: ScanTokenType::Option,
                        items: vec![
                            TokenItem {
                                typ: TokenItemType::Ident,
                                val: "optionname".to_string(),
                                pos: 0,
                            },
                            TokenItem {
                                typ: TokenItemType::String,
                                val: String::new(),
                                pos: 0,
                            },
                        ],
                    },
                ],
            ),
            (
                "simple",
                "config sectiontype 'sectionname' \n\t option optionname 'optionvalue'\n".to_string(),
                vec![
                    Token {
                        typ: ScanTokenType::Section,
                        items: vec![
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
                        ],
                    },
                    Token {
                        typ: ScanTokenType::Option,
                        items: vec![
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
                    },
                ],
            ),
            (
                "export",
                "package \"pkgname\"\n config empty \n config squoted 'sqname'\n config dquoted \"dqname\"\n config multiline 'line1\\\n\tline2'\n".to_string(),
                vec![
                    Token {
                        typ: ScanTokenType::Package,
                        items: vec![
                            TokenItem {
                                typ: TokenItemType::String,
                                val: "pkgname".to_string(), pos: 0
                            },
                        ]
                    },
                    Token {
                        typ: ScanTokenType::Section,
                        items: vec![
                            TokenItem {
                                typ: TokenItemType::Ident,
                                val: "empty".to_string(), pos: 0
                            },
                        ]
                    },
                    Token {
                        typ:ScanTokenType::Section,
                        items:vec![
                            TokenItem {
                                typ: TokenItemType::Ident,
                                val: "squoted".to_string(),
                                pos: 0
                            },
                            TokenItem {
                                typ: TokenItemType::String,
                                val: "sqname".to_string(),
                                pos: 0
                            }
                        ]
                    },
                    Token {
                        typ: ScanTokenType::Section,
                        items: vec![
                            TokenItem {
                                typ: TokenItemType::Ident,
                                val: "dquoted".to_string(),
                                pos: 0
                            },
                            TokenItem {
                                typ: TokenItemType::String,
                                val: "dqname".to_string(),
                                pos: 0
                            }
                        ]
                    },
                    Token {
                        typ: ScanTokenType::Section,
                        items: vec![
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
                    },
                ]
            ),
            (
                "unquoted",
                "config foo bar\noption answer 42\n".to_string(),
                vec![
                    Token {
                        typ: ScanTokenType::Section,
                        items: vec![
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
                        ]
                    },
                    Token {
                        typ: ScanTokenType::Option,
                        items: vec![
                            TokenItem {
                                typ: TokenItemType::Ident,
                                val: "answer".to_string(),
                                pos: 0
                            },
                            TokenItem {
                                typ: TokenItemType::String,
                                val: "42".to_string(),
                                pos: 0
                            }
                        ]
                    },
                ]
            ),
            (
                "unnamed",
                "\nconfig foo named\n\toption pos '0'\n\toption unnamed '0'\n\tlist list 0\n\nconfig foo\n\toption pos '1'\n\toption unnamed '1'\n\tlist list 10\n\nconfig foo\n\toption pos '2'\n\toption unnamed '1'\n\tlist list 20\n\nconfig foo named\n\toption pos '3'\n\toption unnamed '0'\n\tlist list 30\n".to_string(),
                vec![
                    Token {
                        typ: ScanTokenType::Section,
                        items: vec![
                            TokenItem {
                                typ: TokenItemType::Ident,
                                val: "foo".to_string(),
                                pos: 0
                            },
                            TokenItem {
                                typ: TokenItemType::String,
                                val: "named".to_string(),
                                pos: 0
                            }
                        ]
                    },
                    Token {
                        typ: ScanTokenType::Option,
                        items: vec![
                            TokenItem {
                                typ: TokenItemType::Ident,
                                val: "pos".to_string(),
                                pos: 0
                            },
                            TokenItem{
                                typ: TokenItemType::String,
                                val: "0".to_string(),
                                pos: 0
                            },
                        ]
                    },
                    Token {
                        typ: ScanTokenType::Option,
                        items: vec![
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
                        ]
                    },
                    Token {
                        typ: ScanTokenType::List,
                        items: vec![
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
                        ]
                    },
                    Token {
                        typ: ScanTokenType::Section,
                        items: vec![
                            TokenItem {
                                typ: TokenItemType::Ident,
                                val: "foo".to_string(),
                                pos: 0
                            }
                        ]
                    },
                    Token {
                        typ: ScanTokenType::Option,
                        items: vec![
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
                        ]
                    },
                    Token {
                        typ: ScanTokenType::Option,
                        items: vec![
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
                        ]
                    },
                    Token {
                        typ: ScanTokenType::List,
                        items: vec![
                            TokenItem {
                                typ: TokenItemType::Ident,
                                val: "list".to_string(), pos: 0
                            },
                            TokenItem {
                                typ: TokenItemType::String,
                                val: "10".to_string(),
                                pos: 0
                            },
                        ]
                    },
                    Token {
                        typ: ScanTokenType::Section,
                        items: vec![
                            TokenItem {
                                typ: TokenItemType::Ident,
                                val: "foo".to_string(),
                                pos: 0
                            },
                        ]
                    },
                    Token {
                        typ: ScanTokenType::Option,
                        items: vec![
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
                        ]
                    },
                    Token {
                        typ: ScanTokenType::Option,
                        items: vec![
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
                        ]
                    },
                    Token {
                        typ: ScanTokenType::List,
                        items: vec![
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
                        ]
                    },
                    Token {
                        typ: ScanTokenType::Section,
                        items: vec![
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
                        ]
                    },
                    Token {
                        typ: ScanTokenType::Option,
                        items: vec![
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
                        ]
                    },
                    Token {
                        typ: ScanTokenType::Option,
                        items: vec![
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
                        ]
                    },
                    Token {
                        typ: ScanTokenType::List,
                        items: vec![
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
                    },
                ]
            ),
            (
                "hyphenated",
                "\nconfig wifi-device wl0\n\toption type 'broadcom'\n\toption channel '6'\n\nconfig wifi-iface wifi0\n\toption device 'wl0'\n\toption mode 'ap'\n".to_string(),
                vec![
                    Token {
                        typ: ScanTokenType::Section,
                        items: vec![
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
                        ]
                    },
                    Token {
                        typ: ScanTokenType::Option,
                        items: vec![
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
                        ]
                    },
                    Token {
                        typ: ScanTokenType::Option,
                        items: vec![
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
                        ]
                    },
                    Token{
                        typ: ScanTokenType::Section,
                        items: vec![
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
                        ]
                    },
                    Token {
                        typ: ScanTokenType::Option,
                        items: vec![
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
                        ]
                    },
                    Token {
                        typ: ScanTokenType::Option,
                        items: vec![
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
                    },
                ]
            ),
            (
                "commented",
                "\n# heading\n\n# another heading\nconfig foo\n\toption opt1 1\n\t# option opt1 2\n\toption opt2 3 # baa\n\toption opt3 hello\n\n# a comment block spanning\n# multiple lines, surrounded\n# by empty lines\n\n# eof\n".to_string(),
                vec![
                    Token {
                        typ: ScanTokenType::Section,
                        items: vec![
                            TokenItem {
                                typ: TokenItemType::Ident,
                                val: "foo".to_string(),
                                pos: 0
                            },
                        ]
                    },
                    Token {
                        typ: ScanTokenType::Option,
                        items: vec![
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
                        ]
                    },
                    Token {
                        typ: ScanTokenType::Option,
                        items: vec![
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
                        ]
                    },
                    Token {
                        typ: ScanTokenType::Option,
                        items: vec![
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
                    },
                ]
            ),
            (
                "invalid",
                "\n<?xml version=\"1.0\">\n<error message=\"not a UCI file\" />\n".to_string(),
                vec![
                    Token{
                        typ: ScanTokenType::Error,
                        items: vec![
                            TokenItem {
                                typ: TokenItemType::Error,
                                val: "config: invalid, expected keyword (package, config, option, list) or eof".to_string(),
                                pos: 0
                            }
                        ]
                    },
                ],
            ),
            (
                "pkg invalid",
                "\n package\n".to_string(),
                vec![
                    Token {
                        typ: ScanTokenType::Error,
                        items: vec![
                            TokenItem {
                                typ: TokenItemType::Error,
                                val: "config: pkg invalid, incomplete package name".to_string(),
                                pos: 0
                            },
                        ]
                    }
                ],
            ),
            (
                "unterminated quoted string",
                "\nconfig foo \"bar\n".to_string(),
                vec![
                    Token {
                        typ: ScanTokenType::Section,
                        items: vec![
                            TokenItem {
                                typ: TokenItemType::Ident,
                                val: "foo".to_string(),
                                pos: 0
                            },
                        ]
                    },
                    Token {
                        typ: ScanTokenType::Error,
                        items: vec![
                            TokenItem {
                                typ: TokenItemType::Error,
                                val: "config: unterminated quoted string, unterminated quoted string".to_string(),
                                pos: 0
                            }
                        ]
                    }
                ]
            ),
            (
                "unterminated unquoted string",
                "\nconfig foo\n\toption opt opt\\\n".to_string(),
                vec![
                    Token {
                        typ: ScanTokenType::Section,
                        items: vec![
                            TokenItem {
                                typ: TokenItemType::Ident,
                                val: "foo".to_string(),
                                pos: 0
                            },
                        ]
                    },
                    Token {
                        typ: ScanTokenType::Error,
                        items: vec![
                            TokenItem {
                                typ: TokenItemType::Error,
                                val: "config: unterminated unquoted string, unterminated unquoted string".to_string(),
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
