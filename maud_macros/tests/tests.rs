#![feature(plugin)]
#![plugin(maud_macros)]

extern crate maud;

#[test]
fn literals() {
    let s = html!("du\tcks" -23 3.14 '\n' "geese").to_string();
    assert_eq!(s, "du\tcks-233.14\ngeese");
}

#[test]
fn escaping() {
    let s = html!("<flim&flam>").to_string();
    assert_eq!(s, "&lt;flim&amp;flam&gt;");
}

#[test]
fn semicolons() {
    let s = html! {
        "one";
        "two";
        "three";
        ;;;;;;;;;;;;;;;;;;;;;;;;
        "four";
    }.to_string();
    assert_eq!(s, "onetwothreefour");
}

#[test]
fn blocks() {
    let s = html! {
        "hello"
        {
            " ducks" " geese"
        }
        " swans"
    }.to_string();
    assert_eq!(s, "hello ducks geese swans");
}

mod elements {
    #[test]
    fn simple() {
        let s = html! {
            p { b { "pickle" } "barrel" i { "kumquat" } }
        }.to_string();
        assert_eq!(s, "<p><b>pickle</b>barrel<i>kumquat</i></p>");
    }

    #[test]
    fn nesting() {
        let s = html!(html body div p sup "butts").to_string();
        assert_eq!(s, "<html><body><div><p><sup>butts</sup></p></div></body></html>");
    }

    #[test]
    fn empty() {
        let s = html!("pinkie" br/ "pie").to_string();
        assert_eq!(s, "pinkie<br>pie");
    }

    #[test]
    fn attributes() {
        let s = html! {
            link rel="stylesheet" href="styles.css"/
            section id="midriff" {
                p class="hotpink" "Hello!"
            }
        }.to_string();
        assert_eq!(s, concat!(
                r#"<link rel="stylesheet" href="styles.css">"#,
                r#"<section id="midriff"><p class="hotpink">Hello!</p></section>"#));
    }

    #[test]
    fn empty_attributes() {
        let s = html! { div readonly? input type="checkbox" checked? / }.to_string();
        assert_eq!(s, r#"<div readonly><input type="checkbox" checked></div>"#);
    }
}

mod splices {
    #[test]
    fn literals() {
        let s = html! { $"<pinkie>" }.to_string();
        assert_eq!(s, "&lt;pinkie&gt;");
    }

    #[test]
    fn raw_literals() {
        let s = html! { $$"<pinkie>" }.to_string();
        assert_eq!(s, "<pinkie>");
    }

    #[test]
    fn blocks() {
        let s = html! {
            ${
                let mut result = 1i32;
                for i in 2..11 {
                    result *= i;
                }
                result
            }
        }.to_string();
        assert_eq!(s, "3628800");
    }

    #[test]
    fn attributes() {
        let rocks = true;
        let s = html! {
            input checked?=true /
            input checked?=false /
            input checked?=rocks /
            input checked?=(!rocks) /
        }.to_string();
        assert_eq!(s, concat!(
                r#"<input checked>"#,
                r#"<input>"#,
                r#"<input checked>"#,
                r#"<input>"#));
    }

    static BEST_PONY: &'static str = "Pinkie Pie";

    #[test]
    fn statics() {
        let s = html! { $BEST_PONY }.to_string();
        assert_eq!(s, "Pinkie Pie");
    }

    #[test]
    fn closures() {
        let best_pony = "Pinkie Pie";
        let s = html! { $best_pony }.to_string();
        assert_eq!(s, "Pinkie Pie");
    }

    /// An example struct, for testing purposes only
    struct Creature {
        name: &'static str,
        /// Rating out of 10, where:
        /// * 0 is a naked mole rat with dysentery
        /// * 10 is Sweetie Belle in a milkshake
        adorableness: u32,
    }

    impl Creature {
        fn repugnance(&self) -> u32 {
            10 - self.adorableness
        }
    }

    #[test]
    fn structs() {
        let pinkie = Creature {
            name: "Pinkie Pie",
            adorableness: 9,
        };
        let s = html! {
            "Name: " $pinkie.name ". Rating: " $pinkie.repugnance()
        }.to_string();
        assert_eq!(s, "Name: Pinkie Pie. Rating: 1");
    }

    #[test]
    fn nested_macro_invocation() {
        let best_pony = "Pinkie Pie";
        let s = html! { $(format!("{}", best_pony)) }.to_string();
        assert_eq!(s, "Pinkie Pie");
    }
}

#[test]
fn issue_1() {
    let markup = html! { "Test" };
    let _ = markup.to_string();
}

#[test]
fn issue_13() {
    let owned = String::from("yay");
    let _ = html! { $owned }.to_string();
}

mod control {
    #[test]
    fn if_expr() {
        for (number, &name) in (1..4).zip(["one", "two", "three"].iter()) {
            let s = html! {
                $if number == 1 {
                    "one"
                } $else if number == 2 {
                    "two"
                } $else if number == 3 {
                    "three"
                } $else {
                    "oh noes"
                }
            }.to_string();
            assert_eq!(s, name);
        }
    }

    #[test]
    fn if_let() {
        for &(input, output) in [(Some("yay"), "yay"), (None, "oh noes")].iter() {
            let s = html! {
                $if let Some(value) = input {
                    $value
                } $else {
                    "oh noes"
                }
            }.to_string();
            assert_eq!(s, output);
        }
    }

    #[test]
    fn for_expr() {
        let ponies = ["Apple Bloom", "Scootaloo", "Sweetie Belle"];
        let s = html! {
            ul $for pony in &ponies {
                li $pony
            }
        }.to_string();
        assert_eq!(s, concat!(
                "<ul>",
                "<li>Apple Bloom</li>",
                "<li>Scootaloo</li>",
                "<li>Sweetie Belle</li>",
                "</ul>"));
    }
}
