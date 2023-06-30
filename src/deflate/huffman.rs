use super::lz77::Symbol;
use bitstream_io::{BitReader, BitWriter, BitRead, BitWrite, Endianness, LittleEndian, BitRecorder};
use std::{io::Read, io::Write};
use std::collections::HashMap;
use std::error::Error;
use crate::error::*;


struct HuffmanTree<T> {
    character: Option<T>, // 0-285
    frequency: usize,
    left: Option<Box<HuffmanTree<T>>>,
    right: Option<Box<HuffmanTree<T>>>
}


impl<T> HuffmanTree<T> {
    //  Create a new huffman tree node with no children
    pub fn new(character: Option<T>, frequency: usize) -> Self {
        HuffmanTree {
            character,
            frequency,
            left: None,
            right: None,
        }
    }

    pub fn is_leaf(&self) -> bool {
        self.left.is_none() && self.right.is_none()
    }

    // Build a huffman tree using given freqencies.
    // Return the root node, or None if the freq map is empty
    pub fn build_from_freq(frequencies: &HashMap<T, usize>) -> Option<Self> {
        let mut nodes: Vec<Box<HuffmanTree<T>>> = frequencies
            .iter()
            .map(|(char, freq)| Box::new(HuffmanTree::new(Some(*char), *freq)))
            .collect();
    
        while nodes.len() > 1 {
            nodes.sort_by(|a, b| b.frequency.cmp(&a.frequency)); // Sort nodes by frequency in descending order
    
            let smallest = nodes.pop().unwrap();
            let second_smallest = nodes.pop().unwrap();
    
            let combined_frequency = smallest.frequency + second_smallest.frequency;
            let combined_node = HuffmanTree {
                character: None,
                frequency: combined_frequency,
                left: Some(smallest),
                right: Some(second_smallest),
            };
    
            nodes.push(Box::new(combined_node));
        }
    
        match nodes.pop() {
            None => None,
            Some(b) => Some(*b)
        }
    }

    // extracts ONE character from the bitstream and decode it using the huffman tree
    pub fn decode_char<R: Read>(&self, bitreader: &mut BitReader<R, LittleEndian>) -> Result<T, Box<dyn Error>> {
        let mut cur = self;

        while !cur.is_leaf() {
            match bitreader.read_bit()? {
                false => if let Some(left) = &cur.left {
                    cur = left;
                } else {
                    return Err(Box::new(DecodeError::from("failed when decoding huffman: stopped at non-leaf node")));
                },
                true => if let Some(right) = &cur.right {
                    cur = right;
                } else {
                    return Err(Box::new(DecodeError::from("failed when decoding huffman: stopped at non-leaf node")));
                }
            }
        }

        if let Some(character) = cur.character {
            Ok(character)
        } else {
            Err(Box::new(DecodeError::from("failed when decoding huffman: empty leaf node")))
        }
    } 
}


#[derive(Debug, Clone, Copy)]
enum BType {
    Uncompressed = 0b00,
    Fixed        = 0b01,
    Dynamic      = 0b10
}

// Compress the block using fixed huffman codes.
// The input block does not include EndOfBlock.
// This function isn't aware of whether the block is the last block or not, so it leaves BFINAL as 0. The caller must deal with it.
fn huffman_encode_block(block: &[Symbol]) -> Result<(Vec<u8>, (u32, u8)), Box<dyn Error>> {
    let fixed = fixed_huffman_encode_block(block)?;
    let dynamic = dynamic_huffman_encode_block(block)?;

    if dynamic.0.len() < fixed.0.len() {
        Ok(dynamic)
    } else {
        Ok(fixed)
    }

}

// Compress the block using fixed huffman codes.
// Return the compressed block, including the header, the compressed data and the end of block symbol
fn fixed_huffman_encode_block(block: &[Symbol]) -> Result<(Vec<u8>, (u32, u8)), Box<dyn Error>> {
    let mut buf = Vec::new();
    let mut writer = BitWriter::endian(&mut buf, LittleEndian);

    writer.write(3, 0b100)?; // BFINAL = 0, BTYPE = 01

    // TODO: encode using fixed huffman code

    Ok((buf, writer.into_unwritten()))
}

// Compress the block using dynamic huffman codes.
// Return the compressed block, including the header, the 2 huffman trees, the compressed data and the end of block symbol
fn dynamic_huffman_encode_block(block: &[Symbol]) -> Result<(Vec<u8>, (u32, u8)), Box<dyn Error>> {
    let mut buf = Vec::new();
    let mut writer = BitWriter::endian(&mut buf, LittleEndian);

    writer.write(3, 0b010)?; // BFINAL = 0, BTYPE = 10

    // TODO: encode using dynamic huffman code

    Ok((buf, writer.into_unwritten()))
}


fn huffman_decode(input: &[u8]) -> Vec<Symbol> {
    unimplemented!();
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_huffman_tree() {
        unimplemented!();
    }
}