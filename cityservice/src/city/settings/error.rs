#[derive(Clone, Debug)]
pub enum Error {
    NonexistantSetting(Vec<String>),
    WrongType(Vec<String>),
    LineError{
        line: u32,
        cause: LineError
    },
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NonexistantSetting(path) => write!(
                f, "You tried to get or modify a nonexistent \
                    setting at path {}", path.join(".")
            ),
            Self::WrongType(path) => write!(
                f, "You set the wrong type for {}, if it's a number, \
                    then check units are correct and make sure there \
                    are no spaces between the number and unit.",
                path.join("."),
            ),
            Self::LineError{
                line,
                cause,
            }=> write!(
                f, "Syntax error on line {}: {}", line + 1, cause
            ),
        }
    }
}

#[derive(Clone, Debug)]
pub enum LineError {
    WordCount,
    Syntax,
    ExpectedNumber,
    NonexistantSetting(Vec<String>)
}

impl std::fmt::Display for LineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::WordCount => format!(
                "More or less than 4 words in a line \
                 is not allowed, do you have an extra space?"
            ),
            Self::Syntax => format!(
                "Did you use proper syntax with the \"let\" \
                 and the \"be\"?"
            ),
            Self::ExpectedNumber => format!(
                "The service expected a number \
                 because of a trailing unit (m, km), \
                 but the value was not parsable as a number"
            ),
            Self::NonexistantSetting(setting) => format!(
                "You tried to update \
                 a nonexistant setting: {}",
                setting.join(".")),
        };

        write!(f, "{}", message)
    }
}
