use std::collections::{HashMap, HashSet};

use quote::{ToTokens, quote};
use proc_macro2::TokenStream;
use syn::TypePath;
use crate::transfer_fn::make_transfer_fn;


use crate::parse::Body;

type State = u64;

pub enum AutomatonType {
    NFA,
    DFA
}

#[derive(Clone)]
pub struct AutomatonContext {
    input_type: TypePath,
    states: HashSet<State>,
    initial_state: State,
    final_states: HashSet<State>,
    edges: HashMap<(State, State), Option<proc_macro2::TokenStream>>
}

impl AutomatonContext {
    pub fn new(body: &Body) -> Self {
        let input_type = body.input_type.clone();
        let initial_state = body.init_stat.base10_parse::<u64>().unwrap();

        // make sure relations and states are only innitialized once
        let mut states: HashSet<State> = HashSet::new();
        let mut edges: HashMap<(State, State), Option<proc_macro2::TokenStream>> = HashMap::new();

        let final_states: HashSet<State> = body.fini_stats.iter().map(|x| {
            x.base10_parse().unwrap()
        }).collect();

        for rule in &body.rules {
            let begin = &rule.begin_stat;
            let end = &rule.end_stat;
            let begin_base10 = begin.base10_parse::<u64>().unwrap();
            let end_base10 = end.base10_parse::<u64>().unwrap();

            states.insert(end_base10);
            states.insert(begin_base10);

            if edges.insert((begin_base10, end_base10), 
                make_transfer_fn(&rule.transfer)).is_some() {
                panic!("duplicated relations.");
            }
        }

        AutomatonContext {
            input_type,
            states,
            initial_state,
            final_states,
            edges
        }
    }

    fn to_dfa(&self) -> Self {
        self.clone()
    }

    fn validate(&self) -> AutomatonType {
        AutomatonType::DFA
    }
}

impl ToTokens for AutomatonContext {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        if let AutomatonType::NFA = self.validate() {
            return self.to_dfa().to_tokens(tokens);
        }

        let input_type = &self.input_type;
        let init_state = self.initial_state;
        
        let mut fill_states: Vec<proc_macro2::TokenStream> = Vec::new();
        let mut fill_relations: Vec<proc_macro2::TokenStream> = Vec::new();
        let mut fill_transfers: Vec<proc_macro2::TokenStream> = Vec::new();

        let mut relation_begins: HashSet<State> = HashSet::new();

        for state in &self.states {
            fill_states.push(quote! {
                automaton.states.insert(#state);
            });
        }

        for ((begin, end), transfer) in &self.edges {
            if relation_begins.insert(*begin) {
                fill_relations.push(quote! {
                    automaton.relations.insert(#begin, ::std::vec::Vec::new());
                });
            }

            fill_relations.push(quote! {
                automaton.relations.get_mut(&#begin).unwrap().push(#end);
            });

            fill_transfers.push(quote! {
                automaton.transfer.insert((#begin, #end), #transfer);
            });
        }

        let fill_fini_states: Vec<proc_macro2::TokenStream> = self.final_states.iter().map(|x| {
            quote! {
                automaton.fini_states.insert(#x);
            }
        }).collect();

        tokens.extend(quote! {
            impl Automaton<#input_type> {
                pub fn new() -> Self {
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
        
                    automaton
                }
            }
        });
    }
}