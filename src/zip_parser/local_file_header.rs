use super::{Error, Result, Signature};
use bytes::Buf;
use std::convert::TryFrom;

pub const LOCAL_FILE_HEADER_SIZE: usize = std::mem::size_of::<LocalFileHeader>();

#[repr(packed)]
#[derive(Debug, Copy, Clone)]
pub struct LocalFileHeader {
    pub signature: Signature,
    pub version_needed_to_extract: u16,
    pub general_purpose_bit_flag: u16,
    pub compression_method: u16,
    pub last_mod_file_time: u16,
    pub last_mod_file_date: u16,
    pub crc32: u32,
    pub compressed_size: u32,
    pub uncompressed_size: u32,
    pub file_name_length: u16,
    pub extra_field_length: u16,
}

impl TryFrom<&[u8]> for LocalFileHeader {
    type Error = Error;

    fn try_from(value: &[u8]) -> Result<Self> {
        if value.len() < LOCAL_FILE_HEADER_SIZE {
            return Err(Error::LocalFileHeaderParseError);
        }

        let mut buf = &value[..];
        let signature = buf.get_u32_le();

        if signature != Signature::LocalFileHeader as u32 {
            return Err(Error::InvalidSignature);
        }

        let version_needed_to_extract = buf.get_u16_le();
        let general_purpose_bit_flag = buf.get_u16_le();
        let compression_method = buf.get_u16_le();
        let last_mod_file_time = buf.get_u16_le();
        let last_mod_file_date = buf.get_u16_le();
        let crc32 = buf.get_u32_le();
        let compressed_size = buf.get_u32_le();
        let uncompressed_size = buf.get_u32_le();
        let file_name_length = buf.get_u16_le();
        let extra_field_length = buf.get_u16_le();

        Ok(Self {
            signature: signature.into(),
            version_needed_to_extract,
            general_purpose_bit_flag,
            compression_method,
            last_mod_file_time,
            last_mod_file_date,
            crc32,
            compressed_size,
            uncompressed_size,
            file_name_length,
            extra_field_length,
        })
    }
}
