pub mod parser;
pub mod tree;

use self::tree::UciTree;

#[allow(dead_code)]
const DEFAULT_TREE_PATH: &str = "/etc/config";

#[allow(dead_code)]
pub fn init_default_tree() -> UciTree {
    UciTree::new(DEFAULT_TREE_PATH)
}

#[cfg(test)]
mod test {
    use crate::storage::uci::tree::Uci;

    use super::*;
    #[test]
    fn test_init_default_tree() {
        let tree = init_default_tree();
        assert_eq!(tree.get_dir().to_str().unwrap(), DEFAULT_TREE_PATH);
    }
}
