use std::collections::HashMap;
use std::{io::Read, io::Write};
use bitstream_io::{BitReader, BitWriter, BitRead, BitWrite, LittleEndian};
use lazy_static::lazy_static;
use std::error::Error;
use crate::error::*;
use super::lz77::Symbol;


/// The maximum number of bits in a huffman code
const MAX_BITS: u32 = 15;


lazy_static! {
    static ref FIXED_HUFFMAN_BITS: [u32; 288] = {
        let mut bits = [0; 288];
        for i in 0..144 {
            bits[i] = 8;
        }
        for i in 144..256 {
            bits[i] = 9;
        }
        for i in 256..280 {
            bits[i] = 7;
        }
        for i in 280..288 {
            bits[i] = 8;
        }
        bits
    };

    static ref FIXED_LITERAL_TREE: HuffmanTree = HuffmanTree::build_canonical_from_bitlen(&*FIXED_HUFFMAN_BITS).unwrap();
}


struct HuffmanDict {
    pub dict: HashMap<u16, Vec<bool>>
}

impl HuffmanDict {
    pub fn build_from_tree(tree: &HuffmanTree) -> Self {
        unimplemented!()
    }

    pub fn build_from_bitlen(bitlen: &[u32]) -> Result<Self, DecodeError> {
        // Step 1: Count the number of codes for each code length
        let mut bl_count = [0_u32; MAX_BITS as usize + 1]; // A huffman code is 15 bits long at most.
        for &bits in bitlen {
            if bits > MAX_BITS { return Err(DecodeError::from("")); }
            bl_count[bits as usize] += 1;
        }

        // Step 2: Find the numerical value for the smallest code for each code length
        bl_count[0] = 0;
        let mut next_code = [0; MAX_BITS as usize + 1];
        let mut code = 0_u16;
        for bits in 0..MAX_BITS as usize {
            code = (code + bl_count[bits] as u16) << 1;
            next_code[bits + 1] = code;
        }

        // Step 3: Assign numerical values to all codes, using consecutive values for all codes of the same length with the base values determined at step 2
        let mut code_dict = HuffmanDict {
            dict: HashMap::new()
        };
        for (symbol, &bits) in bitlen.iter().enumerate() {
            if bits != 0 {
                code = next_code[bits as usize];
                next_code[bits as usize] += 1;
                code_dict.set_code(symbol as u16, (bits, code));
            }
        }

        Ok(code_dict)
    }

    /// Encode the given character into the bitstream
    pub fn encode_char<W: Write>(&self, writer: &mut BitWriter<W, LittleEndian>, character: u16) -> Result<(), Box<dyn Error>> {
        match self.dict.get(&character) {
            Some(code) => {
                for bit in code {
                    writer.write_bit(*bit)?;
                }
                Ok(())
            },
            None => Err(Box::new(EncodeError::from("failed when encoding huffman: character not found in alphabet")))
        }
    }

    /// code: (bits, value)
    pub fn set_code(&mut self, character: u16, code: (u32, u16)) {
        let mut codevec = Vec::new();
        for i in (0..code.0).rev() {
            codevec.push((code.1 & (1 << i)) != 0);
        }
        self.dict.insert(character, codevec);
    }
}

struct HuffmanTree {
    character: Option<u16>, // 0-285
    left: Option<Box<HuffmanTree>>,
    right: Option<Box<HuffmanTree>>
}

impl HuffmanTree {
    /// Create a new huffman tree node with no children
    pub fn new(character: Option<u16>) -> Self {
        HuffmanTree {
            character,
            left: None,
            right: None,
        }
    }

    pub fn is_leaf(&self) -> bool {
        self.left.is_none() && self.right.is_none()
    }

    /// Build a huffman tree using given freqencies, NOT guaranteeing that the tree is canonical.
    /// Assume that the symbols in the alphabet begin from 0 and grow consecutively.
    /// Return the root node, or None if the frequency array is empty
    /*
    pub fn build_from_freq(frequencies: &[u32]) -> Option<Self> {
        let mut nodes: Vec<HuffmanTree> = frequencies
            .iter().enumerate()
            .filter(|(_, freq)| **freq != 0)
            .map(|(char, freq)| HuffmanTree::new(Some(char as u16), *freq))
            .collect();
    
        while nodes.len() > 1 {
            nodes.sort_by(|a, b| b.frequency.cmp(&a.frequency)); // Sort nodes by frequency in descending order
            let smallest = nodes.pop().unwrap();
            let second_smallest = nodes.pop().unwrap();

            let combined_node = HuffmanTree {
                character: None,
                frequency: smallest.frequency + second_smallest.frequency,
                left: Some(Box::new(smallest)),
                right: Some(Box::new(second_smallest)),
            };
    
            nodes.push(combined_node);
        }
    
        nodes.pop()
    }
    */
    

    /// Build a canonical, length-limited huffman tree using given symbol freqencies.
    /// Assume that the symbols in the alphabet begin from 0 and grow consecutively.
    /// Return the root node, or None if the any error occurs
    pub fn build_canonical_from_freq(frequencies: &[u32]) -> Result<Self, DecodeError> {
        let mut bitlen = vec![0_u32; frequencies.len()];

        // decide the code length for each symbol using package-merge algorithm
        let mut packages: Vec<(u16, u32)> = frequencies
            .iter().enumerate()
            .filter(|(_, freq)| **freq > 0)
            .map(|(char, freq)| (char as u16, *freq))
            .collect();
        packages.sort_by(|a, b| b.1.cmp(&a.1)); // sort packages by frequency in descending order

        unimplemented!();

        Self::build_canonical_from_bitlen(&bitlen)
    }

    /// Build a canonical huffman tree using given bit lengths. The algorithm is described in RFC 1951, Section 3.2.2.
    /// Assume that the symbols in the alphabet begin from 0 and grow consecutively.
    /// For example, `bitlen[3] == 2` means that the symbol 3 is encoded using 2 bits. 
    pub fn build_canonical_from_bitlen(bitlen: &[u32]) -> Result<Self, DecodeError> {
        Self::build_from_codes(&HuffmanDict::build_from_bitlen(bitlen)?)
    }

    /// Build a huffman tree from given huffman codes.
    /// Return None if the codes cannot generate a legal huffman tree
    fn build_from_codes(codes: &HuffmanDict) -> Result<Self, DecodeError> {
        let mut tree = HuffmanTree::new(None);
        for (symbol, code) in &codes.dict {
            let mut cur = &mut tree;

            for bit in code {
                match bit {
                    false => { // go to the left
                        if cur.left.is_none() {
                            cur.left = Some(Box::new(HuffmanTree::new(None)));
                        }
                        cur = cur.left.as_mut().unwrap();
                    },
                    true => { // go to the right
                        if cur.right.is_none() {
                            cur.right = Some(Box::new(HuffmanTree::new(None)));
                        }
                        cur = cur.right.as_mut().unwrap();
                    }
                }
            }

            if !cur.is_leaf() {
                return Err(DecodeError::from("build_from_codes: duplicated prefix"));
            } else {
                cur.character = Some(*symbol);
            }
            
        }

        Ok(tree)
    }

    /// extracts ONE character from the bitstream and decode it using the huffman tree
    pub fn decode_char<R: Read>(&self, bitreader: &mut BitReader<R, LittleEndian>) -> Result<u16, Box<dyn Error>> {
        let mut cur = self;

        while !cur.is_leaf() {
            match bitreader.read_bit()? {
                false => if let Some(ref left) = cur.left {
                    cur = left;
                } else {
                    return Err(Box::new(DecodeError::from("failed when decoding huffman: stopped at non-leaf node")));
                },
                true => if let Some(ref right) = cur.right {
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


/// Compress the block using huffman codes.
/// The input block does not include EndOfBlock.
/// 
/// This function isn't aware of whether the block is the last block or not, so it leaves BFINAL bit as 0. The caller must deal with it.
/// 
/// The second part `(bits, value)` of the returned tuple exists since the encoded block may not be byte-aligned.
/// For example, if the block is ...01011010 01001, then `(bits, value) = (5, 0b01001)`
fn huffman_encode_block(block: &[Symbol]) -> Result<(Vec<u8>, (u32, u8)), Box<dyn Error>> {
    let fixed = fixed_huffman_encode_block(block)?;
    let dynamic = dynamic_huffman_encode_block(block)?;

    if dynamic.0.len() < fixed.0.len() {
        Ok(dynamic)
    } else {
        Ok(fixed)
    }

}

/// Compress the block using fixed huffman codes.
/// Return the compressed block, including the header, the compressed data and the end of block symbol
fn fixed_huffman_encode_block(block: &[Symbol]) -> Result<(Vec<u8>, (u32, u8)), Box<dyn Error>> {
    let mut buf = Vec::new();
    let mut writer = BitWriter::endian(&mut buf, LittleEndian);

    writer.write(3, 0b100)?; // BFINAL = 0, BTYPE = 01

    // TODO: encode using fixed huffman code
    

    let last_byte = writer.into_unwritten();
    Ok((buf, last_byte))
}

/// Compress the block using dynamic huffman codes.
/// Return the compressed block, including the header, the 2 huffman trees, the compressed data and the end of block symbol
fn dynamic_huffman_encode_block(block: &[Symbol]) -> Result<(Vec<u8>, (u32, u8)), Box<dyn Error>> {
    let mut buf = Vec::new();
    let mut writer = BitWriter::endian(&mut buf, LittleEndian);

    writer.write(3, 0b010)?; // BFINAL = 0, BTYPE = 10

    // TODO: encode using dynamic huffman code

    let last_byte = writer.into_unwritten();
    Ok((buf, last_byte))
}


fn huffman_decode(input: &[u8]) -> Vec<Symbol> {
    unimplemented!();
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_written() {
        let mut buf = Vec::new();
        let mut writer = BitWriter::endian(&mut buf, LittleEndian);
 
        writer.write(3, 0b100); // BFINAL = 0, BTYPE = 10

        assert_eq!(writer.into_unwritten(), (3_u32, 0b100_u8));
    }
    
}