use std::fmt;

#[derive(PartialEq, Debug, Clone)]
pub struct TokenItem {
    pub typ: TokenItemType,
    pub val: String,
    pub pos: usize,
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum TokenItemType {
   Error = 0,
   Eof,
   Package,
   Config,
   Option,
   List,
   Ident,
   String,
}

impl fmt::Display for TokenItemType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Error => {
                write!(f, "Error")
            }
            Self::Eof => {
                write!(f, "EOF")
            }
            Self::Package => {
                write!(f, "Package")
            }
            Self::Config => {
                write!(f, "Config")
            }
            Self::Option => {
                write!(f, "Option")
            }
            Self::List => {
                write!(f, "List")
            }
            Self::Ident => {
                write!(f, "Ident")
            }
            Self::String => {
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
        if self.typ != TokenItemType::Error && self.val.len() > 25 {
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

#[derive(PartialEq, Debug)]
pub enum ScanTokenType {
    Error = 0,
    Package,
    Section,
    Option,
    List,
}

impl fmt::Display for ScanTokenType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Error => {
                write!(f, "error")
            }
            Self::List => {
                write!(f, "list")
            }
            Self::Option => {
                write!(f, "option")
            }
            Self::Package => {
                write!(f, "package")
            }
            Self::Section => {
                write!(f, "config")
            }
        }
    }
}

#[derive(Debug)]
pub struct Token {
    pub typ: ScanTokenType,
    pub items: Vec<TokenItem>,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {:?}", self.typ, self.items)
    }
}

#[cfg(test)]
mod test;