use proc_macro::TokenStream;
use proc_macro2::{Group, TokenStream as TokenStream2};
use quote::{ToTokens, format_ident, quote};
use syn::{Ident, LitStr, Path, Token, parse::Parse};

pub struct ErrorNode {
    is_pub: bool,
    node_name: Ident,
    variants: Vec<Path>,
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

        let mut variants: Vec<Path> = vec![];
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

impl ErrorNode {

    pub fn error_node_enum(&self) -> TokenStream {
        let mut token_buffer = TokenStream2::new();
        token_buffer.extend(quote! { #[derive(Debug)] });
        if self.is_pub {
            token_buffer.extend(quote! { pub });
        }
        token_buffer.extend(quote! { enum });
        token_buffer.extend(self.node_name.clone().into_token_stream());
        token_buffer.extend(
            Group::new(
                proc_macro2::Delimiter::Brace,
                TokenStream2::from_iter(self.variants.iter().enumerate().map(|it| {
                    let variant_ident = Self::format_variant_name(it.0);
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

    pub fn error_node_display_impl(&self) -> TokenStream {
        let mut token_buffer = TokenStream2::new();
        let node_name = &self.node_name;
        token_buffer.extend(quote! { impl std::fmt::Display for #node_name });
        let message_format = format!(
            "{}: {{}}",
            match &self.message_prefix {
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

    pub fn error_node_error_impl(&self) -> TokenStream {
        let mut token_buffer = TokenStream2::new();
        let node_name = &self.node_name;
        token_buffer.extend(quote! { impl std::error::Error for #node_name });
        let variant_matches = TokenStream2::from_iter(self.variants.iter().enumerate().map(|it| {
            let variant_name = Self::format_variant_name(it.0);
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

    pub fn error_node_from_impls(&self) -> TokenStream {
        let mut token_buffer = TokenStream2::new();
        let node_name = &self.node_name;
        token_buffer.extend(self.variants.iter().enumerate().map(|it| {
            let variant_inner_type = it.1;
            let variant_name = Self::format_variant_name(it.0);
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

    fn format_variant_name(number: usize) -> Ident {
        format_ident!("Variant{}", number)
    }
}


