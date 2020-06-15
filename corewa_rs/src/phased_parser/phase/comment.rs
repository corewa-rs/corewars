//! In this phase, all comments are removed from the input
//! [buffer](../struct.Buffer.html). Any comments like ";redcode" and ";author"
//! will be parsed and stored in an [Info](struct.Info.html) struct.

/// The state of [Buffer](../struct.Buffer.html) after comments have been removed
/// and metadata parsed from the comments.
pub struct Stripped {
    pub lines: Vec<String>,
    pub metadata: Info,
}

/// Metadata about a Redcode program that is stored in the comments
#[derive(Debug, Default, PartialEq)]
pub struct Info {
    /// The Redcode standard for this warrior (e.g. "94")
    redcode: Option<String>,
    /// The name of this warrior
    name: Option<String>,
    /// The author of this warrior
    author: Option<String>,
    /// The date when this warrior was written
    date: Option<String>,
    /// The version of this warrior
    version: Option<String>,
    /// An assertion for this warrior to ensure compilation
    assertion: Option<String>,
}

impl Info {
    /// Parse a raw String input and return the [stripped](struct.Stripped.html)
    /// output.
    pub fn extract_from_string(input: &str) -> Stripped {
        let mut metadata = Self::default();

        let lines: Vec<String> = input
            .split_terminator('\n')
            .filter_map(|line| {
                let stripped_line = metadata.parse_line(line);
                if stripped_line.is_empty() {
                    None
                } else {
                    Some(stripped_line)
                }
            })
            .collect();

        Stripped { lines, metadata }
    }

    fn parse_line(&mut self, line: &str) -> String {
        let split_line: Vec<&str> = line.splitn(2, ';').map(|p| p.trim()).collect();

        if split_line.len() > 1 {
            let split_comment: Vec<&str> = split_line[1].splitn(2, char::is_whitespace).collect();
            let value = Some(
                split_comment
                    .get(1)
                    .map_or_else(String::new, ToString::to_string),
            );

            match split_comment[0] {
                "redcode" => self.redcode = value,
                "name" => self.name = value,
                "author" => self.author = value,
                "date" => self.date = value,
                "version" => self.version = value,
                "assertion" => self.assertion = value,
                _ => (),
            }
        }

        split_line[0].trim().to_string()
    }
}

#[cfg(test)]
mod tests {
    use test_case::test_case;
    use testutils::assert_that;
    use textwrap_macros::dedent;

    use super::*;

    struct Param {
        input: &'static str,
        expected: Vec<String>,
        info: Info,
    }

    #[test_case(
        Param {
            input: dedent!(
                "
                  foody who
                bar di bar
                baz.  "
            ),
            expected: vec![
                "foody who".to_string(),
                "bar di bar".to_string(),
                "baz.".to_string(),
            ],
            info: Info::default(),
        };
        "no comments"
    )]
    #[test_case(
        Param {
            input: dedent!(
                "foody who
                ; bar di bar
                baz. ; bar"
            ),
            expected: vec![
                "foody who".to_string(),
                "baz.".to_string(),
            ],
            info: Info::default(),
        };
        "comments removed"
    )]
    #[test_case(
        Param {
            input: dedent!(
                "
                ;redcode
                ;author Ian Chamberlain
                ; name my-amazing-warrior
                ORG 5"
            ),
            expected: vec!["ORG 5".to_string()],
            info: Info {
                redcode: Some("".to_string()),
                name: Some("my-amazing-warrior".to_string()),
                author: Some("Ian Chamberlain".to_string()),
                ..Default::default()
            },
        };
        "info comments parsed"
    )]
    #[test_case(
        Param {
            input: dedent!(
                "
                ;author Ian Chamberlain
                ; name my-amazing-warrior
                ; some silly comment"
            ),
            expected: vec![],
            info: Info {
                name: Some("my-amazing-warrior".to_string()),
                author: Some("Ian Chamberlain".to_string()),
                ..Default::default()
            },
        };
        "empty result"
    )]
    fn parse(param: Param) {
        let result = Info::extract_from_string(param.input);
        let Stripped { metadata, lines } = result;

        assert_eq!(metadata, param.info);
        assert_that!(&lines, predicate::eq(param.expected));
    }
}
