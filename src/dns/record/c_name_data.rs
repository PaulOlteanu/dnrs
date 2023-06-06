#[derive(Debug)]
pub struct CNameData(String);

impl CNameData {
    pub fn from_data(bytes: &[u8]) -> Result<Self, ()> {
        Ok(Self(std::str::from_utf8(bytes).or(Err(()))?.to_owned()))
    }

    pub fn to_data(&self) -> Vec<u8> {
        self.0.as_bytes().to_vec()
    }
}
