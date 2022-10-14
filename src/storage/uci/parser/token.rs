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
        }
    }
}

pub struct KeyWord {}

impl KeyWord {
    pub const KW_PACKAGE: &'static str = "package";
    pub const KW_CONFIG: &'static str = "config";
    pub const KW_OPTION: &'static str = "option";
    pub const KW_LIST: &'static str = "list";
}

impl fmt::Display for TokenItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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
