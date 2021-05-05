type State = u64;

pub trait Exhausted {
    fn exhausted(&self) -> bool;
}
pub struct Automaton<T> {
    pub init_state: State,
    pub fini_states: ::std::collections::HashSet<State>,
    pub states: ::std::collections::HashSet<State>,
    pub relations: ::std::collections::HashMap<State, Vec<State>>,
    pub transfer: ::std::collections::HashMap<(State, State), Box<dyn Fn(&mut T) -> bool>>,
}

pub trait NewAutomaton<T> {
    fn new() -> Automaton<T>;
}

#[derive(Debug)]
pub enum AutomatonResult {
    Accepted,
    Rejected
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

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
            Some(_) => self.__run(context, next_state.unwrap()),
            None => AutomatonResult::Rejected
        }
    }
}
