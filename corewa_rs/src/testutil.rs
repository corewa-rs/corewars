pub use predicates;
pub use predicates_tree;

/// Simple macro to panic with a prettier error message
///
/// # Examples
///
/// ```
/// use predicates::prelude::*;
/// use testutil::assert_that;
///
/// assert_that!(
///     "Hello World",
///     str::similar("Hello World"),
/// );
///
/// assert_that!(
///     "Hello World",
///     str::diff("Goodbye World"),
/// );
///
/// assert_that!("Hello World", eq("Goodbye World"));
/// ```
#[macro_export]
macro_rules! assert_that {
    (
        $value:expr,
        $($pred:tt)+ ( $args:tt ) $(,)?
    ) => {{
        use predicate::*;
        use $crate::predicates::prelude::*;
        use $crate::predicates_tree::CaseTreeExt;

        if let Some(case) = $($pred)+ rgo ($args).find_case(false, $value) {
            panic!("{}", case.tree());
        };
    }};
}
