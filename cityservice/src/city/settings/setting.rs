pub enum Setting {
    String(String),
    Int(i64),
    Float(f64),
    Bool(bool),
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

impl From<i64> for Setting {
    fn from(from: i64) -> Self {
        Self::Int(from)
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
