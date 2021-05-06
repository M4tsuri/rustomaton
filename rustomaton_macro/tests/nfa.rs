mod initial {
    pub use rustomaton::{AutomatonContext, Automaton};
    use rustomaton_macro::rustomaton;

    rustomaton! {
        #[init(0)]
        #[ends(7)]

        0 -> 5: _;
        5 -> 5: "a" | "b";
        5 -> 1: _;
        1 -> 3: "a";
        1 -> 4: "b";
        3 -> 2: "a";
        4 -> 2: "b";
        2 -> 6: _;
        6 -> 6: "a" | "b";
        6 -> 7: _
    }
}

#[test]
fn test_tmp() {
    let automaton = initial::Automaton::new();

    println!("{:?}", automaton.run(&mut initial::AutomatonContext {
        src: String::from("baaaaaaaaa"),
        cur: 0
    }));
}