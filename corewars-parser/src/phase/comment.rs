//! In this phase, all comments are removed from the input phase.
//! Any comments like `;redcode` and `;author` will be parsed and stored in
//! [`Metadata`]. This phase also finds the origin and end of the program.

use super::CommentsRemoved;

use corewars_core::load_file::Metadata;

use crate::grammar;

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
        } else {
            origin = Some(new_origin);
        }
    };

    let mut lines: Vec<String> = Vec::new();

    for line in input.lines() {
        let trimmed_line = metadata.parse_line(line);
        if trimmed_line.is_empty() {
            continue;
        }

        if let Ok(found_origin) = find_origin_in_line(&trimmed_line) {
            match found_origin {
                OriginInLine::NewOrigin(new_origin) => {
                    set_origin(new_origin);
                }
                OriginInLine::EndWithNewOrigin(new_origin) => {
                    set_origin(new_origin);
                    break;
                }
                OriginInLine::End => break,
                OriginInLine::NotFound => lines.push(trimmed_line),
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
    use OriginInLine::{End, EndWithNewOrigin, NewOrigin, NotFound};

    let tokenized = grammar::tokenize(line);

    if tokenized.is_empty() {
        return Ok(NotFound);
    }

    match &tokenized[0].as_rule() {
        grammar::Rule::Opcode => {
            let remainder = tokenized
                .get(1)
                .map(|s| &line[s.as_span().start()..])
                .filter(|s| !s.is_empty());

            match tokenized[0].as_str().to_uppercase().as_str() {
                "ORG" => {
                    if let Some(remainder) = remainder {
                        Ok(NewOrigin(remainder.to_owned()))
                    } else {
                        // TODO (#25) proper error handling, probably in the return type
                        eprintln!("Error: ORG must be given an argument!");
                        Err(())
                    }
                }
                "END" => remainder.map_or(Ok(End), |remainder| {
                    Ok(EndWithNewOrigin(remainder.to_owned()))
                }),
                _ => Ok(NotFound),
            }
        }
        _ => Ok(NotFound),
    }
}

#[cfg(test)]
mod test {
    #![allow(clippy::default_trait_access)]

    use test_case::test_case;
    use textwrap_macros::dedent;

    use super::*;

    struct Param {
        input: &'static str,
        expected: CommentsRemoved,
    }

    #[test_case(
        &Param {
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
        &Param {
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
        &Param {
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
        &Param {
            input: dedent!(
                "
                ORG 5
                MOV 0, 1
                ; ORG 4 behind comment ignored
                "
            ),
            expected: CommentsRemoved {
                lines: vec![
                    "MOV 0, 1".to_string()
                ],
                origin: Some("5".to_string()),
                ..Default::default()
            },
        };
        "parse ORG"
    )]
    #[test_case(
        &Param {
            input: dedent!(
                "
                ORG lbl1
                lbl1 MOV 0, 1
                ; ORG 4 behind comment ignored
                "
            ),
            expected: CommentsRemoved {
                lines: vec![
                    "lbl1 MOV 0, 1".to_string()
                ],
                origin: Some("lbl1".to_string()),
                ..Default::default()
            },
        };
        "parse ORG label"
    )]
    #[test_case(
        &Param {
            input: dedent!(
                "
                ORG lbl1 + 1
                lbl1 MOV 0, 1
                ; ORG 4 behind comment ignored
                "
            ),
            expected: CommentsRemoved {
                lines: vec![
                    "lbl1 MOV 0, 1".to_string()
                ],
                origin: Some("lbl1 + 1".to_string()),
                ..Default::default()
            },
        };
        "parse ORG expression"
    )]
    #[test_case(
        &Param {
            input: dedent!(
                "
                ORG 5
                ORG 2 ; should warn and leave org 5
                "
            ),
            expected: CommentsRemoved {
                lines: vec![],
                origin: Some("5".to_string()),
                ..Default::default()
            }

        };
        "parse multiple ORG"
    )]
    #[test_case(
        &Param {
            input: dedent!(
                "
                org 5
                END 2 ; should warn and leave org 5
                "
            ),
            expected: CommentsRemoved {
                lines: vec![],
                origin: Some("5".to_string()),
                ..Default::default()
            }
        };
        "parse ORG and END"
    )]
    #[test_case(
        &Param {
            input: dedent!(
                "
                MOV 1, 1

                END 2
                "
            ),
            expected: CommentsRemoved {
                lines: vec!["MOV 1, 1".to_string()],
                origin: Some("2".to_string()),
                ..Default::default()
            }
        };
        "parse END"
    )]
    #[test_case(
        &Param {
            input: dedent!(
                "
                MOV 1, 1
                END 2
                end 3 ; this one is ignored
                stuff here should also be ignored
                "
            ),
            expected: CommentsRemoved {
                lines: vec!["MOV 1, 1".to_string()],
                origin: Some("2".to_string()),
                ..Default::default()
            }
        };
        "parse multiple END"
    )]
    #[test_case(
        &Param {
            input: dedent!(
                "
                ; no real data in this input
                ; some silly comment"
            ),
            expected: Default::default()
        };
        "empty result"
    )]
    fn parse(param: &Param) {
        let result = extract_from_string(param.input);

        assert_eq!(result, param.expected);
    }

    #[test_case(
        &Param {
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
    fn parse_error(param: &Param) {
        let result = extract_from_string(param.input);

        assert_eq!(result, param.expected);
    }
}
