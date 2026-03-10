use std::{error::Error, fmt::Display, str::FromStr};

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{Error as SynError, Ident, ItemStruct, LitBool, Macro, Token, parse::Parse, spanned::Spanned};

pub struct ErrorLeaf {
    config: ErrorLeafConfig,
    struct_def: ItemStruct,
}

impl ErrorLeaf {
    pub fn new(config: ErrorLeafConfig, struct_def: ItemStruct) -> ErrorLeaf {
        ErrorLeaf { config, struct_def }
    }

    pub fn to_token_stream(&self) -> TokenStream {
        let struct_def = &self.struct_def;
        let struct_name = &self.struct_def.ident;
        let (impl_generics, ty_generics, where_clause) = &self.struct_def.generics.split_for_impl();

        let display_impl = {
            let format_arg = &self.config.message;
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
        let derive_debug = if self.config.derive_debug {
            quote! {
                #[derive(Debug)]
            }
        } else {
            TokenStream2::new()
        };

        let result_stream = quote! {
            #derive_debug
            #struct_def
            #display_impl
            #error_impl
        };

        result_stream.into()
    }
}

pub struct ErrorLeafConfig {
    message: Macro,
    derive_debug: bool,
}

impl Parse for ErrorLeafConfig {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut macro_config_builder = ErrorLeafConfigBuilder::new();
        while !input.is_empty() {
            let keyword: Ident = input.parse()?;
            let _: Token![=] = input.parse()?;
            match keyword
                .to_string()
                .parse::<ErrorLeafConfigKeyword>()
                .map_err(|it| SynError::new(keyword.span(), it))?
            {
                ErrorLeafConfigKeyword::DeriveDebug => {
                    let value: LitBool = input.parse()?;
                    macro_config_builder.set_derive_debug(value.value());
                }
                ErrorLeafConfigKeyword::Message => {
                    let value: Macro = input.parse()?;
                    if value.path.segments.last().expect("A Macro call must have a last path segment").ident != "format" {
                        return Err(SynError::new(value.span(), format!("The only accepted macro for keyword {} is 'format'", ErrorLeafConfigKeyword::Message)));
                    }
                    macro_config_builder.set_message(value);
                }
            }
            if !input.is_empty() {
                let _: Token![,] = input.parse()?;
            }
        }
        macro_config_builder
            .build()
            .map_err(|it| SynError::new(input.span(), it))
    }
}

struct ErrorLeafConfigBuilder {
    message: Option<Macro>,
    derive_debug: Option<bool>,
}

impl ErrorLeafConfigBuilder {
    pub fn new() -> Self {
        ErrorLeafConfigBuilder {
            message: None,
            derive_debug: None,
        }
    }

    pub fn set_message(&mut self, format: Macro) {
        self.message = Some(format);
    }

    pub fn set_derive_debug(&mut self, derive_debug: bool) {
        self.derive_debug = Some(derive_debug);
    }

    pub fn build(&self) -> Result<ErrorLeafConfig, MissingRequiredConfigurationError> {
        if self.message.is_none() {
            return Err(MissingRequiredConfigurationError {
                keyword: String::from("path"),
            });
        }
        Ok(ErrorLeafConfig {
            message: self
                .message
                .as_ref()
                .expect("path existence is already checked")
                .clone(),
            derive_debug: self.derive_debug.unwrap_or(true),
        })
    }
}

#[derive(Debug, Clone, Copy)]
enum ErrorLeafConfigKeyword {
    Message,
    DeriveDebug,
}

impl Display for ErrorLeafConfigKeyword {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Message => "message",
                Self::DeriveDebug => "derive_debug",
            }
        )
    }
}

impl FromStr for ErrorLeafConfigKeyword {
    type Err = UnknownConfigKeywordError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "message" => Ok(Self::Message),
            "derive_debug" => Ok(Self::DeriveDebug),
            _ => Err(UnknownConfigKeywordError {
                keyword: s.to_string(),
            }),
        }
    }
}

#[derive(Debug, Clone)]
pub struct UnknownConfigKeywordError {
    keyword: String,
}

impl Display for UnknownConfigKeywordError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "unknown keyword '{}' in macro configuration",
            self.keyword
        )
    }
}

impl Error for UnknownConfigKeywordError {}

#[derive(Debug, Clone)]
pub struct MissingRequiredConfigurationError {
    keyword: String,
}

impl Display for MissingRequiredConfigurationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "the required keyword '{}' is missing in macro configuration",
            self.keyword
        )
    }
}

impl Error for MissingRequiredConfigurationError {}
