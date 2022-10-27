use super::*;
#[test]
fn test_token_item_type_to_string() {
        assert_eq!(TokenItemType::Eof.to_string(), "EOF");
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