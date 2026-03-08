//! Hierrorchy is a proc-macro library to simplify the creation of error hierarchies (hence the
//! name), in a tree-like structure.
//!
//! This crate is based on two concepts:
//! - Leaves: base errors which can occur during the execution of a program
//!   ([`hierrorchy::error_leaf`](macro@error_leaf)).
//! - Nodes: errors which source can be a leaf or another node
//!   ([`hierrorchy::error_node`](macro@error_node)).
//!
//! As nodes are "just" containers for other errors, they are `enum`s with a variant for each type
//! of error they can contain; while leaves, which must be as open as possible, are `struct`s.
//!
//! # Example of an error leaf
//! Error leaves are declared by adding an attribute to a struct definition (see
//! [`hierrorchy::error_leaf`](macro@error_leaf) documentation for details on its configuration):
//! ```
//! use hierrorchy::error_leaf;
//!
//! #[error_leaf("My error")]
//! struct MyError {}
//! ```
//!
//! The attribute adds the implementation of [`std::fmt::Display`] and [`std::error::Error`], thus
//! writing the snippet of code above is equivalent to wrinting the following code:
//! ```
//! #[derive(Debug)]
//! struct MyError{}
//!
//! impl std::fmt::Display for MyError {
//!     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//!         write!(f, "{}", "My error")
//!     }
//! }
//!
//! impl std::error::Error for MyError {}
//! ```
//!
//! As you can see from the snippet above, [`hierrorchy::error_leaf`](macro@error_leaf) adds the attribute for
//! deriving the [`std::fmt::Debug`] implementation, as it is required by [`std::error::Error`].
//!
//! # Example of an error node
//! Error nodes are declared by the function-like macro [`hierrorchy::error_node`](macro@error_node):
//! ```
//! use hierrorchy::{error_leaf,error_node};
//! use std::error::Error;
//!
//! #[error_leaf("My error")]
//! struct MyError {}
//!
//! error_node! { type MyErrorNode<MyError> = "my error node" }
//! ```
//!
//! This snippet is equivalent to:
//! ```
//! use hierrorchy::error_leaf;
//! use std::error::Error;
//!
//! #[error_leaf("My error")]
//! struct MyError {}
//!
//! #[derive(Debug)]
//! enum MyErrorNode {
//!     Variant0(MyError),
//! }
//!
//! impl std::fmt::Display for MyErrorNode {
//!     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//!         write!(f, "my error node: {}", &self.source().expect("MyErrorNode always has a source"))
//!     }
//! }
//!
//! impl std::error::Error for MyErrorNode {
//!    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
//!        match self {
//!            Self::Variant0(e) => Some(e),
//!         }
//!     }
//! }
//!
//! impl From<MyError> for MyErrorNode {
//!     fn from(value: MyError) -> Self {
//!         Self::Variant0(value)
//!     }
//! }
//! ```
//!
//! As it can be seen in the snippet above, [`hierrorchy::error_node`](macro@error_node) also implements
//! [`std::convert::From`]s
//! for each variant of the node, allowing to leverage the `?` operator in functions which return a
//! [`std::result::Result`].
//!
//! # Complete example
//! ```
//! use hierrorchy::{error_leaf,error_node};
//! use rand::prelude::*;
//! use std::error::Error;
//!
//! fn main() {
//!     if let Err(e) = entrypoint() {
//!         eprintln!("{}", e);
//!     }
//! }
//!
//! fn entrypoint() -> Result<(), MyErrorNode> {
//!     check_boolean()?;
//!     let value = rand::random::<i32>();
//!     check_value(value)?;
//!     Ok(())
//! }
//!
//! fn check_boolean() -> Result<(), MyFirstErrorLeaf> {
//!     if rand::random() {
//!         Err(MyFirstErrorLeaf {})
//!     } else {
//!        Ok(())
//!     }
//! }
//!
//! fn check_value(value: i32) -> Result<(), MySecondErrorLeaf> {
//!     if value % 2 == 0 {
//!        Err(MySecondErrorLeaf { value })
//!     } else {
//!         Ok(())
//!     }
//! }
//!
//! #[error_leaf("first check failed")]
//! struct MyFirstErrorLeaf {}
//!
//! #[error_leaf(format!("second check failed: value is {}", self.value))]
//! struct MySecondErrorLeaf {
//!     value: i32,
//! }
//!
//! error_node! { type MyErrorNode<MyFirstErrorLeaf, MySecondErrorLeaf> = "error node" }
//! ```
mod error_leaf;
mod error_node;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemStruct};

use crate::{error_leaf::MessageFormat, error_node::ErrorNode};

/// Attribute to mark a Struct definition as an error leaf.
/// Implementation of `Display` and `Error` is created by the macro.
///
/// # Examples
/// The message can be written in 2 forms: plain string or format macro.
///
/// The format macro form allows to use the struct fields and methods to enhance the error message.
/// In this form, use `self` to access them.
///
/// The plain string form cannot use struct fields, thus is better suited for errors which do not
/// need a message which depends in the internal fields.
/// ```
/// use hierrorchy::error_leaf;
///
/// // Format macro form
/// #[error_leaf(format!("{} is wrong", self.myfield))]
/// struct MyError {
///    myfield: String,
/// }
///
/// // Plain string form
/// #[error_leaf("simple error")]
/// struct SimpleError {}
/// ```
#[proc_macro_attribute]
pub fn error_leaf(attr: TokenStream, item: TokenStream) -> TokenStream {
    let msg_fmt = parse_macro_input!(attr as MessageFormat);
    let struct_def = parse_macro_input!(item as ItemStruct);
    let struct_name = &struct_def.ident;
    let (impl_generics, ty_generics, where_clause) = &struct_def.generics.split_for_impl();

    let display_impl = {
        let format_arg = match msg_fmt {
            MessageFormat::Format(f) => quote! { #f },
            MessageFormat::Lit(l) => quote! { #l },
        };
        quote! {
            impl #impl_generics std::fmt::Display for #struct_name #ty_generics #where_clause {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    write!(f, "{}", #format_arg)
                }
            }
        }
    };
    let error_impl = quote! {
        impl #impl_generics std::error::Error for #struct_name #ty_generics #where_clause {}
    };
    let derive_debug = quote! {
        #[derive(Debug)]
    };

    let result_stream = quote! {
        #derive_debug
        #struct_def
        #display_impl
        #error_impl
    };

    result_stream.into()
}

/// Function-like proc macro to construct error nodes.
/// The body requires the following format:
/// `type (name)<variants> [= (string)]`
/// where `name` is the name to give to the error node (an enum), `variants` is a comma-separated list of other
/// errors (both leaves and nodes), and `string` is an optional string to use rather than the node
/// name when printing the error node.
///
/// # Examples:
/// ```
/// use hierrorchy::{error_leaf, error_node};
/// use std::error::Error;
///
/// #[error_leaf(format!("error child 1"))]
/// pub struct ErrorChild1 {}
///
/// error_node! { type MyErrorNode<ErrorChild1> = "custom prefix" }
/// ```
///
/// ## Variants with paths
/// > Since version 0.2.0
///
/// error_node also accept variants in the form of paths, e.g. `std::io::Error`.
///
/// This allows to write:
/// ```ignore
/// error_node! { type MyErrorNode<std::io::Error> = "custom message" }
/// ```
/// rather than:
/// ```ignore
/// use std::io::Error as IoError;
/// error_node! { type MyErrorNode<IoError> = "custom message" }
/// ```
///
#[proc_macro]
pub fn error_node(tokens: TokenStream) -> TokenStream {
    let input = parse_macro_input!(tokens as ErrorNode);

    let enum_declaration = input.error_node_enum();
    let impl_display = input.error_node_display_impl();
    let impl_error = input.error_node_error_impl();
    let impl_froms = input.error_node_from_impls();

    let mut token_buffer = TokenStream::new();
    token_buffer.extend(enum_declaration);
    token_buffer.extend(impl_display);
    token_buffer.extend(impl_error);
    token_buffer.extend(impl_froms);
    token_buffer
}
