//! Hierrorchy is a proc-macro library to simplify the creation of error hierarchies (hence the
//! name), in a tree-like structure.
//!
//! This crate is based on two concepts:
//! - Leaves: base errors which can occur during the execution of a program
//! ([`hierrorchy::error_leaf`](macro@error_leaf)).
//! - Nodes: errors which source can be a leaf or another node
//! ([`hierrorchy::error_node`](macro@error_node)).
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
//! use std::process::exit;
//!
//! fn main() {
//!     if let Err(e) = entrypoint() {
//!         eprintln!("{}", e);
//!         exit(1);
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
use proc_macro::TokenStream;
use proc_macro2::{Group, TokenStream as TokenStream2};
use quote::{format_ident, quote, ToTokens};
use syn::{parse::Parse, parse_macro_input, Ident, ItemStruct, LitStr, Macro, Token};

enum MessageFormat {
    Lit(LitStr),
    Format(Macro),
}

impl Parse for MessageFormat {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(LitStr) {
            input.parse().map(MessageFormat::Lit)
        } else {
            Ok(MessageFormat::Format(input.parse::<Macro>()?))
        }
    }
}

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

    let display_impl = match msg_fmt {
        MessageFormat::Format(f) => {
            quote! {
                impl std::fmt::Display for #struct_name {
                    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(f, "{}", #f)
                    }
                }
            }
        }
        MessageFormat::Lit(l) => {
            quote! {
                impl std::fmt::Display for #struct_name {
                    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(f, "{}", #l)
                    }
                }
            }
        }
    };
    let error_impl = quote! {
        impl std::error::Error for #struct_name {}
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

struct ErrorNode {
    is_pub: bool,
    node_name: Ident,
    variants: Vec<Ident>,
    message_prefix: Option<LitStr>,
}

impl Parse for ErrorNode {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let is_pub = input.lookahead1().peek(Token![pub]);
        if is_pub {
            let _: Token![pub] = input.parse()?;
        }

        let _: Token![type] = input.parse()?;
        let node_name: Ident = input.parse()?;

        let mut variants: Vec<Ident> = vec![];
        let _open_angle_bracket: Token![<] = input.parse()?;
        let mut keep_parsing_variants = true;
        while keep_parsing_variants {
            if input.lookahead1().peek(Token![>]) {
                keep_parsing_variants = false;
                let _close_angle_bracket: Token![>] = input.parse()?;
            } else {
                variants.push(input.parse()?);
                if input.lookahead1().peek(Token![,]) {
                    let _: Token![,] = input.parse()?;
                }
            }
        }

        if input.is_empty() {
            Ok(ErrorNode {
                is_pub,
                node_name,
                variants,
                message_prefix: None,
            })
        } else {
            let _: Token![=] = input.parse()?;
            let message_prefix: LitStr = input.parse()?;
            Ok(ErrorNode {
                is_pub,
                node_name,
                variants,
                message_prefix: Some(message_prefix),
            })
        }
    }
}

fn format_variant_name(number: usize) -> Ident {
    format_ident!("Variant{}", number)
}

fn error_node_enum(node_name: &Ident, is_pub: bool, variants: &[Ident]) -> TokenStream {
    let mut token_buffer = TokenStream2::new();
    token_buffer.extend(quote! { #[derive(Debug)] });
    if is_pub {
        token_buffer.extend(quote! { pub });
    }
    token_buffer.extend(quote! { enum });
    token_buffer.extend(node_name.clone().into_token_stream());
    token_buffer.extend(
        Group::new(
            proc_macro2::Delimiter::Brace,
            TokenStream2::from_iter(variants.iter().enumerate().map(|it| {
                let variant_ident = format_variant_name(it.0);
                let variant_inner_type = it.1;
                quote! {
                    #variant_ident(#variant_inner_type),
                }
            })),
        )
        .to_token_stream(),
    );
    token_buffer.into()
}

fn error_node_display_impl(node_name: &Ident, message_prefix: Option<&LitStr>) -> TokenStream {
    let mut token_buffer = TokenStream2::new();
    token_buffer.extend(quote! { impl std::fmt::Display for #node_name });
    let message_format = format!(
        "{}: {{}}",
        match message_prefix {
            Some(l) => l.value(),
            None => node_name.to_string(),
        }
    );
    let expect_message = format!("{} always has a source", node_name);
    token_buffer.extend(
        Group::new(
            proc_macro2::Delimiter::Brace,
            quote! {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    write!(f, #message_format, &self.source().expect(#expect_message))
                }
            },
        )
        .to_token_stream(),
    );
    token_buffer.into()
}

fn error_node_error_impl(node_name: &Ident, variants: &[Ident]) -> TokenStream {
    let mut token_buffer = TokenStream2::new();
    token_buffer.extend(quote! { impl std::error::Error for #node_name });
    let variant_matches = TokenStream2::from_iter(variants.iter().enumerate().map(|it| {
        let variant_name = format_variant_name(it.0);
        quote! {
            Self::#variant_name(err) => Some(err),
        }
    }));
    token_buffer.extend(
        Group::new(
            proc_macro2::Delimiter::Brace,
            quote! {
                fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
                    match self {
                        #variant_matches
                    }
                }
            },
        )
        .to_token_stream(),
    );
    token_buffer.into()
}

fn error_node_from_impls(node_name: &Ident, variants: &[Ident]) -> TokenStream {
    let mut token_buffer = TokenStream2::new();
    token_buffer.extend(variants.iter().enumerate().map(|it| {
        let variant_inner_type = it.1;
        let variant_name = format_variant_name(it.0);
        quote! {
            impl From<#variant_inner_type> for #node_name {
                fn from(value: #variant_inner_type) -> Self {
                    Self::#variant_name(value)
                }
            }
        }
    }));
    token_buffer.into()
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
#[proc_macro]
pub fn error_node(tokens: TokenStream) -> TokenStream {
    let input = parse_macro_input!(tokens as ErrorNode);
    let is_pub = input.is_pub;
    let node_name = input.node_name;
    let variants = input.variants;
    let message_prefix = input.message_prefix;

    let enum_declaration = error_node_enum(&node_name, is_pub, &variants);
    let impl_display = error_node_display_impl(&node_name, message_prefix.as_ref());
    let impl_error = error_node_error_impl(&node_name, &variants);
    let impl_froms = error_node_from_impls(&node_name, &variants);

    let mut token_buffer = TokenStream::new();
    token_buffer.extend(enum_declaration);
    token_buffer.extend(impl_display);
    token_buffer.extend(impl_error);
    token_buffer.extend(impl_froms);
    token_buffer
}
