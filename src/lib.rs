extern crate proc_macro;

mod parse;

use std::collections::HashSet;

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{BinOp, Expr, Ident, parse_macro_input};
use parse::Body;

type State = u64;

fn make_runtime_decl() -> proc_macro2::TokenStream {
    quote! {
        type State = u64;

        pub trait Exhausted {
            fn exhausted(&self) -> bool;
        }

        struct Automaton<T> {
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
        
        impl <T: Exhausted> Automaton<T> {
            pub fn run(&self, mut context: &mut T) -> AutomatonResult {
                self.__run(context, self.init_state)
            }
        
            fn __run(&self, mut context: &mut T, cur_state: State) -> AutomatonResult {
                let may_next_states = self.relations.get(&cur_state).unwrap();
                let mut next_state: Option<State> = None;
        
                for may_next_state in may_next_states {
                    if let Some(func) = self.transfer.get(&(cur_state, *may_next_state)) {
                        if func(context) {
                            if self.fini_states.contains(may_next_state) && context.exhausted() {
                                return AutomatonResult::Accepted;
                            }
                            next_state = Some(*may_next_state);
                            break;
                        }
                    }
                }
        
                match next_state {
                    Some(x) => self.__run(context, next_state.unwrap()),
                    None => AutomatonResult::Rejected
                }
            }
        }
    }
}

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

fn make_transfer_fn(func: &Option<Expr>) -> proc_macro2::TokenStream {
    match func {
        None => quote! {|_| true},
        Some(x) => {
            let arg = format_ident!("arg");
            let body = make_binary_clause(&Box::new(x.clone()), &arg);
            quote! {
                |#arg| {
                    #body
                }
            }
        }
    }
}

fn make_runtime_impl(body: &Body) -> proc_macro2::TokenStream {
    let input_type = &body.input_type;
    let init_state = &body.init_stat;

    let mut states_set: HashSet<State> = HashSet::new();
    let mut begin_states_set: HashSet<State> = HashSet::new();
    let mut relations_set: HashSet<(State, State)> = HashSet::new();

    let mut fill_states: Vec<proc_macro2::TokenStream> = Vec::new();
    let mut fill_relations: Vec<proc_macro2::TokenStream> = Vec::new();
    let mut fill_transfers: Vec<proc_macro2::TokenStream> = Vec::new();

    for rule in &body.rules {
        let begin = &rule.begin_stat;
        let end = &rule.end_stat;
        let begin_base10 = begin.base10_parse::<u64>().unwrap();
        let end_base10 = end.base10_parse::<u64>().unwrap();

        if states_set.insert(end_base10) {
            fill_states.push(quote! {
                automaton.states.insert(#end);
            });
        }

        if states_set.insert(begin_base10) {
            fill_states.push(quote! {
                automaton.states.insert(#begin);
            });
        }

        if begin_states_set.insert(begin_base10) {
            fill_relations.push(quote! {
                automaton.relations.insert(#begin, ::std::vec::Vec::new());
            });
        }
        
        if relations_set.insert((begin_base10, end_base10)) {
            fill_relations.push(quote! {
                automaton.relations.get_mut(&#begin).unwrap().push(#end);
            });

            let transfer_fn = make_transfer_fn(&rule.transfer);

            fill_transfers.push(quote! {
                automaton.transfer.insert((#begin, #end), ::std::boxed::Box::new(#transfer_fn));
            });
        } else {
            panic!("duplicated relations.");
        }
    }

    let fill_fini_states: Vec<proc_macro2::TokenStream> = body.fini_stats.iter().map(|x| {
        quote! {
            automaton.fini_states.insert(#x);
        }
    }).collect();
    
    quote! {
        #[derive(Debug)]
        pub struct Tmp {
            init_state: State,
            fini_states: ::std::collections::HashSet<State>,
            states: ::std::collections::HashSet<State>,
            relations: ::std::collections::HashMap<State, ::std::vec::Vec<State>>,
        }

        pub fn run(src: &mut #input_type) -> AutomatonResult {
            let mut automaton: Automaton<#input_type> = Automaton {
                init_state: #init_state,
                fini_states: ::std::collections::HashSet::new(),
                states: ::std::collections::HashSet::new(),
                relations: ::std::collections::HashMap::new(),
                transfer: ::std::collections::HashMap::new(),
            };

            #(#fill_fini_states)*
            #(#fill_states)*
            #(#fill_relations)*
            #(#fill_transfers)*

            let tmp = Tmp {
                init_state: #init_state,
                fini_states: automaton.fini_states.clone(),
                states: automaton.states.clone(),
                relations: automaton.relations.clone(),
            };

            println!("{:?}", tmp);

            automaton.run(src)
        }
    }
}

fn make_runtime(body: &Body) -> proc_macro2::TokenStream {
    let decls = make_runtime_decl();
    let impls = make_runtime_impl(body);
    quote! {
        #decls
        #impls
    }
}

#[proc_macro]
pub fn rustomaton(item: TokenStream) -> TokenStream {
    let input: Body = parse_macro_input!(item as Body);
    let context = make_runtime(&input);

    context.into()
}


