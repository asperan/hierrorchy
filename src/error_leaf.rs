use syn::{LitStr, Macro, parse::Parse};

pub enum MessageFormat {
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

