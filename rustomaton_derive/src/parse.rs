use syn::{Attribute, Expr, LitInt, Token, TypePath};
use syn::punctuated::Punctuated;
use syn::token::{Colon, RArrow};
use syn::parse::{ParseStream, Parse, Result};

pub struct Rule {
    pub begin_stat: LitInt,
    _rarrow: RArrow,
    pub end_stat: LitInt,
    _colon: Colon,
    pub transfer: Option<Expr>
}

pub struct Body {
    pub init_stat: LitInt,
    pub fini_stats: Punctuated<LitInt, Token!(,)>,
    pub rules: Punctuated<Rule, Token!(;)>,
}

impl Parse for Rule {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Rule {
            begin_stat: input.parse()?,
            _rarrow: input.parse()?,
            end_stat: input.parse()?,
            _colon: input.parse()?,
            transfer: {
                if input.peek(Token![_]) {
                    let _: Token![_] = input.parse()?;
                    None
                } else {
                    Some(input.parse()?)
                }
            }
        })
    }
}



impl Parse for Body {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut init_state: Option<LitInt> = None;
        let mut fini_states: Option<Punctuated<LitInt, Token!(,)>> = None;
        let mut input_type: Option<TypePath> = None;
        
        let attrs = input.call(Attribute::parse_outer)?;
        for attr in attrs {
            if attr.path.is_ident("input") {
                input_type = Some(attr.parse_args()?);
            } else if attr.path.is_ident("init") {
                init_state = Some(attr.parse_args()?);
            } else if attr.path.is_ident("ends") {
                fini_states = Some(attr.parse_args_with(Punctuated::parse_separated_nonempty)?);
            } else {
                return Err(input.error("message"));
            }
        }
        

        Ok(Body {
            rules: input.call(Punctuated::parse_separated_nonempty)?,
            init_stat: init_state.ok_or(input.error("you must specify a initial state"))?,
            fini_stats: fini_states.ok_or(input.error("you must specify final states."))?
        })
    }
}