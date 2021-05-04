extern crate proc_macro;

mod parse;
mod graph;
mod transfer_fn;

use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;
use parse::Body;
use graph::AutomatonContext;


/// genarate the core function doing the execution
/// 
/// thanks to our jump table design, this is relatively fast
fn make_runtime_decl() -> proc_macro2::TokenStream {
    quote! {
        type State = u64;

        pub trait Exhausted {
            fn exhausted(&self) -> bool;
        }

        pub struct Automaton<T> {
            init_state: State,
            fini_states: ::std::collections::HashSet<State>,
            states: ::std::collections::HashSet<State>,
            relations: ::std::collections::HashMap<State, Vec<State>>,
            transfer: ::std::collections::HashMap<(State, State), Box<dyn Fn(&mut T) -> bool>>,
        }
        
        #[derive(Debug)]
        pub enum AutomatonResult {
            Accepted,
            Rejected
        }
    }
}

fn make_runtime_exec() -> proc_macro2::TokenStream {
    quote! {
        impl<T: Exhausted> Automaton<T> {
            pub fn run(&self, context: &mut T) -> AutomatonResult {
                self.__run(context, self.init_state)
            }
        
            fn __run(&self, context: &mut T, cur_state: State) -> AutomatonResult {
                let may_next_states = self.relations.get(&cur_state).unwrap();
                let mut next_state: Option<State> = None;
        
                for may_next_state in may_next_states {
                    let func = self.transfer.get(&(cur_state, *may_next_state)).unwrap();
                    if !func(context) {
                        continue;
                    }

                    if context.exhausted() {
                        if self.fini_states.contains(may_next_state) {
                            return AutomatonResult::Accepted;
                        }
                        return AutomatonResult::Rejected;
                    }
                    next_state = Some(*may_next_state);
                    break;
                }
        
                match next_state {
                    Some(x) => self.__run(context, next_state.unwrap()),
                    None => AutomatonResult::Rejected
                }
            }
        }
    }
}

fn make_runtime(body: &Body) -> proc_macro2::TokenStream {
    let decls = make_runtime_decl();
    let exec = make_runtime_exec();
    let impls = AutomatonContext::new(body);
    quote! {
        #decls
        #exec
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


