use std::fmt;

pub type BoxErr = Box<dyn std::error::Error + Send + Sync>;
pub type Result<T> = std::result::Result<T, BoxErr>;

#[derive(Copy, Clone, Debug)]
pub enum Error {
	BadSecretFileMode0,
	BadSecretFileMode16,
	BadSecretFileMode32,
}

impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{:?}", self)
	}
}

impl std::error::Error for Error{}

