use quote::{format_ident, quote};
use syn::{BinOp, Expr, Ident};

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
                BinOp::And(_) | BinOp::Or(_) => {
                    let op = x.op;
                    quote! {
                        (#lhs #op #rhs)
                    }
                },
                _ => panic!("unsupported binop.")
            }
        }
        Expr::Call(x) => quote! {#x(#arg)},
        Expr::Paren(x) => {
            let inside = make_binary_clause(&x.expr, arg);
            quote! {(#inside)}
        },
        Expr::Path(x) => quote! {#x(#arg)},
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