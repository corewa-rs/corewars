pub use predicates;
pub use predicates_tree;

/// Simple macro to panic with a prettier error message
///
/// # Examples
///
/// ```rust
/// use predicates::prelude::*;
/// use testutil::assert_that;
///
/// assert_that!(
///     "Hello World",
///     predicates::str::similar("Hello World"),
/// );
/// assert_that!(
///     "Hello World",
///     predicates::str::diff("Goodbye World"),
/// );
/// ```
#[macro_export]
macro_rules! assert_that {
    ($item:expr, $pred:expr $(,)?) => {{
        use $crate::predicates::prelude::*;
        use $crate::predicates_tree::CaseTreeExt;

        if let Some(case) = $pred.find_case(false, $item) {
            panic!("{}", case.tree());
        };
    }};
}
