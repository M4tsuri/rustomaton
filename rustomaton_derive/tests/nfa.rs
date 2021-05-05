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
    fn eat(pat: &'static str) -> impl Fn(&mut Context) -> bool {
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
        #[ends(7)]

        0 -> 5: _;
        5 -> 5: eat("a") || eat("b");
        5 -> 1: _;
        1 -> 3: eat("a");
        1 -> 4: eat("b");
        3 -> 2: eat("a");
        4 -> 2: eat("b");
        2 -> 6: _;
        6 -> 6: eat("a") || eat("b");
        6 -> 7: _
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