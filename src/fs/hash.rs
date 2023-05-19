use std::io::Read;
use ring::digest::{Context, Digest, SHA256};

pub fn hash_reader<R: Read>(mut reader: R) -> std::io::Result<Digest> {
    let mut context = Context::new(&SHA256);
    let mut buffer = [0; 1024];
    loop {
        let count = reader.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        context.update(&buffer[..count]);
    }
    Ok(context.finish())
}