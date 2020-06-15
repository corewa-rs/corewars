use itertools::Itertools;

use super::Buffer;

/// Metadata about a Redcode program that is stored in the comments
#[derive(Debug, Default, PartialEq)]
pub struct Info {
    redcode: Option<String>,
    name: Option<String>,
    author: Option<String>,
    date: Option<String>,
    version: Option<String>,
    assertion: Option<String>,
}

impl Info {
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

        split_line[0].trim().to_owned()
    }

    pub fn extract_from_string(input: String) -> Buffer {
        let mut info = Self::default();

        let remaining = input
            .split_terminator('\n')
            .filter_map(|line| {
                let stripped_line = info.parse_line(line);
                if stripped_line.is_empty() {
                    None
                } else {
                    Some(stripped_line)
                }
            })
            .join("\n");

        Buffer::NoComments { info, remaining }
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
        expected: &'static str,
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
            expected: dedent!(
                "
                foody who
                bar di bar
                baz."
            ).trim(),
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
            expected: dedent!(
                "
                foody who
                baz."
            ).trim(),
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
            expected: "ORG 5",
            info: Info {
                redcode: Some("".to_owned()),
                name: Some("my-amazing-warrior".to_owned()),
                author: Some("Ian Chamberlain".to_owned()),
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
            expected: "",
            info: Info {
                name: Some("my-amazing-warrior".to_owned()),
                author: Some("Ian Chamberlain".to_owned()),
                ..Default::default()
            },
        };
        "empty result"
    )]
    fn parse(param: Param) {
        let in_str = param.input.to_owned();

        let result = Info::extract_from_string(in_str);
        if let Buffer::NoComments { info, remaining } = result {
            assert_eq!(info, param.info);
            assert_that!(&remaining, predicate::str::similar(param.expected));
        } else {
            panic!("Wrong enum variant returned: {:?}", result)
        }
    }
}
