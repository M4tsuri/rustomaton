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

this macro will generate an `Automaton` struct which implements a new method for the input type.

just new a instance and call the `run` method.

for example:

```rust
mod initial {
    use rustomaton::rustomaton;

    /// the context used in this automaton
    #[derive(Clone)]
    pub struct Context {
        pub src: String,
        pub cur: usize
    }

    /// an automaton context must implement Exhausted trait to tell runtime if the input has been exhausted
    impl Exhausted for Context {
        fn exhausted(&self) -> bool {
            self.cur == self.src.len()
        }
    }

    /// a custom function
    fn eat_char(pat: &'static str) -> impl Fn(&mut Context) -> bool {
        move |x: &mut Context| {
            if x.src[x.cur..].starts_with(pat) {
                x.cur += pat.len();
                true
            } else {
                false
            }
        }
    }

    rustomaton!{
        #[input(Context)]
        #[init(0)]
        #[ends(3)]

        0 -> 1: eat_char("a");
        0 -> 2: eat_char("b");
        1 -> 2: eat_char("b");
        2 -> 1: eat_char("a");
        1 -> 3: eat_char("a");
        2 -> 3: eat_char("b");
        3 -> 3: eat_char("a") || eat_char("b")
    }
}

#[test]
fn test_tmp() {
    let automaton = initial::Automaton::new();

    println!("{:?}", automaton.run(&mut initial::Context {
        src: String::from("baaaaaaaaa"),
        cur: 0
    }));
}
```
