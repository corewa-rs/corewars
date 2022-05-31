/// Generate an enum impl with string conversion methods.
///
/// # Examples
///
/// ```
/// # #[macro_use]
/// # extern crate corewars_core;
/// #
/// enum_string! {
///     #[derive(Copy, Clone, Debug, PartialEq, Eq)]
///     pub enum Foo {
///         Bar => "Bar",
///         Baz => "BAZ",
///     }
/// }
///
/// # fn main() {
/// assert_eq!(&Foo::Bar.to_string(), "Bar");
/// assert_eq!(&Foo::Baz.to_string(), "BAZ");
/// # }
/// ```
///
/// This will generate a `pub enum Foo` with variants `Bar` and `Baz`, which
/// implements `std::str::FromStr` and `std::fmt::Display` for the string
/// values specified.
///
// This really should have #[cfg_attr(doctest, macro_export)]
// But cfg(doctest) does not work as expected: https://github.com/rust-lang/rust/issues/67295
#[macro_export]
macro_rules! enum_string {
    (
        $(#[$attr:meta])*
        $vis:vis enum $name:ident {
            $($variant:ident => $value:expr),* $(,)?
        }
    ) => {
        $(#[$attr])*
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

#[cfg(test)]
mod test {
    use std::str::FromStr;

    mod submod {
        enum_string! {
            #[derive(Debug, PartialEq, Eq)]
            pub enum Foo {
                Bar => "Bar",
            }
        }

        enum_string! {
            pub enum Comma {
                NoTrailing => "still works"
            }
        }
    }

    enum_string! {
        #[derive(Debug, PartialEq, Eq, Clone, Copy)]
        enum Foo {
            Bar => "Bar",
            Baz => "Baz",
            SomethingElse => "blahblah",
        }
    }

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
        let values_from_iter: Vec<Foo> = Foo::iter_values().copied().collect();
        assert_eq!(
            values_from_iter,
            vec![Foo::Bar, Foo::Baz, Foo::SomethingElse]
        );
    }
}
