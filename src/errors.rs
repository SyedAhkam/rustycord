/// All errors from this crate implement this trait
pub trait RustyCordError {
    fn cause(&self) -> String;
}

impl std::fmt::Debug for dyn RustyCordError + Send + Sync {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "RustyCordError: {}", self.cause())
    }
}

pub type RustyCordResult<T> = Result<T, Box<dyn RustyCordError + Send + Sync>>;

#[derive(Debug, Clone)]
pub struct ClientException(pub String);

impl RustyCordError for ClientException {
    fn cause(&self) -> String {
        format!("ClientException({})", self.0)
    }
}

#[derive(Debug, Clone)]
pub struct HTTPException(pub String);

impl RustyCordError for HTTPException {
    fn cause(&self) -> String {
        format!("HTTPException({})", self.0)
    }
}