use proc_macro::TokenStream;
use quote::quote;
use syn::{parse::Parse, parse_macro_input, ItemStruct, LitStr, Macro};

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
/// use error_tree::error_leaf;
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
