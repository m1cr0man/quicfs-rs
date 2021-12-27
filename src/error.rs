pub type Error = Box<dyn std::error::Error>;

macro_rules! return_err_boxed {
    ($var:ident) => {
        return Err(Box::new($var))
    };
}

// Fancy syntax to export/publicise the macro
pub(crate) use return_err_boxed;

#[derive(Debug, Clone)]
pub struct GeneralError {
    pub message: String,
}

impl From<&str> for GeneralError {
    fn from(msg: &str) -> Self {
        Self {
            message: String::from(msg),
        }
    }
}

impl std::fmt::Display for GeneralError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for GeneralError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}
