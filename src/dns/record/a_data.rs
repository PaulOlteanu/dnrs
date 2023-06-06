#[derive(Debug)]
pub struct AData(pub [u8; 4]);

impl AData {
    pub fn from_data(bytes: &[u8]) -> Result<Self, ()> {
        if bytes.len() < 4 {
            return Err(());
        }

        Ok(Self(bytes[0..4].try_into().unwrap()))
    }

    pub fn to_data(&self) -> Vec<u8> {
        self.0.to_vec()
    }
}
