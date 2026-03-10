use std::{error::Error, fmt::Debug, io, marker::PhantomData};
use hierrorchy::{error_leaf, error_node};

error_node! {
    type MyErrorNode<ErrorChild1,> = "custom message"
}

#[error_leaf(message = format!("error child 1"))]
struct ErrorChild1 {}

#[error_leaf(message = format!("test"))]
struct GenericError<T: Debug> {
    _phantom_data: PhantomData<T>,
}

#[error_leaf(message = format!("This message has not debug derivation"), derive_debug = false)]
struct GenericErrorWithoutDebug<T> {
    _phantom_data: PhantomData<T>,
}

impl<T> Debug for GenericErrorWithoutDebug<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "GenericErrorWithoutDebug")
    }
}

#[error_leaf(message = println!(""))]
struct MyError {}

error_node! {
    type PathErrorNode<io::Error, ErrorChild1, GenericError<i32>> = "path error"
}

