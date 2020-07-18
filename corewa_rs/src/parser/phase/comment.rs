//! In this phase, all comments are removed from the input phase.
//! Any comments like `;redcode` and `;author` will be parsed and stored in
//! load_file::Metadata. This phase also finds the origin and end of the program.

use super::CommentsRemoved;

use crate::load_file::Metadata;

enum OriginInLine {
    NewOrigin(String),
    EndWithNewOrigin(String),
    End,
    NotFound,
}

/// Parse a raw String input and return the output sans comments, with metadata.
pub fn extract_from_string(input: &str) -> CommentsRemoved {
    let mut metadata = Metadata::default();
    let mut origin: Option<String> = None;

    // Returns whether a new origin was set
    let mut set_origin = |new_origin: String| {
        if let Some(old_origin) = origin.as_ref() {
            // TODO (#25) proper warnings instead of just eprintln
            eprintln!(
                "Warning: ORG already defined as {:?}, new definition {:?} will be ignored",
                new_origin, old_origin
            );
            false
        } else {
            origin = Some(new_origin);
            true
        }
    };

    let mut lines: Vec<String> = Vec::new();

    for line in input.split_terminator('\n') {
        let trimmed_line = metadata.parse_line(line);
        if trimmed_line.is_empty() {
            continue;
        }

        if let Ok(found_origin) = find_origin_in_line(&trimmed_line) {
            lines.push(trimmed_line);

            match found_origin {
                OriginInLine::NewOrigin(new_origin) => {
                    if !set_origin(new_origin) {
                        lines.pop();
                    }
                }
                OriginInLine::EndWithNewOrigin(new_origin) => {
                    if !set_origin(new_origin) {
                        // We need to ignore the new origin, but still register this as the end.
                        // So, for now just remove the line and break
                        lines.pop();
                    }
                    break;
                }
                OriginInLine::End => break,
                OriginInLine::NotFound => (),
            }
        } else {
            // TODO (#25) return error
        }
    }

    CommentsRemoved {
        lines,
        metadata,
        origin,
    }
}

/// Find and return the origin defined in the given line.
fn find_origin_in_line(line: &str) -> Result<OriginInLine, ()> {
    let tokenized_line = line
        .split_whitespace()
        .map(|s| s.to_uppercase())
        .collect::<Vec<String>>();

    let tokens: Vec<_> = tokenized_line.iter().map(String::as_str).collect();

    use OriginInLine::*;
    match tokens[..] {
        ["ORG"] => {
            // TODO (#25) proper error handling, probably in the return type
            eprintln!("Error: ORG must be given an argument!");
            Err(())
        }
        ["ORG", new_origin] => Ok(NewOrigin(new_origin.to_owned())),
        ["END"] => Ok(End),
        ["END", new_origin] => Ok(EndWithNewOrigin(new_origin.to_owned())),
        _ => Ok(NotFound),
    }
}

#[cfg(test)]
mod test {
    use test_case::test_case;
    use textwrap_macros::dedent;

    use super::*;

    struct Param {
        input: &'static str,
        expected: CommentsRemoved,
    }

    #[test_case(
        Param {
            input: dedent!(
                "
                  foo who
                bar di bar
                baz.  "
            ),
            expected: CommentsRemoved {
                lines: vec![
                    "foo who".to_string(),
                    "bar di bar".to_string(),
                    "baz.".to_string(),
                ],
                ..Default::default()
            }
        };
        "no comments"
    )]
    #[test_case(
        Param {
            input: dedent!(
                "foo who
                ; bar di bar
                baz. ; bar"
            ),
            expected: CommentsRemoved {
                lines: vec![
                    "foo who".to_string(),
                    "baz.".to_string(),
                ],
                ..Default::default()
            }
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
            expected: CommentsRemoved {
                lines: vec!["MOV 1, 1".to_string()],
                metadata: Metadata {
                    redcode: Some("".to_string()),
                    name: Some("my-amazing-warrior".to_string()),
                    author: Some("Ian Chamberlain".to_string()),
                    ..Default::default()
                },
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
            expected: CommentsRemoved {
                lines: vec![
                    "ORG 5".to_string(),
                    "MOV 0, 1".to_string()
                ],
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
                ORG 2 ; should warn and leave org 5
                "
            ),
            expected: CommentsRemoved {
                lines: vec![
                    "ORG 5".to_string(),
                ],
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
                END 2 ; should warn and leave org 5
                "
            ),
            expected: CommentsRemoved {
                lines: vec![
                    "org 5".to_string(),
                ],
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
            expected: CommentsRemoved {
                lines: vec![
                    "MOV 1, 1".to_string(),
                    "END 2".to_string(),
                ],
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
            expected: CommentsRemoved {
                lines: vec![
                    "MOV 1, 1".to_string(),
                    "END 2".to_string(),
                ],
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
            expected: Default::default()
        };
        "empty result"
    )]
    fn parse(param: Param) {
        let result = extract_from_string(param.input);

        assert_eq!(result, param.expected);
    }

    #[test_case(
        Param {
            input: dedent!(
                "
                ORG
                MOV 0, 1
                "
            ),
            expected: CommentsRemoved {
                lines: vec!["MOV 0, 1".to_string()],
                ..Default::default()
            }
        };
        "inconclusive(should error): parse ORG without arg"
    )]
    fn parse_error(param: Param) {
        let result = extract_from_string(param.input);

        assert_eq!(result, param.expected);
    }
}
