mod initial {
    pub use rustomaton::{AutomatonContext, Automaton};
    use rustomaton_macro::rustomaton;

    rustomaton! {
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