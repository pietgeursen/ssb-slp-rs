//! ssb-slp-encoding
//!
//! A module that implements the Shallow Length Prefixed (SLP) encoding of collections of bytes
//! used by the scuttlebutt envelope-spec.
//!
//! Spec defined [here](https://github.com/ssbc/envelope-spec/blob/master/encoding/slp.md)
//!

use bytes::{BufMut, Buf};
use snafu::{ResultExt, Snafu};
use std::convert::TryInto;
use std::io::Read;
use std::io::Write;

#[derive(Debug, Snafu)]
pub enum Error {
    ItemTooLong { source: std::num::TryFromIntError },
    WriteError { source: std::io::Error },
    ReadError { source: std::io::Error },
}

#[derive(PartialEq, Debug)]
pub struct SLP(pub Vec<Vec<u8>>);

impl SLP {
    pub fn encode_write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        self.0.iter().try_for_each(|bytes| {
            let byte_len: u16 = bytes.len().try_into().context(ItemTooLong)?;
            let mut length_buf = [0u8;2];
            (&mut length_buf[..]).put_u16_le(byte_len);

            writer.write(&length_buf).context(WriteError)?;
            writer.write(&bytes).context(WriteError)?;

            Ok::<(), Error>(())
        })?;

        Ok(())
    }

    pub fn decode_read<R: Read>(reader: &mut R) -> Result<Self, Error>{
        let mut items = Vec::new();

        let mut length_buf = [0u8;2];

        while let Ok(_) = reader.read_exact(&mut length_buf){
            let byte_len = (&length_buf[..]).get_u16_le();
            let mut item = Vec::with_capacity(byte_len as usize);
            item.resize(byte_len as usize, 0);
            reader.read_exact(&mut item).context(ReadError)?;
            items.push(item);
        }
        Ok(SLP(items))
    }

    pub fn into_inner(self) -> Vec<Vec<u8>>{
        self.0
    }
}

#[cfg(test)]
mod tests {
    use crate::*;
    #[test]
    fn encode_decode() {
        let data = vec![vec![1u8,2,3], vec![4u8, 5, 6, 7]];
        let slp = SLP(data);

        let mut encode_writer = vec![];
        slp.encode_write(&mut encode_writer).unwrap();
        

        let decoded = SLP::decode_read(&mut encode_writer.as_slice()).unwrap();

        assert_eq!(decoded, slp);
    }
}
