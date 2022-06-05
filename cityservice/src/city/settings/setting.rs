pub enum Setting {
    String(String),
    Uint(u32),
    Float(f64),
    Bool(bool),
}

pub enum Error {
    WrongType
}

impl TryFrom<&Setting> for String {
    type Error = Error;
    
    fn try_from(from: &Setting) -> Result<Self, Error> {
        if let Setting::String(string) = from {
            Ok(string.clone())
        } else {
            Err(Error::WrongType)
        }
    }
}

impl TryFrom<&Setting> for u32 {
    type Error = Error;

    fn try_from(from: &Setting) -> Result<Self, Error> {
        if let Setting::Uint(integer) = from {
            Ok(*integer)
        } else {
            Err(Error::WrongType)
        }
    }
}

impl TryFrom<&Setting> for f64 {
    type Error = Error;

    fn try_from(from: &Setting) -> Result<Self, Error> {
        if let Setting::Float(float) = from {
            Ok(*float)
        } else {
            Err(Error::WrongType)
        }
    }
}

impl TryFrom<&Setting> for bool {
    type Error = Error;

    fn try_from(from: &Setting) -> Result<Self, Error> {
        if let Setting::Bool(boolean) = from {
            Ok(*boolean)
        } else {
            Err(Error::WrongType)
        }
    }
}

impl From<String> for Setting {
    fn from(from: String) -> Self {
        Self::String(from)
    }
}

impl From<&str> for Setting {
    fn from(from: &str) -> Self {
        Self::String(String::from(from))
    }
}

impl From<u32> for Setting {
    fn from(from: u32) -> Self {
        Self::Uint(from)
    }
}

impl From<f64> for Setting {
    fn from(from: f64) -> Self {
        Self::Float(from)
    }
}

impl From<bool> for Setting {
    fn from(from: bool) -> Self {
        Self::Bool(from)
    }
}
