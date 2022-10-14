pub mod parser;
pub mod tree;

use self::tree::UciTree;

#[allow(dead_code)]
const DEFAULT_TREE_PATH: &str = "/etc/config";


pub fn init_default_tree() -> UciTree {
    UciTree::new(DEFAULT_TREE_PATH)
}
