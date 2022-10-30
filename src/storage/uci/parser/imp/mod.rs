use std::{collections::VecDeque, vec};

use crate::{utils::{Error, Result}};

use super::{
    lexer::Lexer,
    token::{ScanTokenType, Token, TokenItem, TokenItemType},
};

use super::super::tree::{UciConfig, UciOption, UciOptionType, UciSection};
use super::super::imp::Uci;

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
            it if it.typ == TokenItemType::Eof => None,
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
                cfg.set_name(&tok.items[0].val);
            }
            ScanTokenType::Section => {
                if sec.is_some() {
                    if let Some(s) = sec.as_ref() {
                        if s.sec_type != String::new() && s.name != String::new() {
                            cfg.merge(s.clone());
                        } else {
                            cfg.add(s.clone());
                        }
                    };
                };
                if tok.items.len() == 2 {
                    let sec_typ = &tok.items[0].val;
                    let name = &tok.items[1].val;
                    sec = Some(UciSection::new(sec_typ, name));
                } else {
                    let sec_typ = &tok.items[0].val;
                    sec = Some(UciSection::new(sec_typ, ""));
                }
            }
            ScanTokenType::Option => {
                let name = &tok.items[0].val;
                let val = tok.items[1].val.clone();

                if let Some(opt) = sec.as_mut().unwrap().get_mut(&name) {
                    opt.set_values(vec![val]);
                } else if let Some(s) = sec.as_mut() {
                    s.add(UciOption::new(&name, UciOptionType::TypeOption, vec![val]))
                };
            }
            ScanTokenType::List => {
                let name = &tok.items[0].val;
                let val = tok.items[1].val.clone();

                if let Some(opt) = sec.as_mut().unwrap().get_mut(&name) {
                    opt.merge_values(vec![val]);
                } else if let Some(s) = sec.as_mut() {
                    s.add(UciOption::new(&name, UciOptionType::TypeList, vec![val]))
                };
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

pub fn uci_parse_to_uci(name: &str, input: String) -> Result<Uci> {
    let cfg = uci_parse(name, input)?;
    let mut uci = Uci::new(name);
    uci.insert_config(cfg);
    Ok(uci)
}

#[cfg(test)]
mod test;
