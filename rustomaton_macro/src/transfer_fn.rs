use quote::{format_ident, quote};
use syn::{BinOp, Expr, Ident, Lit, Token, parse2};

/// generate implementation of transfer functions.
///
/// All generated functions have the form `|arg| -> bool {body}`
///
/// for example 
/// - `_` will be transformed into 
///   ```
///   |_| true
///   ```
/// - `fun_sym` will be transformed into 
///
///   ```
///   |__arg| {
///     fun_sym(__arg)
///   }
///   ```
///
/// - `ret_closure(args)` will be transformed into
///
///   ```
///   |__arg| {
///     ret_closure(args)(__arg)
///   }
///   ```
///
///   , which is useful when you are fimiliar with currying
/// - `fun_sym || ret_closure(args)` will be transformed into 
///
///   ```
///   |__arg| {
///     fun_sym(__arg) || ret_closure(args)(__arg)
///   }
///   ```
fn make_binary_clause(expr: &Box<Expr>, arg: &Ident) -> proc_macro2::TokenStream {
    match expr.as_ref() {
        Expr::Binary(x) => {
            let lhs = make_binary_clause(&x.left, arg);
            let rhs = make_binary_clause(&x.right, arg);

            match x.op {
                BinOp::BitOr(_) => {
                    quote! {
                        (#lhs || #rhs)
                    }
                },
                _ => panic!("unsupported binop.")
            }
        },
        Expr::Lit(s) => {
            match &s.lit {
                Lit::Str(t) => quote! {#arg.eat(#t)},
                _ => panic!("automaton language type not match.")
            }
        },
        Expr::Verbatim(x) => {
            if let Ok(_) = parse2::<Token![_]>(x.clone()) {
                quote! {|| true}
            } else {
                panic!("unsupported representation.")
            }
        },
        _ => panic!("unsupported representation.")
    }
}


pub fn make_transfer_fn(func: &Option<Expr>) -> Option<proc_macro2::TokenStream> {
    match func {
        None => None,
        Some(x) => {
            let arg = format_ident!("__arg");
            let body = make_binary_clause(&Box::new(x.clone()), &arg);
            Some(quote! {
                ::std::boxed::Box::new(|#arg| {
                    #body
                })
            })
        }
    }
}