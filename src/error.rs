#[derive(Debug)]
pub enum DnsError {
    FormatError,
    ServerFailure(String),
    NameError,
    NotImplemented,
    Refused,
}

impl From<std::io::Error> for DnsError {
    fn from(value: std::io::Error) -> Self {
        Self::ServerFailure(value.to_string())
    }
}
