/// Simple macro to panic with a prettier error message
/// # Examples
/// ```
/// use predicates::prelude::*;
/// use testutils::assert_that;
///
/// assert_that!(
///     "Hello World",
///     predicates::str::similar("Hello World"),
/// );
/// assert_that!(
///     "Hello world",
///     predicates::str::diff("Goodbye world"),
/// );
/// ```
#[macro_export]
macro_rules! assert_that {
    ($item:expr, $pred:expr $(,)?) => {{
        use ::predicates::prelude::*;
        use ::predicates_tree::CaseTreeExt;

        if let Some(case) = $pred.find_case(false, $item) {
            panic!("{}", case.tree());
        };
    }};
}
