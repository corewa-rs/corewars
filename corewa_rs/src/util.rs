/// Generate an enum impl with string conversion methods.
///
/// # Examples
///
/// ```
/// enum_string!(pub Foo {
///     Bar => "Bar",
///     Baz => "BAZ"
/// })
/// ```
///
/// This will generate a `pub enum Foo` with variants `Bar` and `Baz`, which
/// implements `std::str::FromStr` and `std::fmt::Display` for the string
/// values specified.
macro_rules! enum_string {
    ($vis:vis $name:ident {
        $($variant:ident => $value:expr),* $(,)?
    }) => {
        #[derive(Copy, Clone, Debug, PartialEq)]
        $vis enum $name {
            $($variant,)*
        }

        impl ::std::fmt::Display for $name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                match *self {
                    $(Self::$variant => f.pad($value),)*
                }
            }
        }

        impl ::std::str::FromStr for $name {
            type Err = String;
            fn from_str(input_str: &str) -> Result<Self, Self::Err> {
                match input_str {
                    $($value => Ok(Self::$variant),)*
                    _ => Err(format!(
                        "No variant '{}' found for enum '{}'",
                        input_str,
                        stringify!($name),
                    )),
                }
            }
        }

        impl $name {
            #[allow(dead_code)]
            pub fn iter_values() -> ::std::slice::Iter<'static, Self> {
                const VALUES: &[$name] = &[$($name::$variant,)*];
                VALUES.iter()
            }
        }
    };
}

/// Print a format string like `dbg!` without stringify-ing the input.
///
/// # Examples:
///
/// ```
/// dbgf!("Print: {:?}", "123");
/// ```
#[cfg(debug_assertions)]
macro_rules! dbgf {
    ( $fmt:expr $(, $($args:expr),* $(,)? )? ) => {
        eprintln!(
            concat!("[{}:{}] ", $fmt),
            file!(),
            line!(),
            $($($args),* )?
        )
    }
}

#[cfg(test)]
mod test {
    use std::str::FromStr;

    mod submod {
        enum_string!(pub Foo {
            Bar => "Bar",
        });

        enum_string!(pub Comma {
            NoTrailing => "still works"
        });
    }

    enum_string!(Foo {
        Bar => "Bar",
        Baz => "Baz",
        SomethingElse => "blahblah",
    });

    #[test]
    fn pub_visible() {
        let _ = submod::Foo::Bar;
        let _ = submod::Comma::NoTrailing;
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

    #[test]
    fn iter_values() {
        let values_from_iter: Vec<Foo> = Foo::iter_values().cloned().collect();
        assert_eq!(
            values_from_iter,
            vec![Foo::Bar, Foo::Baz, Foo::SomethingElse]
        );
    }
}
