# A toy automaton engine for Rust

Now you can use a macro like this:

```rust
rustomaton! {
    #[input(name_of_the_context_structure)]
    /// we use u64 type index to distinguish different states
    /// an automaton can only have one initial state
    #[init(initial_state)]
    /// an automaton can have multiple final states 
    #[ends(final_state1, final_state2, ...)]
    
    /// an automaton edge has the form: 
    start -> end: transfer_function;
    
    /// a transfer function function can be empty:
    start -> end: _;
    
    /// or anything can be called and returns a bool value:
    start -> end: fun_symbol;
    start -> end: fun_returning_a_closure(args);
    
    /// or a bool expression
    start -> end: (fun_sym1 && fun_sym2) || fun_returning_a_closure(args)

    /// note that there cannot be two rules with the same edge, use a OR instead
    /// be aware that there is no extra semicolon after the last rule
}
```

this macro will generate a function named `run` in current namespace, its declaration is

```rust
impl <T: Exhausted> Automaton<T> {
    pub fn run(&self, mut context: &mut T) -> AutomatonResult;
}
```

call this function with your context, then it will return a result, either Accepted or Rejected

