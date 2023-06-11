#[derive(Debug)]
pub enum DnrsError {
    Other(String),
}

impl From<std::io::Error> for DnrsError {
    fn from(value: std::io::Error) -> Self {
        Self::Other(value.to_string())
    }
}
