use std::collections::VecDeque;

use super::token::{KeyWord, TokenItem, TokenItemType};

pub struct Lexer {
    name: String,
    input: String,
    start: usize,
    pos: usize,
    width: usize,
    state: Option<LexerState>,
    items: Option<VecDeque<TokenItem>>,
}

trait LexerStateMachine {
    fn action(&mut self) -> Option<LexerState>;
    fn lex_key_word(&mut self) -> Option<LexerState>;
    fn lex_comment(&mut self) -> Option<LexerState>;
    fn lex_package(&mut self) -> Option<LexerState>;
    fn lex_package_name(&mut self) -> Option<LexerState>;
    fn lex_config(&mut self) -> Option<LexerState>;
    fn lex_config_type(&mut self) -> Option<LexerState>;
    fn lex_optional_name(&mut self) -> Option<LexerState>;
    fn lex_option(&mut self) -> Option<LexerState>;
    fn lex_list(&mut self) -> Option<LexerState>;
    fn lex_option_name(&mut self) -> Option<LexerState>;
    fn lex_value(&mut self) -> Option<LexerState>;
    fn lex_quoted(&mut self) -> Option<LexerState>;
    fn lex_unquoted(&mut self) -> Option<LexerState>;
}

enum LexerState {
    LexKeyWord,
    LexComment,
    LexPackage,
    LexPackageName,
    LexConfig,
    LexConfigType,
    LexOptionalName,
    LexOption,
    LexList,
    LexOptionName,
    LexValue,
    LexQuoted,
    LexUnquoted,
}

impl Lexer {
    pub fn new(name: &str, input: String) -> Self {
        return Lexer {
            name: name.to_string(),
            input: input,
            state: Some(LexerState::LexKeyWord),
            items: Some(VecDeque::new()),
            start: 0,
            pos: 0,
            width: 0,
        };
    }

    fn next_rune(&mut self) -> Option<char> {
        if self.pos >= self.input.len() {
            self.width = 0;
            return None;
        };
        if let Some((rune_index, rune)) = self.input.get(self.pos..).unwrap().char_indices().next()
        {
            self.width = rune_index - self.pos;
            self.pos = rune_index;
            Some(rune)
        } else {
            None
        }
    }

    fn accept_rune(&mut self, val: &str) {
        loop {
            match self.next_rune() {
                Some(c) if val.contains(c) => break,
                None => break,
                _ => {}
            };
        }
        self.backup()
    }

    fn backup(&mut self) {
        self.pos -= self.width;
    }

    fn ignore(&mut self) {
        self.start = self.pos;
    }

    fn peek(&mut self) -> Option<char> {
        if let Some(rune) = self.next_rune() {
            self.backup();
            Some(rune)
        } else {
            self.backup();
            None
        }
    }

    fn rest(&self) -> Option<String> {
        self.input.get(self.pos..).map(|s| s.to_string())
    }

    fn emit(&mut self, typ: TokenItemType) {
        if self.pos > self.start {
            self.items.as_mut().unwrap().push_back(TokenItem {
                typ: typ,
                val: self.input.get(self.start..self.pos).unwrap().to_string(),
                pos: self.pos,
            });
            self.start = self.pos;
        }
    }

    fn emit_error(&mut self, error: &str) -> Option<LexerState> {
        self.items.as_mut().unwrap().push_back(TokenItem {
            typ: TokenItemType::TokenError,
            val: format!("config: {}, {}", self.name, error),
            pos: self.pos,
        });
        return None;
    }

    fn accept_comment(&mut self) {
        if self.next_rune().unwrap() == '#' {
            loop {
                if let Some(r) = self.next_rune() {
                    if r == '\n' {
                        break;
                    }
                } else {
                    break;
                }
            }
        };
        self.backup();
    }

    fn consume_whitespace(&mut self) {
        while self.peek().unwrap().is_whitespace() {
            self.next_rune();
        }
        self.ignore();
    }

    fn accept_ident(&mut self) {
        loop {
            match self.next_rune() {
                Some(r)
                    if !(r != '-'
                        || r == '_'
                        || 'a' <= r && r <= 'z'
                        || 'A' <= r && r <= 'Z'
                        || '0' <= r && r <= '9') =>
                {
                    self.backup();
                    break;
                }
                _ => {}
            }
        }
    }

    fn accept(&mut self, val: &str) -> bool {
        match self.next_rune() {
            Some(c) if val.contains(c) => true,
            _ => {
                self.backup();
                false
            }
        }
    }

    pub fn next_item(&mut self) -> TokenItem {
        while self.state.is_some() {
            if self.items.is_none() {
                return self.eof();
            } else {
                match self.items.as_mut().unwrap().pop_front() {
                    Some(it) => {
                        return it;
                    }
                    None => {
                        self.state = self.action();
                        if self.state.is_none() {
                            return self.stop();
                        }
                    }
                }
            }
        }
        self.eof()
    }

    pub fn stop(&mut self) -> TokenItem {
        let mut it = self.eof();
        if self.items.is_none() {
            return it;
        } else {
            if self.items.as_ref().unwrap().len() > 0 {
                it = self.items.as_mut().unwrap().pop_front().unwrap();
            }

            self.items = None;
            return it;
        }
    }

    fn eof(&self) -> TokenItem {
        return TokenItem {
            typ: TokenItemType::TokenEOF,
            val: self.input.get(self.start..self.pos).unwrap().to_string(),
            pos: self.pos,
        };
    }

    fn emit_string(&mut self, t: TokenItemType) {
        if self.pos - 1 > self.start + 1 {
            self.items.as_mut().unwrap().push_back(TokenItem {
                typ: t,
                val: self
                    .input
                    .get(self.start + 1..self.pos - 1)
                    .unwrap()
                    .to_string(),
                pos: self.pos,
            });
            self.start = self.pos;
        };
    }
}

impl LexerStateMachine for Lexer {
    fn action(&mut self) -> Option<LexerState> {
        match self.state {
            Some(LexerState::LexComment) => self.lex_comment(),
            Some(LexerState::LexConfig) => self.lex_config(),
            Some(LexerState::LexConfigType) => self.lex_config_type(),
            Some(LexerState::LexKeyWord) => self.lex_key_word(),
            Some(LexerState::LexList) => self.lex_list(),
            Some(LexerState::LexOption) => self.lex_option(),
            Some(LexerState::LexOptionName) => self.lex_option_name(),
            Some(LexerState::LexOptionalName) => self.lex_optional_name(),
            Some(LexerState::LexPackage) => self.lex_package(),
            Some(LexerState::LexPackageName) => self.lex_package_name(),
            Some(LexerState::LexQuoted) => self.lex_quoted(),
            Some(LexerState::LexUnquoted) => self.lex_unquoted(),
            Some(LexerState::LexValue) => self.lex_value(),
            None => None,
        }
    }
    fn lex_key_word(&mut self) -> Option<LexerState> {
        self.accept_rune(" \t\n");
        self.ignore();
        match self.rest() {
            Some(curr) if curr.starts_with("#") => Some(LexerState::LexComment),
            Some(curr) if curr.starts_with(KeyWord::KW_PACKAGE) => Some(LexerState::LexPackage),
            Some(curr) if curr.starts_with(KeyWord::KW_CONFIG) => Some(LexerState::LexConfig),
            Some(curr) if curr.starts_with(KeyWord::KW_OPTION) => Some(LexerState::LexOption),
            Some(curr) if curr.starts_with(KeyWord::KW_LIST) => Some(LexerState::LexList),
            _ => {
                if self.next_rune().is_none() {
                    self.emit(TokenItemType::TokenEOF);
                } else {
                    self.emit_error("expected keyword (package, config, option, list) or eof");
                }
                None
            }
        }
    }

    fn lex_comment(&mut self) -> Option<LexerState> {
        self.accept_comment();
        self.ignore();
        Some(LexerState::LexKeyWord)
    }

    fn lex_package(&mut self) -> Option<LexerState> {
        self.pos += KeyWord::KW_PACKAGE.len();
        self.emit(TokenItemType::TokenPackage);
        Some(LexerState::LexPackageName)
    }

    fn lex_package_name(&mut self) -> Option<LexerState> {
        loop {
            match self.next_rune() {
                Some(r) if r == '\n' => return self.emit_error("incomplete package name"),
                Some(r) if r.is_whitespace() => {
                    self.ignore();
                }
                Some(r) if r == '\'' || r == '"' => {
                    self.backup();
                    return Some(LexerState::LexQuoted);
                }
                _ => {}
            }
        }
    }
    fn lex_config(&mut self) -> Option<LexerState> {
        self.pos += KeyWord::KW_CONFIG.len();
        self.emit(TokenItemType::TokenConfig);
        self.consume_whitespace();
        Some(LexerState::LexConfigType)
    }

    fn lex_config_type(&mut self) -> Option<LexerState> {
        self.accept_ident();
        self.emit(TokenItemType::TokenIdent);
        self.consume_whitespace();
        Some(LexerState::LexOptionalName)
    }

    fn lex_optional_name(&mut self) -> Option<LexerState> {
        match self.next_rune() {
            Some(r) if r == '\n' => self.ignore(),
            Some(r) if r == '"' || r == '\'' => {
                self.backup();
                return Some(LexerState::LexQuoted);
            }
            _ => {
                self.accept_ident();
                self.emit(TokenItemType::TokenString)
            }
        };
        Some(LexerState::LexKeyWord)
    }

    fn lex_option(&mut self) -> Option<LexerState> {
        self.pos += KeyWord::KW_OPTION.len();
        self.emit(TokenItemType::TokenOption);
        self.consume_whitespace();
        Some(LexerState::LexOptionName)
    }

    fn lex_list(&mut self) -> Option<LexerState> {
        self.pos += KeyWord::KW_LIST.len();
        self.emit(TokenItemType::TokenList);
        self.consume_whitespace();
        Some(LexerState::LexOptionName)
    }

    fn lex_option_name(&mut self) -> Option<LexerState> {
        self.accept_ident();
        self.emit(TokenItemType::TokenIdent);
        self.consume_whitespace();
        Some(LexerState::LexValue)
    }

    fn lex_value(&mut self) -> Option<LexerState> {
        if let Some(r) = self.peek() {
            if r == '"' || r == '\'' {
                return Some(LexerState::LexQuoted);
            }
        };
        Some(LexerState::LexUnquoted)
    }

    fn lex_quoted(&mut self) -> Option<LexerState> {
        if let Some(q) = self.next_rune() {
            if q != '"' && q != '\'' {
                return self.emit_error("expected quotation");
            };
            loop {
                match self.next_rune() {
                    Some(r) if r == '\\' => {
                        if let Some(_) = self.next_rune() {
                            return None;
                        } else {
                            return self.emit_error("unterminated quoted string");
                        };
                    }
                    Some(r) if r == '\n' => {
                        return self.emit_error("unterminated quoted string");
                    }
                    None => {
                        return self.emit_error("unterminated quoted string");
                    }
                    Some(r) if r == q => {
                        break;
                    }
                    Some(_) => {}
                };
            }
            self.emit_string(TokenItemType::TokenString);
            self.consume_whitespace();
            return Some(LexerState::LexKeyWord);
        };
        None
    }

    fn lex_unquoted(&mut self) -> Option<LexerState> {
        loop {
            match self.next_rune() {
                Some(r) if r == '\\' => {
                    if self.next_rune().is_none() {
                        return self.emit_error("unterminated unquoted string");
                    };
                }
                None => {
                    return self.emit_error("unterminated unquoted string");
                }
                Some(r) if r == ' ' || r == '\t' || r == '#' || r == '\n' => {
                    break;
                }
                Some(_) => {}
            };
        }
        self.backup();
        self.emit(TokenItemType::TokenString);
        self.consume_whitespace();
        self.accept("\n");
        self.ignore();
        Some(LexerState::LexKeyWord)
    }
}
