use std::{borrow::BorrowMut, cell::RefCell, collections::VecDeque, sync::Arc, vec, rc::Rc};

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

    fn eof(&self) -> Token {
        return Token {
            typ: ScanTokenType::TokEOF,
            items: vec![],
        };
    }

    fn stop(&mut self) -> Token {
        let mut tok = self.eof();
        if self.tokens.is_none() {
            return tok;
        } else {
            self.lexer.stop();
            if self.tokens.as_ref().unwrap().len() > 0 {
                tok = self.tokens.as_mut().unwrap().pop_front().unwrap();
            }

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

    fn accept(&mut self, it: TokenItemType) -> bool {
        let tok = self.next_item();
        if tok.typ == it {
            self.curr.push(tok);
            return true;
        }
        self.backup(&tok);
        return false;
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
            typ: ScanTokenType::TokError,
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
            } else {
                match self.tokens.as_mut().unwrap().pop_front() {
                    Some(tok) => {
                        if tok.typ == ScanTokenType::TokError
                            || tok.typ == ScanTokenType::TokPackage
                        {
                            self.stop();
                        };
                        return Some(tok);
                    }
                    None => {
                        self.state = self.action();
                        if self.state.is_none() {
                            let tok = self.stop();
                            if tok.typ == ScanTokenType::TokEOF {
                                return None;
                            } else {
                                return Some(tok);
                            }
                        }
                    }
                }
            }
        }
        None
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
            it => self.emit_error("expected package or config token"),
        }
    }

    fn scan_package(&mut self) -> Option<ScannerState> {
        match self.next_item() {
            it if it.typ == TokenItemType::TokenString => {
                self.curr.push(it);
                self.emit(ScanTokenType::TokPackage);
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
                    self.accept(TokenItemType::TokenString);
                };
                self.emit(ScanTokenType::TokSection);
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
        if self.accept(TokenItemType::TokenIdent) {
            Some(ScannerState::ScanOptionValue)
        } else {
            self.emit_error("expected option name")
        }
    }

    fn scan_list_name(&mut self) -> Option<ScannerState> {
        if self.accept(TokenItemType::TokenIdent) {
            Some(ScannerState::ScanListName)
        } else {
            self.emit_error("expected option name")
        }
    }

    fn scan_option_value(&mut self) -> Option<ScannerState> {
        match self.next_item() {
            it if it.typ == TokenItemType::TokenString => {
                self.curr.push(it);
                self.emit(ScanTokenType::TokOption);
                Some(ScannerState::ScanOption)
            }
            it if it.typ == TokenItemType::TokenError => self.emit_error(&it.val),
            it => self.emit_error("expected option value"),
        }
    }

    fn scan_list_value(&mut self) -> Option<ScannerState> {
        match self.next_item() {
            it if it.typ == TokenItemType::TokenString => {
                self.curr.push(it);
                self.emit(ScanTokenType::TokList);
                Some(ScannerState::ScanOption)
            }
            it if it.typ == TokenItemType::TokenError => self.emit_error(&it.val),
            it => self.emit_error("expected option value"),
        }
    }
}

fn parse(name: String, input: String) -> Result<UciConfig> {
    match Scanner::new(&name, input).try_fold(
        (UciConfig::new(&name), None),
        |(mut cfg, mut sec):(UciConfig, Option<UciSection>), tok: Token| -> Result<(UciConfig, Option<UciSection>)> {
            match tok.typ {
                ScanTokenType::TokError => {
                    return Err(Error::new(format!("parse error: {}", tok.items[0].val)));
                },
                ScanTokenType::TokPackage => {
                    return  Err(Error::new(
                    "UCI imports/exports are not yet supported".to_string(),
                ));
            },
                ScanTokenType::TokSection => {
                    let sec_typ = tok.items[0].val.to_string();
                    if tok.items.len() == 2 {
                        sec = Some(cfg.merge(UciSection::new(sec_typ, tok.items[1].val.to_string())).clone());
                    } else {
                        sec = Some(cfg.add(UciSection::new(sec_typ, "".to_string())).clone());
                    }
                },
                ScanTokenType::TokOption => {
                    let name = tok.items[0].val.to_string();
                    let val = tok.items[1].val.to_string();

                    if let Some(opt) = sec.as_mut().unwrap().get_mut(&name) {
                        opt.set_values(vec![val]);
                    } else {
                        sec.as_mut().map(|s| s.add(UciOption::new(name, UciOptionType::TypeOption, vec![val])));
                    };
                },
                ScanTokenType::TokList => {
                    let name = tok.items[0].val.to_string();
                    let val = tok.items[1].val.to_string();

                    if let Some(opt) = sec.as_mut().unwrap().get_mut(&name) {
                        opt.merge_values(vec![val]);
                    } else {
                        sec.as_mut().map(|s| s.add(UciOption::new(name, UciOptionType::TypeList, vec![val])));
                    };
                },
                ScanTokenType::TokEOF => {},
            };
            Ok((cfg, sec))
        },
    ) {
        Ok((cfg,_)) => Ok(cfg),
        Err(err) => Err(err),
    }
}
