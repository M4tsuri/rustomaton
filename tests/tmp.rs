mod initial {
    use rustomaton::rustomaton;

    pub struct Context {
        pub src: String,
        pub cur: usize
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

        #[init(5)]
        #[ends(3)]

        5 -> 0: eat_char("a");
        0 -> 1: eat_char("b");
        0 -> 2: eat_char("c");
        1 -> 3: eat_char("d");
        2 -> 3: eat_char("d")
    }
}

#[test]
fn test_tmp() {
    println!("{:?}", initial::run(&mut initial::Context {
        src: String::from("abd"),
        cur: 0
    }));
}