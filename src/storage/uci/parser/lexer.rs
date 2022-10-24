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

#[derive(Debug)]
enum LexerState {
    KeyWord,
    Comment,
    Package,
    PackageName,
    Config,
    ConfigType,
    OptionalName,
    Option,
    List,
    OptionName,
    Value,
    Quoted,
    Unquoted,
}

impl Lexer {
    pub fn new(name: &str, input: String) -> Self {
        Lexer {
            name: name.to_string(),
            input,
            state: Some(LexerState::KeyWord),
            items: Some(VecDeque::new()),
            start: 0,
            pos: 0,
            width: 0,
        }
    }

    fn next_rune(&mut self) -> Option<char> {
        if self.pos >= self.input.len() {
            self.width = 0;
            return None;
        };
        if let Some(rune) = self.input.get(self.pos..).unwrap().chars().next() {
            self.width = rune.len_utf8();
            self.pos += self.width;
            Some(rune)
        } else {
            None
        }
    }

    #[allow(dead_code)]
    fn accept_rune(&mut self, val: &str) {
        loop {
            match self.next_rune() {
                Some(c) if val.contains(c) => {}
                _ => break,
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
        let rune = self.next_rune();
        self.backup();
        rune
    }

    fn rest(&self) -> Option<String> {
        self.input.get(self.pos..).map(|s| s.to_string())
    }

    fn emit(&mut self, typ: TokenItemType) {
        if self.pos > self.start {
            self.items.as_mut().unwrap().push_back(TokenItem {
                typ,
                val: self.input.get(self.start..self.pos).unwrap().to_string(),
                pos: self.pos,
            });
            self.start = self.pos;
        }
    }

    fn emit_error(&mut self, error: &str) -> Option<LexerState> {
        self.items.as_mut().unwrap().push_back(TokenItem {
            typ: TokenItemType::Error,
            val: format!("config: {}, {}", self.name, error),
            pos: self.pos,
        });
        None
    }

    fn accept_comment(&mut self) {
        if self.next_rune().unwrap() == '#' {
           while let Some(r) = self.next_rune() {
                    if r == '\n' {
                        break;
                    }
                } 
            }
        self.backup();
    }

    fn consume_nowrap_whitespace(&mut self) {
        while let Some(rune) = self.peek() {
                if rune == ' ' || rune == '\t' {
                    self.next_rune();
                } else {
                    break;
                }
         
        }

        self.ignore();
    }

    fn consume_whitespace(&mut self) {
        while let Some(rune) = self.peek() {
                if rune.is_whitespace() {
                    self.next_rune();
                } else {
                    break;
                }
        }

        self.ignore();
    }

    fn accept_ident(&mut self) {
        loop {
            match self.next_rune() {
                Some(r)
                    if !(r == '-'
                        || r == '_'
                        || ('a'..='z').contains(&r)
                        || ('A'..='Z').contains(&r)
                        || ('0'..='9').contains(&r)) =>
                {
                    self.backup();
                    break;
                }
                _ => {}
            }
        }
    }

    fn accept_once(&mut self, val: &str) -> bool {
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
                return self.stop();
            } else if let Some(it) = self.items.as_mut().unwrap().pop_front() {
                return it;
            } else {
                self.state = self.action();
            }
        };
        self.stop()
    }

    pub fn stop(&mut self) -> TokenItem {
        let mut it = self.eof();
        if self.items.is_none() {
            it
        } else {
            if let Some(last_it) = self.items.as_mut().unwrap().pop_front() {
                it = last_it;
            }
            self.items = None;
            it
        }
    }

    fn eof(&self) -> TokenItem {
        return TokenItem {
            typ: TokenItemType::EOF,
            val: self.input.get(self.start..self.pos).unwrap().to_string(),
            pos: self.pos,
        };
    }

    fn emit_string(&mut self, t: TokenItemType) {
        if self.pos > self.start + 1 {
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
            Some(LexerState::Comment) => self.lex_comment(),
            Some(LexerState::Config) => self.lex_config(),
            Some(LexerState::ConfigType) => self.lex_config_type(),
            Some(LexerState::KeyWord) => self.lex_key_word(),
            Some(LexerState::List) => self.lex_list(),
            Some(LexerState::Option) => self.lex_option(),
            Some(LexerState::OptionName) => self.lex_option_name(),
            Some(LexerState::OptionalName) => self.lex_optional_name(),
            Some(LexerState::Package) => self.lex_package(),
            Some(LexerState::PackageName) => self.lex_package_name(),
            Some(LexerState::Quoted) => self.lex_quoted(),
            Some(LexerState::Unquoted) => self.lex_unquoted(),
            Some(LexerState::Value) => self.lex_value(),
            None => None,
        }
    }
    fn lex_key_word(&mut self) -> Option<LexerState> {
        self.consume_whitespace();
        match self.rest() {
            Some(curr) if curr.starts_with('#') => Some(LexerState::Comment),
            Some(curr) if curr.starts_with(KeyWord::KW_PACKAGE) => Some(LexerState::Package),
            Some(curr) if curr.starts_with(KeyWord::KW_CONFIG) => Some(LexerState::Config),
            Some(curr) if curr.starts_with(KeyWord::KW_OPTION) => Some(LexerState::Option),
            Some(curr) if curr.starts_with(KeyWord::KW_LIST) => Some(LexerState::List),
            _ => {
                if self.next_rune().is_none() {
                    self.emit(TokenItemType::EOF);
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
        Some(LexerState::KeyWord)
    }

    fn lex_package(&mut self) -> Option<LexerState> {
        self.pos += KeyWord::KW_PACKAGE.len();
        self.emit(TokenItemType::Package);
        Some(LexerState::PackageName)
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
                    return Some(LexerState::Quoted);
                }
                _ => {}
            }
        }
    }
    fn lex_config(&mut self) -> Option<LexerState> {
        self.pos += KeyWord::KW_CONFIG.len();
        self.emit(TokenItemType::Config);
        self.consume_nowrap_whitespace();
        Some(LexerState::ConfigType)
    }

    fn lex_config_type(&mut self) -> Option<LexerState> {
        self.accept_ident();
        self.emit(TokenItemType::Ident);
        self.consume_nowrap_whitespace();
        Some(LexerState::OptionalName)
    }

    fn lex_optional_name(&mut self) -> Option<LexerState> {
        match self.next_rune() {
            Some(r) if r == '\n' => self.ignore(),
            Some(r) if r == '"' || r == '\'' => {
                self.backup();
                return Some(LexerState::Quoted);
            }
            _ => {
                self.accept_ident();
                self.emit(TokenItemType::String)
            }
        };
        Some(LexerState::KeyWord)
    }

    fn lex_option(&mut self) -> Option<LexerState> {
        self.pos += KeyWord::KW_OPTION.len();
        self.emit(TokenItemType::Option);
        self.consume_nowrap_whitespace();
        Some(LexerState::OptionName)
    }

    fn lex_list(&mut self) -> Option<LexerState> {
        self.pos += KeyWord::KW_LIST.len();
        self.emit(TokenItemType::List);
        self.consume_nowrap_whitespace();
        Some(LexerState::OptionName)
    }

    fn lex_option_name(&mut self) -> Option<LexerState> {
        self.accept_ident();
        self.emit(TokenItemType::Ident);
        self.consume_nowrap_whitespace();
        Some(LexerState::Value)
    }

    fn lex_value(&mut self) -> Option<LexerState> {
        if let Some(r) = self.peek() {
            if r == '"' || r == '\'' {
                return Some(LexerState::Quoted);
            }
        };
        Some(LexerState::Unquoted)
    }

    fn lex_quoted(&mut self) -> Option<LexerState> {
        if let Some(q) = self.next_rune() {
            if q != '"' && q != '\'' {
                return self.emit_error("expected quotation");
            };
            loop {
                match self.next_rune() {
                    Some(r) if r == '\\' => {
                        if self.next_rune().is_some() {
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
            self.emit_string(TokenItemType::String);
            self.consume_nowrap_whitespace();
            return Some(LexerState::KeyWord);
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
        self.emit(TokenItemType::String);
        self.consume_nowrap_whitespace();
        self.accept_once("\n");
        self.ignore();
        Some(LexerState::KeyWord)
    }
}

#[cfg(test)]
mod test {
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
                if item.typ == TokenItemType::EOF {
                    break;
                };
                assert_eq!(item.typ, expected[idx].typ);
                assert_eq!(item.val, expected[idx].val);
                idx += 1;
            }

            assert_eq!(expected.len(), idx);
        }
    }
}
