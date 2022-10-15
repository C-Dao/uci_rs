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
    TokenError = 0,
    TokenEOF,
    TokenPackage,
    TokenSection,
    TokenOption,
    TokenList,
}

impl fmt::Display for ScanTokenType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TokenError => {
                write!(f, "error")
            }
            Self::TokenEOF => {
                write!(f, "eof")
            }
            Self::TokenList => {
                write!(f, "list")
            }
            Self::TokenOption => {
                write!(f, "option")
            }
            Self::TokenPackage => {
                write!(f, "package")
            }
            Self::TokenSection => {
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

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn token_item_type_to_string() {
        assert_eq!(TokenItemType::TokenEOF.to_string(), "EOF");
        assert_eq!(TokenItemType::TokenConfig.to_string(), "Config");
        assert_eq!(TokenItemType::TokenError.to_string(), "Error");
        assert_eq!(TokenItemType::TokenIdent.to_string(), "Ident");
        assert_eq!(TokenItemType::TokenList.to_string(), "List");
        assert_eq!(TokenItemType::TokenOption.to_string(), "Option");
        assert_eq!(TokenItemType::TokenPackage.to_string(), "Package");
        assert_eq!(TokenItemType::TokenString.to_string(), "String");
    }
    #[test]
    fn scan_token_to_string() {
        assert_eq!(ScanTokenType::TokenEOF.to_string(), "eof");
        assert_eq!(ScanTokenType::TokenError.to_string(), "error");
        assert_eq!(ScanTokenType::TokenList.to_string(), "list");
        assert_eq!(ScanTokenType::TokenOption.to_string(), "option");
        assert_eq!(ScanTokenType::TokenPackage.to_string(), "package");
        assert_eq!(ScanTokenType::TokenSection.to_string(), "config");
    }
    #[test]
    fn token_item_to_string() {
        let token_item = TokenItem {
            typ: TokenItemType::TokenOption,
            val: format!("network wlan"),
            pos: 0,
        };
        assert_eq!(token_item.to_string(), "(Option \"network wlan\" 0)");
    }

    #[test]
    fn token_to_string(){
        let token = Token {
            typ: ScanTokenType::TokenPackage,
            items: vec![
                TokenItem {
                    typ: TokenItemType::TokenIdent,
                    val: format!("network"),
                    pos: 0,
                }
            ]
           
        };
        assert_eq!(token.to_string(), "package [TokenItem { typ: TokenIdent, val: \"network\", pos: 0 }]");
    
    }
}
