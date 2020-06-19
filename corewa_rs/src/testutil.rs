pub use predicates;
pub use predicates_tree;

/// Simple macro to make assertions with a better error message.
///
/// # Examples
///
/// ```
/// extern crate corewa_rs;
/// use corewa_rs::assert_that;
///
/// assert_that!("Hello World", str::similar("Hello World"));
///
/// assert_that!("Hello World", str::diff("Goodbye World"));
///
/// // Can be used with more complex predicates
/// assert_that!(
///     &1234,
///     ge(-5).and(le(i16::MAX))
/// );
/// ```
#[macro_export]
macro_rules! assert_that {
    ($value:expr, $pred:expr $(,)?) => {{
        use $crate::predicates::prelude::*;
        use $crate::predicates_tree::CaseTreeExt;

        use predicate::*;

        if let Some(case) = $pred.find_case(false, $value) {
            panic!("{}", case.tree());
        };
    }};
}
