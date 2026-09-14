use std::fmt;
use wasm_bindgen::JsValue;

#[derive(Debug)]
pub(crate) struct Error(String);

impl Error {
    pub(crate) fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }

    pub(crate) fn with_context(context: &'static str, source: impl fmt::Display) -> Self {
        Self::new(format!("{context}: {source}"))
    }
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for Error {}

impl From<JsValue> for Error {
    fn from(value: JsValue) -> Self {
        Self::new(value.as_string().unwrap_or_else(|| format!("{value:?}")))
    }
}

impl From<Error> for JsValue {
    fn from(error: Error) -> Self {
        Self::from_str(&error.0)
    }
}
