//! Metadata about a Redcode program. Most of this is not used for execution,
//! with some exceptions, namely `;redcode` and `;assertion`

use std::fmt;

/// Metadata about a Redcode program that is stored in the comments.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Metadata {
    /// The Redcode standard for this warrior (e.g. "94").
    // TODO #38 handle directives like `redcode-94` etc.
    pub redcode: Option<String>,

    /// The name of this warrior.
    pub name: Option<String>,

    /// The author of this warrior.
    pub author: Option<String>,

    /// The date when this warrior was written.
    pub date: Option<String>,

    /// The version of this warrior.
    pub version: Option<String>,

    /// A description of the warrior's strategy
    // TODO #38 handle multiline strategies
    pub strategy: Option<String>,

    /// An assertion for this warrior to ensure compilation.
    pub assertion: Option<String>,
}

impl Metadata {
    /// Parse warrior metadata out of a line. Any comments will be removed and
    /// the resulting string returned, with whitespace trimmed.
    pub fn parse_line(&mut self, line: &str) -> String {
        let split_line: Vec<&str> = line.splitn(2, ';').map(str::trim).collect();

        if split_line.len() > 1 {
            let split_comment: Vec<&str> = split_line[1].splitn(2, char::is_whitespace).collect();
            let value = Some(
                split_comment
                    .get(1)
                    .map_or_else(String::new, |s| s.trim().to_owned()),
            );

            let directive = split_comment[0].to_lowercase();
            match directive.as_ref() {
                "redcode" => self.redcode = value,
                "name" => self.name = value,
                "author" => self.author = value,
                "date" => self.date = value,
                "version" => self.version = value,
                "strategy" => self.strategy = value,
                "assert" => self.assertion = value,
                _ => (),
            }
        }

        split_line[0].trim().to_string()
    }
}

impl fmt::Display for Metadata {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        for (field, name) in &[
            (&self.redcode, "redcode"),
            (&self.name, "name"),
            (&self.author, "author"),
            (&self.version, "version"),
            (&self.date, "date"),
            (&self.strategy, "strategy"),
            (&self.assertion, "assert"),
        ] {
            if let Some(value) = field.as_deref() {
                if value.is_empty() {
                    writeln!(formatter, ";{}", name)?;
                } else {
                    writeln!(formatter, ";{} {}", name, value)?;
                }
            }
        }
        Ok(())
    }
}

// TODO as part of #38 test parse_line
