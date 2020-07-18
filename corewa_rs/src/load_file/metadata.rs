/// Metadata about a Redcode program that is stored in the comments.
#[derive(Debug, Default, PartialEq)]
pub struct Metadata {
    /// The Redcode standard for this warrior (e.g. "94").
    pub redcode: Option<String>,

    /// The name of this warrior.
    pub name: Option<String>,

    /// The author of this warrior.
    pub author: Option<String>,

    /// The date when this warrior was written.
    pub date: Option<String>,

    /// The version of this warrior.
    pub version: Option<String>,

    /// An assertion for this warrior to ensure compilation.
    pub assertion: Option<String>,
}

impl Metadata {
    /// Parse warrior metadata out of a line. Any comments will be removed and
    /// the resulting string returned, with whitespace trimmed.
    pub fn parse_line(&mut self, line: &str) -> String {
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

    pub fn to_lines(&self) -> Vec<String> {
        todo!()
    }
}
