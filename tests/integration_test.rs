use std::error::Error;
use error_tree::{error_leaf, error_node};

error_node! {
    type MyErrorNode<ErrorChild1,> = "custom message"
}

#[error_leaf(format!("error child 1"))]
struct ErrorChild1 {}

