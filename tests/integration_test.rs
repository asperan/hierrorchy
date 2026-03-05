use std::{error::Error, fmt::Debug, io, marker::PhantomData};
use hierrorchy::{error_leaf, error_node};

error_node! {
    type MyErrorNode<ErrorChild1,> = "custom message"
}

#[error_leaf(format!("error child 1"))]
struct ErrorChild1 {}

#[error_leaf(format!("test"))]
struct GenericError<T: Debug> {
    _phantom_data: PhantomData<T>,
}

error_node! {
    type PathErrorNode<io::Error, ErrorChild1, GenericError<i32>> = "path error"
}

