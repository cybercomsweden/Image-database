use actix_web::ResponseError;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct Error(anyhow::Error);

impl<E> From<E> for Error
where
    E: Into<anyhow::Error>,
{
    fn from(error: E) -> Self {
        Self(error.into())
    }
}

impl From<Error> for Box<dyn std::error::Error + 'static + Send + Sync> {
    fn from(error: Error) -> Self {
        error.0.into()
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // TODO: Use proper error
        dbg!(self);
        write!(f, "Error!")
    }
}

impl ResponseError for Error {}
