use std::{error::Error, io};
use hierrorchy::{error_leaf, error_node};

error_node! {
    type MyErrorNode<ErrorChild1,> = "custom message"
}

#[error_leaf(format!("error child 1"))]
struct ErrorChild1 {}

error_node! {
    type PathErrorNode<io::Error, ErrorChild1> = "path error"
}

