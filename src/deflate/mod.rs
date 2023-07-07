mod lz77;
mod huffman;

use core::panic;
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};

use bitstream_io::{BitWriter, BitWrite, LittleEndian};

use lz77::{Symbol, Lz77Status, WINDOW_SIZE, LOOKAHEAD_SIZE};
use lz77::encode::lz77_encode_block;
use huffman::encode::huffman_encode_block;
use crate::circular_buf::CircularBuf;

const BLKSIZE: usize = 65535;

pub fn deflate(src: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut dst = Vec::new();
    if src.is_empty() {
        return Ok(dst);
    }

    let mut writer = BitWriter::endian(&mut dst, LittleEndian);
    let mut reader = BufReader::new(src);

    let mut blocks = Vec::new();
    let mut window = CircularBuf::with_capacity(WINDOW_SIZE);
    let mut lookahead = CircularBuf::with_capacity(LOOKAHEAD_SIZE);


    loop {
        match lz77_encode_block(&mut reader, &mut window, &mut lookahead, BLKSIZE)? {
            Lz77Status::Normal(block) => {
                blocks.push(huffman_encode_block(&block)?);
            }
            Lz77Status::LastBlock(block) => {
                if block.is_empty() {
                    if blocks.is_empty() {
                        panic!("lz77 output is empty"); // should not happen
                    } else {
                        // the last block should have BFINAL
                        let len = blocks.len();
                        set_bfinal(&mut blocks[len - 1]);
                    }
                } else {
                    let mut last = huffman_encode_block(&block)?;
                    set_bfinal(&mut last);
                    blocks.push(last);
                }
                break;
            }
        };
    }

    // write the blocks
    for (data, (bits, value)) in blocks {
        writer.write_bytes(&data)?;
        writer.write(bits, value)?;
    }

    // write the last byte
    writer.byte_align()?;

    Ok(dst)

}

fn set_bfinal(block: &mut (Vec<u8>, (u32, u8))) {
    if block.0.is_empty() {
        block.1.1 |= 1;
    } else {
        block.0[0] |= 0x01;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deflate() {
        let raw = Vec::from("Hello, world, world");
        let dst: Vec<u8> = deflate(raw.as_slice()).unwrap();

        println!("{:?}", dst);
    }

    #[test]
    fn test_all() {
        let raw = "Hello, world!\nHello, Rust!\nRust is the best language!\n";
        let dst: Vec<u8> = deflate(raw.as_bytes()).unwrap();
        println!("{:?}", dst);
    }
}


