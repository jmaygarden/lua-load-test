use super::{Error, Result};
use std::convert::TryFrom;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub enum Signature {
    Unknown = 0,
    LocalFileHeader = 0x04034b50,
    CentralFileHeader = 0x02014b50,
    CentralDirEnd = 0x06054b50,
}

impl Default for Signature {
    fn default() -> Self {
        Signature::Unknown
    }
}

impl From<u32> for Signature {
    fn from(value: u32) -> Self {
        match value {
            0x04034b50 => Signature::LocalFileHeader,
            0x02014b50 => Signature::CentralFileHeader,
            0x06054b50 => Signature::CentralDirEnd,
            _ => Signature::Unknown,
        }
    }
}

impl From<[u8; 4]> for Signature {
    fn from(value: [u8; 4]) -> Self {
        u32::from_le_bytes([value[0], value[1], value[2], value[3]]).into()
    }
}

impl TryFrom<&[u8]> for Signature {
    type Error = Error;

    fn try_from(value: &[u8]) -> Result<Self> {
        if value.len() < 4 {
            return Err(Error::InvalidSignature);
        }
        Ok([value[0], value[1], value[2], value[3]].into())
    }
}
