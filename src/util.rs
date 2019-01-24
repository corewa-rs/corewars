/// # Syntax
///
/// ```
/// enum_string!(pub Foo, {
///     Bar => "Bar",
///     Baz => "BAZ"
/// })
/// ```
/// This will generate a `pub enum Foo` with variants `Bar` and `Baz`, which
/// implements `std::str::FromStr` and `std::string::ToString` for the string
/// values specified.
macro_rules! enum_string {
    ($vis:vis $name:ident, {
        $($variant:ident => $value:expr,)*
    }) => {
        #[derive(Copy, Clone, Debug, PartialEq)]
        $vis enum $name {
            $($variant,)*
        }

        impl ::std::string::ToString for $name {
            fn to_string(&self) -> String {
                use self::$name::*;
                match *self {
                    $($variant => $value,)*
                }.to_owned()
            }
        }

        impl ::std::str::FromStr for $name {
            type Err = String;
            fn from_str(input_str: &str) -> Result<Self, Self::Err> {
                use self::$name::*;
                match input_str {
                    $($value => Ok($variant),)*
                    _ => Err(format!(
                        "No variant '{}' found for enum '{}'",
                        input_str,
                        stringify!($name),
                    )),
                }
            }
        }
    };
}

mod tests {
    // seems to be a case of https://github.com/rust-lang/rust/issues/45268
    #[allow(unused_imports)]
    use std::{str::FromStr, string::ToString};

    mod submod {
        enum_string!(pub Foo, {
            Bar => "Bar",
        });
    }

    enum_string!(Foo, {
        Bar => "Bar",
        Baz => "Baz",
        SomethingElse => "blahblah",
    });

    #[test]
    fn pub_visible() {
        let _ = submod::Foo::Bar;
    }

    #[test]
    fn to_string() {
        assert_eq!(Foo::Bar.to_string(), "Bar");
        assert_eq!(Foo::Baz.to_string(), "Baz");
        assert_eq!(Foo::SomethingElse.to_string(), "blahblah");
    }

    #[test]
    fn from_string() {
        assert_eq!(Foo::from_str("Bar").unwrap(), Foo::Bar);
        assert_eq!(Foo::from_str("Baz").unwrap(), Foo::Baz);
        assert_eq!(Foo::from_str("blahblah").unwrap(), Foo::SomethingElse);

        assert_eq!(
            Foo::from_str("Should fail"),
            Err("No variant 'Should fail' found for enum 'Foo'".to_owned())
        );
    }
}
