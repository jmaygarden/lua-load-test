use super::local_file_header::LOCAL_FILE_HEADER_SIZE;
use super::{Error, LocalFileHeader, Result};
use std::convert::TryFrom;
use std::io::{Read, Seek, SeekFrom};

#[derive(Debug)]
pub struct LocalFile<S>
where
    S: Read + Seek,
{
    reader: S,
    pos: u64,
    header: LocalFileHeader,
    file_name: String,
}

impl<S> LocalFile<S>
where
    S: Read + Seek,
{
    pub fn new(mut reader: S, pos: u64, header: LocalFileHeader) -> Result<Self> {
        // extract local file name
        let file_name_length = header.file_name_length as usize;
        let mut file_name_buffer = vec![0u8; file_name_length];
        reader.read_exact(&mut file_name_buffer[..])?;
        let file_name = String::from_utf8(file_name_buffer)?;

        // seek past extra field
        reader.seek(SeekFrom::Current(header.extra_field_length as i64))?;

        Ok(Self {
            reader,
            pos,
            header,
            file_name,
        })
    }

    pub fn file_name(&self) -> String {
        self.file_name.clone()
    }

    pub fn skip(mut self) -> Result<(u64, S)> {
        let pos = self
            .reader
            .seek(SeekFrom::Current(self.header.compressed_size as i64))?;

        Ok((pos, self.reader))
    }

    #[allow(dead_code)]
    pub fn extract_compressed(self) -> Result<(S, Vec<u8>)> {
        let Self {
            mut reader,
            pos,
            header,
            ..
        } = self;
        let _ = reader.seek(SeekFrom::Start(pos))?;
        let len = LOCAL_FILE_HEADER_SIZE
            + header.compressed_size as usize
            + header.file_name_length as usize
            + header.extra_field_length as usize;
        let mut buf = vec![0u8; len];

        reader.read_exact(&mut buf[..])?;

        Ok((reader, buf))
    }

    pub fn extract_uncompressed(self) -> Result<(S, Vec<u8>)> {
        let Self {
            mut reader,
            pos,
            header,
            ..
        } = self;
        let _ = reader.seek(SeekFrom::Start(
            pos + LOCAL_FILE_HEADER_SIZE as u64
                + header.file_name_length as u64
                + header.extra_field_length as u64,
        ))?;

        let mut input = vec![0u8; header.compressed_size as usize];
        reader.read_exact(&mut input[..])?;

        let mut output = vec![0u8; header.uncompressed_size as usize];
        flate2::Decompress::new(false).decompress(
            &input[..],
            &mut output[..],
            flate2::FlushDecompress::Finish,
        )?;

        Ok((reader, output))
    }
}

pub fn find_local_file<S: Read + Seek>(mut reader: S, file_name: &str) -> Result<LocalFile<S>> {
    let file_len = reader.seek(SeekFrom::End(0))?;
    let mut pos = reader.seek(SeekFrom::Start(0))?;
    let mut header_buffer = [0u8; LOCAL_FILE_HEADER_SIZE];

    loop {
        if file_len - pos < LOCAL_FILE_HEADER_SIZE as u64 {
            return Err(Error::LocalFileNotFound);
        }

        reader.read_exact(&mut header_buffer)?;

        if let Ok(header) = LocalFileHeader::try_from(&header_buffer[..]) {
            let local_file = LocalFile::new(reader, pos, header)?;

            if file_name == local_file.file_name() {
                return Ok(local_file);
            } else {
                (pos, reader) = local_file.skip()?;
            }
        } else {
            break;
        }
    }

    Err(Error::LocalFileNotFound)
}
