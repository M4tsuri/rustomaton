mod initial {
    use std::rc::Rc;

    use rustomaton::rustomaton;

    #[derive(Clone)]
    pub struct Context {
        pub src: String,
        pub cur: usize
    }

    impl Exhausted for Context {
        fn exhausted(&self) -> bool {
            println!("{}", self.cur);
            self.cur == self.src.len()
        }
    }

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
    println!("{:?}", initial::run(&mut initial::Context {
        src: String::from("baab"),
        cur: 0
    }));
}