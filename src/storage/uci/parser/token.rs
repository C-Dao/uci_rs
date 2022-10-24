use std::fmt::{self};

#[derive(PartialEq, Debug, Clone)]
pub struct TokenItem {
    pub typ: TokenItemType,
    pub val: String,
    pub pos: usize,
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum TokenItemType {
   Error = 0,
   EOF,
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
            Self::EOF => {
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
mod test {
    use super::*;
    #[test]
    fn test_token_item_type_to_string() {
        assert_eq!(TokenItemType::EOF.to_string(), "EOF");
        assert_eq!(TokenItemType::Config.to_string(), "Config");
        assert_eq!(TokenItemType::Error.to_string(), "Error");
        assert_eq!(TokenItemType::Ident.to_string(), "Ident");
        assert_eq!(TokenItemType::List.to_string(), "List");
        assert_eq!(TokenItemType::Option.to_string(), "Option");
        assert_eq!(TokenItemType::Package.to_string(), "Package");
        assert_eq!(TokenItemType::String.to_string(), "String");
    }
    #[test]
    fn test_scan_token_to_string() {
        assert_eq!(ScanTokenType::Error.to_string(), "error");
        assert_eq!(ScanTokenType::List.to_string(), "list");
        assert_eq!(ScanTokenType::Option.to_string(), "option");
        assert_eq!(ScanTokenType::Package.to_string(), "package");
        assert_eq!(ScanTokenType::Section.to_string(), "config");
    }
    #[test]
    fn test_token_item_to_string() {
        let token_item = TokenItem {
            typ: TokenItemType::Option,
            val: "network wlan".to_string(),
            pos: 0,
        };
        assert_eq!(token_item.to_string(), "(Option \"network wlan\" 0)");
    }

    #[test]
    fn test_token_to_string() {
        let token = Token {
            typ: ScanTokenType::Package,
            items: vec![TokenItem {
                typ: TokenItemType::Ident,
                val: "network".to_string(),
                pos: 0,
            }],
        };
        assert_eq!(
            token.to_string(),
            "package [TokenItem { typ: Ident, val: \"network\", pos: 0 }]"
        );
    }
}
