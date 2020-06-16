//! In this phase, all comments are removed from the input phase.
//! Any comments like `;redcode` and `;author` will be parsed and stored in
//! an Info struct.

use itertools::Itertools;

use super::Cleaned;

/// Metadata about a Redcode program that is stored in the comments.
#[derive(Debug, Default, PartialEq)]
pub struct Info {
    /// The Redcode standard for this warrior (e.g. "94").
    redcode: Option<String>,

    /// The name of this warrior.
    name: Option<String>,

    /// The author of this warrior.
    author: Option<String>,

    /// The date when this warrior was written.
    date: Option<String>,

    /// The version of this warrior.
    version: Option<String>,

    /// An assertion for this warrior to ensure compilation.
    assertion: Option<String>,

    /// The entry point of the warrior. May be a label, which is why it's a String.
    origin: Option<String>,
}

impl Info {
    /// Parse a raw String input and return the output sans comments.
    pub fn extract_from_string(input: &str) -> Cleaned {
        let mut metadata = Self::default();
        let mut origin = None;

        let lines: Vec<String> = input
            .split_terminator('\n')
            .filter_map(|line| metadata.parse_line(line))
            // TODO: This leaves the last END off...
            .take_while(|line| Self::set_origin_from_line(&mut origin, line))
            .collect();

        metadata.origin = origin;

        Cleaned { lines, metadata }
    }

    fn parse_line(&mut self, line: &str) -> Option<String> {
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

        let trimmed = split_line[0].trim().to_string();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed)
        }
    }

    /// Set the value of the origin when ORG or END is encountered.
    /// Returns whether or not parsing should continue (i.e. false if END was
    /// read).
    fn set_origin_from_line(origin: &mut Option<String>, line: &str) -> bool {
        let split_trimmed: Vec<&str> = line.split_whitespace().collect();

        let mut set_origin = || {
            if let Some(&value) = split_trimmed.get(1) {
                if origin.is_some() {
                    // TODO warn user
                }
                *origin = Some(value.to_string());
            }
        };

        match split_trimmed.get(0) {
            Some(&"ORG") => {
                set_origin();
                true
            }
            Some(&"END") => {
                set_origin();
                false
            }
            _ => true,
        }
    }
}

#[cfg(test)]
mod tests {
    use predicates::prelude::*;
    use test_case::test_case;
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
        "remove comments"
    )]
    #[test_case(
        Param {
            input: dedent!(
                "
                ;redcode
                ;author Ian Chamberlain
                ; name my-amazing-warrior
                MOV 1, 1"
            ),
            expected: vec!["ORG 5".to_string()],
            info: Info {
                redcode: Some("".to_string()),
                name: Some("my-amazing-warrior".to_string()),
                author: Some("Ian Chamberlain".to_string()),
                ..Default::default()
            },
        };
        "parse info comments"
    )]
    #[test_case(
        Param {
            input: dedent!(
                "
                ORG 5
                MOV 0, 1
                ; ORG 5 behind comment ignored
                "
            ),
            expected: vec!["ORG 5".to_string()],
            info: Info {
                redcode: Some("".to_string()),
                name: Some("my-amazing-warrior".to_string()),
                author: Some("Ian Chamberlain".to_string()),
                origin: Some("5".to_string()),
                ..Default::default()
            },
        };
        "parse origin"
    )]
    #[test_case(
        Param {
            input: dedent!(
                "
                ORG 5
                END 2 ; should warn, but now ORG = 2
                "
            ),
            expected: vec![
                "ORG 5".to_string(),
                "END 2".to_string(),
            ],
            info: Info {
                origin: Some("2".to_string()),
                ..Default::default()
            }
        };
        "parse ORG and END"
    )]
    #[test_case(
        Param {
            input: dedent!(
                "
                ORG 5
                ORG 2 ; should warn, but now ORG = 2
                "
            ),
            expected: vec![
                "ORG 5".to_string(),
                "ORG 2".to_string(),
            ],
            info: Info {
                origin: Some("2".to_string()),
                ..Default::default()
            }

        };
        "parse multiple ORG"
    )]
    #[test_case(
        Param {
            input: dedent!(
                "
                MOV 1, 1
                END 2
                "
            ),
            expected: vec![
                "MOV 1, 1".to_string(),
                "END 2".to_string(),
            ],
            info: Info {
                origin: Some("2".to_string()),
                ..Default::default()
            }
        };
        "parse END"
    )]
    #[test_case(
        Param {
            input: dedent!(
                "
                MOV 1, 1
                END 2
                END 3 ; this one is ignored
                "
            ),
            expected: vec![
                "MOV 1, 1".to_string(),
                "END 2".to_string(),
            ],
            info: Info {
                origin: Some("2".to_string()),
                ..Default::default()
            }
        };
        "parse multiple END"
    )]
    #[test_case(
        Param {
            input: dedent!(
                "
                ; no real data in this input
                ; some silly comment"
            ),
            expected: vec![],
            info: Default::default(),
        };
        "empty result"
    )]
    fn parse(param: Param) {
        let result = Info::extract_from_string(param.input);
        let Cleaned { metadata, lines } = result;

        assert_eq!(metadata, param.info);
        assert_eq!(&lines, predicate::eq(param.expected));
    }
}
