use std::fmt::{self};

#[derive(PartialEq, Debug, Clone)]
pub struct TokenItem {
    pub typ: TokenItemType,
    pub val: String,
    pub pos: usize,
}


#[derive(PartialEq, Debug, Clone, Copy)]
pub enum TokenItemType {
    TokenError = 0,
    TokenBOF,
    TokenEOF,
    TokenPackage,
    TokenConfig,
    TokenOption,
    TokenList,
    TokenIdent,
    TokenString,
}

impl fmt::Display for TokenItemType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TokenError => {
                write!(f, "Error")
            }
            Self::TokenBOF => {
                write!(f, "BOF")
            }
            Self::TokenEOF => {
                write!(f, "EOF")
            }
            Self::TokenPackage => {
                write!(f, "Package")
            }
            Self::TokenConfig => {
                write!(f, "Config")
            }
            Self::TokenOption => {
                write!(f, "Option")
            }
            Self::TokenList => {
                write!(f, "List")
            }
            Self::TokenIdent => {
                write!(f, "Ident")
            }
            Self::TokenString => {
                write!(f, "String")
            }
            _ => {
                write!(f, "Unknown")
            }
        }
    }
}

pub struct Keyword {}

impl Keyword {
    pub const KwPackage: &'static str = "package";
    pub const KwConfig: &'static str = "config";
    pub const KwOption: &'static str = "option";
    pub const KwList: &'static str = "list";
}

impl fmt::Display for TokenItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.pos < 0 {
            if self.typ != TokenItemType::TokenError && self.val.len() > 25 {
                return write!(f, "({} {:?})", self.typ, self.val.get(0..25).unwrap());
            };
            return write!(f, "({} {:?})", self.typ, self.val);
        };

        if self.typ != TokenItemType::TokenError && self.val.len() > 25 {
            return write!(
                f,
                "({} {:?} {})",
                self.typ,
                self.val.get(0..25).unwrap(),
                self.pos
            );
        }
        return write!(f, "({} {:?} {})", self.typ, self.val, self.pos);
    }
}

#[derive(PartialEq)]
pub enum ScanTokenType {
    TokError = 0,
    TokEOF,
    TokPackage,
    TokSection,
    TokOption,
    TokList,
}

impl fmt::Display for ScanTokenType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TokError => {
                write!(f, "error")
            }
            Self::TokEOF => {
                write!(f, "eof")
            }
            Self::TokList => {
                write!(f, "list")
            }
            Self::TokOption => {
                write!(f, "option")
            }
            Self::TokPackage => {
                write!(f, "package")
            }
            Self::TokSection => {
                write!(f, "config")
            }
            _ => {
                write!(f, "unknown")
            }
        }
    }
}

pub struct Token {
    pub typ: ScanTokenType,
    pub items: Vec<TokenItem>,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {:?}", self.typ, self.items)
    }
}
