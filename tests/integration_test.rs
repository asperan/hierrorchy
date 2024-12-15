use error_tree::error_leaf;

#[error_leaf(format!("error child 1"))]
struct ErrorChild1 {}

