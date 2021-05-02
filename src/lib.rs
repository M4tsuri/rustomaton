extern crate proc_macro;

mod parse;

use std::collections::HashSet;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input};
use parse::Body;

type State = u64;

fn make_runtime_decl(body: &Body) -> proc_macro2::TokenStream {
    let input_type = &body.input_type;

    quote! {
        type State = u64;

        struct Automaton {
            init_state: State,
            fini_states: ::std::collections::HashSet<State>,
            states: ::std::collections::HashSet<State>,
            relations: ::std::collections::HashMap<State, Vec<State>>,
            transfer: ::std::collections::HashMap<(State, State), Box<dyn Fn(&mut #input_type) -> bool>>,
        }
        
        #[derive(Debug)]
        pub enum AutomatonResult {
            Accepted,
            Rejected
        }
        
        impl Automaton {
            pub fn run(&self, mut context: &mut #input_type) -> AutomatonResult {
                self.__run(context, self.init_state)
            }
        
            fn __run(&self, mut context: &mut #input_type, cur_state: State) -> AutomatonResult {
                let may_next_states = self.relations.get(&cur_state).unwrap();
                let mut next_state: Option<State> = None;
        
                for may_next_state in may_next_states {
                    if let Some(func) = self.transfer.get(&(cur_state, *may_next_state)) {
                        if func(context) {
                            if self.fini_states.contains(may_next_state) {
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

fn make_runtime_impl(body: &Body) -> proc_macro2::TokenStream {
    let input_type = &body.input_type;
    let init_state = &body.init_stat;
    let mut states_set: HashSet<State> = HashSet::new();
    let mut begin_states_set: HashSet<State> = HashSet::new();

    let mut fill_states: Vec<proc_macro2::TokenStream> = Vec::new();
    let mut fill_relations: Vec<proc_macro2::TokenStream> = Vec::new();
    let mut fill_transfers: Vec<proc_macro2::TokenStream> = Vec::new();

    for rule in &body.rules {
        let begin = &rule.begin_stat;
        let end = &rule.end_stat;
        let func = &rule.transfer;

        if states_set.insert(end.base10_parse::<u64>().unwrap()) {
            fill_states.push(quote! {
                automaton.states.insert(#end);
            });
        }

        if states_set.insert(begin.base10_parse::<u64>().unwrap()) {
            fill_states.push(quote! {
                automaton.states.insert(#begin);
            });
        }

        if begin_states_set.insert(begin.base10_parse::<u64>().unwrap()) {
            fill_relations.push(quote! {
                automaton.relations.insert(#begin, ::std::vec::Vec::new());
            });
        }
        
        fill_relations.push(quote! {
            automaton.relations.get_mut(&#begin).unwrap().push(#end);
        });

        fill_transfers.push(quote! {
            automaton.transfer.insert((#begin, #end), ::std::boxed::Box::new(#func));
        })
        
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
            let mut automaton = Automaton {
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
    let decls = make_runtime_decl(body);
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


