//! In this phase, all comments are removed from the input phase.
//! Any comments like `;redcode` and `;author` will be parsed and stored in
//! an Info struct.

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

        let mut set_origin = |new_origin: &str| {
            if let Some(old_origin) = origin.as_ref() {
                // TODO (#25) proper warnings instead of just eprintln
                eprintln!(
                    "Warning: ORG already defined as {:?}, new definition {:?} will be ignored",
                    new_origin, old_origin
                );
            } else {
                origin = Some(new_origin.to_owned());
            }
        };

        let lines = {
            let mut lines = Vec::new();
            for line in input
                .split_terminator('\n')
                .filter_map(|line| metadata.parse_line(line))
            {
                let split_line: Vec<String> =
                    line.split_whitespace().map(|s| s.to_uppercase()).collect();
                let split_line_str: Vec<&str> = split_line.iter().map(|s| s.as_str()).collect();

                if !split_line.is_empty() {
                    match split_line_str[..] {
                        ["ORG"] => {
                            // TODO (#25) proper error handling, probably in the return type
                            eprintln!("Error: ORG must be given an argument!")
                        }
                        ["ORG", new_origin] => set_origin(new_origin),
                        ["END"] => {
                            lines.push(line);
                            break;
                        }
                        ["END", new_origin] => {
                            set_origin(new_origin);
                            lines.push(line);
                            break;
                        }
                        _ => (),
                    }
                }

                lines.push(line);
            }
            lines
        };

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
}

#[cfg(test)]
mod tests {
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
            expected: vec!["MOV 1, 1".to_string()],
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
                ; ORG 4 behind comment ignored
                "
            ),
            expected: vec![
                "ORG 5".to_string(),
                "MOV 0, 1".to_string()
            ],
            info: Info {
                origin: Some("5".to_string()),
                ..Default::default()
            },
        };
        "parse ORG"
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
                origin: Some("5".to_string()),
                ..Default::default()
            }

        };
        "parse multiple ORG"
    )]
    #[test_case(
        Param {
            input: dedent!(
                "
                org 5
                END 2 ; should warn, but now ORG = 2
                "
            ),
            expected: vec![
                "org 5".to_string(),
                "END 2".to_string(),
            ],
            info: Info {
                origin: Some("5".to_string()),
                ..Default::default()
            }
        };
        "parse ORG and END"
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
                end 3 ; this one is ignored
                stuff here should also be ignored
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

        assert_eq!(lines, param.expected);
        assert_eq!(metadata, param.info);
    }
}
