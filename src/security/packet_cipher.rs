#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EncryptedPacketHeader {
    pub nonce: [u8; 12],
    pub auth_tag: [u8; 16],
}

impl EncryptedPacketHeader {
    pub fn new(nonce: [u8; 12], auth_tag: [u8; 16]) -> Self {
        Self { nonce, auth_tag }
    }

    pub fn serialize(&self) -> [u8; 28] {
        let mut buf = [0u8; 28];
        buf[0..12].copy_from_slice(&self.nonce);
        buf[12..28].copy_from_slice(&self.auth_tag);
        buf
    }

    pub fn deserialize(buf: &[u8; 28]) -> Self {
        let mut nonce = [0u8; 12];
        let mut auth_tag = [0u8; 16];
        nonce.copy_from_slice(&buf[0..12]);
        auth_tag.copy_from_slice(&buf[12..28]);
        Self { nonce, auth_tag }
    }
}
