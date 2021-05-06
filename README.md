# A toy automaton engine for Rust

Now you can use a macro like this:

```rust
rustomaton! {
    #[init(initial_state)]
    /// an automaton can have multiple final states 
    #[ends(final_state1, final_state2, ...)]
    
    /// an automaton edge has the form: 
    start -> end: "some string";
    
    /// a transfer function function can be empty:
    start -> end: _;
    
    /// or a bool expression
    start -> end: "str1" | "str2"

    /// note that there cannot be two rules with the same edge, use a OR instead
    /// be aware that there is no extra semicolon after the last rule
}
```

this macro will generate an `Automaton` struct which implements a new method for the input type.

just new a instance and call the `run` method.

for example:

```rust
mod initial {
    pub use rustomaton::{AutomatonContext, Automaton};
    use rustomaton_derive::rustomaton;

    rustomaton!{
        #[init(0)]
        #[ends(3)]

        0 -> 1: "a";
        0 -> 2: "b";
        1 -> 2: "b";
        2 -> 1: "a";
        1 -> 3: "a";
        2 -> 3: "b";
        3 -> 3: "a" | "b"
    }
}

#[test]
fn test_tmp() {
    use initial::NewAutomaton;
    let automaton = initial::Automaton::new();

    println!("{:?}", automaton.run(&mut initial::AutomatonContext {
        src: String::from("baaaaaaaaa"),
        cur: 0
    }));
}
```
