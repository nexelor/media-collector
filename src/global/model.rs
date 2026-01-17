use std::fmt;

#[derive(Debug, Clone)]
pub struct ParseEnumError {
    pub enum_name: &'static str,
    pub value: String,
    pub expected: &'static [&'static str],
}

impl fmt::Display for ParseEnumError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Failed to parse {} from '{}'. Expected one of: {:?}",
            self.enum_name, self.value, self.expected
        )
    }
}