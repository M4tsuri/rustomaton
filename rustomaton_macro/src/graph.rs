use std::collections::{HashMap, HashSet};

use quote::{ToTokens, quote};
use proc_macro2::TokenStream;
use syn::{Expr, Lit, Token, parse2};
use crate::transfer_fn::make_transfer_fn;


use crate::parse::Body;

type State = u64;

pub enum AutomatonType {
    NFA,
    DFA
}

#[derive(Clone)]
pub struct Automaton {
    states: HashSet<State>,
    initial_state: State,
    final_states: HashSet<State>,
    edges: HashMap<State, Vec<State>>,
    transfers: HashMap<(State, State), Option<Expr>>,
    transfers_internel: HashMap<(State, State), Option<Vec<String>>>
}

impl Automaton {
    fn extract_strs(expr: &Box<Expr>) -> Vec<String> {
        let mut res = Vec::new();
        match expr.as_ref() {
            Expr::Binary(x) => {
                res.extend(Self::extract_strs(&x.left));
                res.extend(Self::extract_strs(&x.right));
            },
            Expr::Lit(s) => {
                match &s.lit {
                    Lit::Str(t) => res.push(t.value()),
                    _ => panic!("automaton language type not match.")
                }
            },
            Expr::Verbatim(x) => {
                if let Ok(_) = parse2::<Token![_]>(x.clone()) {
                    res.push(String::from(""));
                } else {
                    panic!("unsupported representation.");
                }
            },
            _ => panic!("unsupported representation.")
        }
        res
    }

    fn parse_edge(expr: &Option<Expr>) -> Option<Vec<String>> {
        if expr.is_none() {
            return None;
        }

        Some(Self::extract_strs(&Box::new(expr.as_ref().unwrap().clone())))
    }

    pub fn new(body: &Body) -> Self {
        let initial_state = body.init_stat.base10_parse::<u64>().unwrap();

        let mut edges: HashMap<State, Vec<State>> = HashMap::new();
        let mut states: HashSet<State> = HashSet::new();
        let mut transfers: HashMap<(State, State), Option<Expr>> = HashMap::new();
        let mut transfers_internel: HashMap<(State, State), Option<Vec<String>>> = HashMap::new();

        let final_states: HashSet<State> = body.fini_stats.iter().map(|x| {
            x.base10_parse().unwrap()
        }).collect();

        for rule in &body.rules {
            let begin = &rule.begin_stat;
            let end = &rule.end_stat;
            let begin_base10 = begin.base10_parse::<State>().unwrap();
            let end_base10 = end.base10_parse::<State>().unwrap();

            states.insert(end_base10);
            states.insert(begin_base10);

            if !edges.contains_key(&begin_base10) {
                edges.insert(begin_base10, Vec::new());
                
            }
            
            edges.get_mut(&begin_base10).unwrap().push(end_base10);

            if transfers.insert((begin_base10, end_base10), rule.transfer.clone()).is_some() {
                panic!("duplicated relations.");
            }
        }

        for (rel, trans) in &transfers {
            transfers_internel.insert(rel.clone(), Self::parse_edge(&trans));
        }

        Automaton {
            states,
            initial_state,
            final_states,
            edges,
            transfers,
            transfers_internel
        }
    }

    fn to_dfa(&self) -> Self {
        panic!("not implemented");
    }

    /// user must use a Epsilon transfer function to explicted show that its a NFA
    /// for example, a NFA like this is not well-defined for our engine:
    /// 
    /// ```
    /// 0 -> 1: eat("b");
    /// 0 -> 2: eat("b");
    /// ```
    ///
    /// it should be transformed into 
    /// 
    /// ```
    /// 0 -> 3: _;
    /// 0 -> 4: _;
    /// 3 -> 1: eat("b");
    /// 4 -> 2: eat("b");
    /// ```
    ///
    fn validate(&self) -> AutomatonType {
        
        AutomatonType::DFA
    }
}

impl ToTokens for Automaton {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        if let AutomatonType::NFA = self.validate() {
            return self.to_dfa().to_tokens(tokens);
        }

        let init_state = self.initial_state;
        
        let mut fill_states: Vec<proc_macro2::TokenStream> = Vec::new();
        let mut fill_relations: Vec<proc_macro2::TokenStream> = Vec::new();
        let mut fill_transfers: Vec<proc_macro2::TokenStream> = Vec::new();

        fill_states.extend(self.states.iter().map(|state| {
            quote! {
                automaton.states.insert(#state);
            }
        }));

        for (begin, ends) in &self.edges {
            fill_relations.push(quote! {
                automaton.relations.insert(#begin, ::std::vec::Vec::new());
            });

            fill_relations.extend(ends.iter().map(|end| {
                quote! {
                    automaton.relations.get_mut(&#begin).unwrap().push(#end);
                }
            }));
        }

        fill_transfers.extend(self.transfers.iter().map(|((begin, end), transfer)| {
            let transfer_fn = make_transfer_fn(transfer);
            quote! {
                automaton.transfer.insert((#begin, #end), #transfer_fn);
            }
        }));

        let fill_fini_states: Vec<proc_macro2::TokenStream> = self.final_states.iter().map(|x| {
            quote! {
                automaton.fini_states.insert(#x);
            }
        }).collect();

        tokens.extend(quote! {
            pub trait NewAutomaton {
                fn new() -> Automaton;
            }

            impl NewAutomaton for Automaton {
                fn new() -> Self {
                    let mut automaton: Automaton = Automaton {
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