use std::collections::HashMap;
use std::io;

use file::File;
use object::{Object, PrimitiveConv};
use primitive::Primitive;
use types::StreamFilter;

#[derive(Object, PrimitiveConv)]
struct StreamInfo {
    #[pdf(key = "Filter")]
    filter: Vec<StreamFilter>,
    
    #[pdf(key = "Type")]
    ty:     String
}

pub struct Stream {
    data:       Vec<u8>,
    info:       StreamInfo
}
impl Stream {
    pub fn from_file(p: &Primitive, data: &[u8]) -> Self {
        Ok(Stream {
            info:   StreamInfo::from_primitive(p),
            data:   data
        })
    }

    pub fn empty(ty: &str) -> Stream {
        Stream {
            data:   Vec::new(),
            info:   StreamInfo {
                filter: vec![],
                ty:     ty
            }
        }
    }
}
impl Object for Stream {
    fn serialize<W: io::Write>(&self, out: &mut W) -> io::Result<()> {
        self.info.serialize(out)?;
        
        out.write(b"stream\n")?;
        out.write(&self.data)?;
        out.write(b"\nendstream\n")?;
        Ok(())
    }
}



// TODO move out to decoding/encoding module
fn flat_decode(data: &[u8]) -> Vec<u8> {
    let mut inflater = InflateStream::from_zlib();
    let mut out = Vec::<u8>::new();
    let mut n = 0;
    while n < data.len() {
        let res = inflater.update(&data[n..]);
        if let Ok((num_bytes_read, result)) = res {
            n += num_bytes_read;
            out.extend(result);
        } else {
            res.unwrap();
        }
    }
    out
}