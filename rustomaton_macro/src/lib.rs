extern crate proc_macro;

mod parse;
mod graph;

use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;
use parse::Body;
use graph::Automaton;

fn make_runtime(body: &Body) -> proc_macro2::TokenStream {
    let impls = Automaton::new(body);
    quote! {
        #impls
    }
}

/// out proc_macro which makes a runtime from our automaton definition
/// the grammar is:
/// 
/// ```
/// rustomaton! {
///     #[input(name_of_the_context_structure)]
///     /// we use u64 type index to distinguish different states
///     /// an automaton can only have one initial state
///     #[init(initial_state)]
///     /// an automaton can have multiple final states 
///     #[ends(final_state1, final_state2, ...)]
///     
///     /// an automaton edge has the form: 
///     start -> end: transfer_function;
///     
///     /// a transfer function function can be empty:
///     start -> end: _;
///     
///     /// or anything can be called and returns a bool value:
///     start -> end: fun_symbol;
///     start -> end: fun_returning_a_closure(args);
///     
///     /// or a bool expression
///     start -> end: (fun_sym1 && fun_sym2) || fun_returning_a_closure(args);
///
///     /// note that there cannot be two rules with the same edge, use a OR instead
/// ```
/// 
/// this macro will generate a function named `run` in current namespace, its declaration is
/// 
/// ```
/// impl <T: Exhausted> Automaton<T> {
///     pub fn run(&self, mut context: &mut T) -> AutomatonResult;
/// }
/// ```
/// 
/// call this function with your context, then it will return a result, either Accepted or Rejected
///
#[proc_macro]
pub fn rustomaton(item: TokenStream) -> TokenStream {
    let input: Body = parse_macro_input!(item as Body);
    let context = make_runtime(&input);

    context.into()
}


