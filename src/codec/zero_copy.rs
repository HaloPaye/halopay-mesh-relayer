#[derive(Debug, PartialEq, Eq)]
pub struct FrameHeader {
    pub magic: u16,
    pub payload_len: u32,
}

pub fn parse_frame_header(buf: &[u8]) -> Option<FrameHeader> {
    if buf.len() < 6 {
        return None;
    }
    let magic = u16::from_be_bytes([buf[0], buf[1]]);
    let payload_len = u32::from_be_bytes([buf[2], buf[3], buf[4], buf[5]]);
    Some(FrameHeader { magic, payload_len })
}