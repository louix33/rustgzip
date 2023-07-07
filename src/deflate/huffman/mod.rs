pub(crate) mod encode;


use core::panic;
use std::collections::HashMap;
use std::{io::Read, io::Write};
use bitstream_io::{BitReader, BitWriter, BitRead, BitWrite, LittleEndian};
use lazy_static::lazy_static;
use std::error::Error;
use crate::error::*;


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
    
    static ref FIXED_LITERAL_CODES: HuffmanCodes = HuffmanCodes::build_from_codelens(&*FIXED_HUFFMAN_BITS).unwrap();

    static ref FIXED_LITERAL_TREE: HuffmanTree = HuffmanTree::build_from_codes(&*FIXED_LITERAL_CODES).unwrap();

    /// [(code, bits, extra_bits), ...)]
    static ref LENGTH_REPR: [(u16, u8, u8); 259] = {
        let mut repr = [(0, 0, 0); 259];
        for i in 3..11 {
            repr[i] = (i as u16 + 254, 0, 0);
        }

        let mut base = 11;

        for code in 265..285 {
            let bits = (code - 265) / 4 + 1;

            for extra in 0..(1 << bits) {
                let j = base + extra;
                repr[j] = (code, bits as u8, extra as u8);
            }
            base += 1 << bits;
        }

        repr[258] = (285, 0, 0);
        repr
    };

    static ref DIST_REPR: [(u8, u8, u16); 32769] = {
        let mut repr: [(u8, u8, u16); 32769]= [(0, 0, 0); 32769];
        for i in 1..5 {
            let j = i as u8 - 1;
            repr[i] = (j + 0, 0, 0);
        }

        let mut base = 5;
        
        for code in 4..30 {
            let bits = (code - 4) / 2 + 1;
            let rcode = reverse_bits(code, 5);

            for extra in 0..(1 << bits) {
                let j = base + extra;
                repr[j] = (rcode, bits as u8, extra as u16);
            }
            base += 1 << bits;
        }

        repr
    };
}

fn limited_codelens_from_freq(frequencies: &[u32], max_bits: u32) -> Vec<u32> {
    unimplemented!();
}

fn reverse_bits(bits: u8, num: u32) -> u8 {
    assert!(num <= 8);
    let mut reversed = 0;
    for i in 0..num {
        reversed |= (bits & (1 << i)) >> i << (num - 1 - i);
    }
    reversed
}


struct HuffmanCodes {
    pub dict: HashMap<u16, Vec<bool>>
}

impl HuffmanCodes {
    pub fn build_from_tree(tree: &HuffmanTree) -> Self {
        unimplemented!()
    }

    pub fn build_from_codelens(codelens: &[u32]) -> Result<Self, DecodeError> {
        // Step 1: Count the number of codes for each code length
        let mut bl_count = [0_u32; MAX_BITS as usize + 1]; // A huffman code is 15 bits long at most.
        for &bits in codelens {
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
        let mut code_dict = HuffmanCodes {
            dict: HashMap::new()
        };
        for (symbol, &bits) in codelens.iter().enumerate() {
            if bits != 0 {
                code = next_code[bits as usize];
                next_code[bits as usize] += 1;
                code_dict.set_code(symbol as u16, (bits, code));
            }
        }

        Ok(code_dict)
    }

    /// Encode and insert the given character into the bitstream
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
    /// 
    /// Insert (or update) the huffman code for given character.
    pub fn set_code(&mut self, character: u16, code: (u32, u16)) {
        if code.0 > MAX_BITS {
            panic!("huffman code longer than MAX_BITS");
        }

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
    

    /// Build a canonical, length-limited huffman tree using given symbol freqencies.
    /// Assume that the symbols in the alphabet begin from 0 and grow consecutively.
    /// Return the root node, or None if the any error occurs
    pub fn build_canonical_from_freq(frequencies: &[u32]) -> Result<Self, DecodeError> {
        let mut codelens = limited_codelens_from_freq(frequencies, MAX_BITS);

        Self::build_canonical_from_codelens(&codelens)
    }

    /// Build a canonical huffman tree using given bit lengths. The algorithm is described in RFC 1951, Section 3.2.2.
    /// Assume that the symbols in the alphabet begin from 0 and grow consecutively.
    /// For example, `bitlen[3] == 2` means that the symbol 3 is encoded using 2 bits. 
    pub fn build_canonical_from_codelens(codelens: &[u32]) -> Result<Self, DecodeError> {
        Self::build_from_codes(&HuffmanCodes::build_from_codelens(codelens)?)
    }

    /// Build a huffman tree from given huffman codes.
    /// Return None if the codes cannot generate a legal huffman tree
    fn build_from_codes(codes: &HuffmanCodes) -> Result<Self, DecodeError> {
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
    pub fn decode_char<R: Read>(&self, reader: &mut BitReader<R, LittleEndian>) -> Result<u16, Box<dyn Error>> {
        let mut cur = self;

        while !cur.is_leaf() {
            match reader.read_bit()? {
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


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_unwritten() {
        let mut buf = Vec::new();
        let mut writer = BitWriter::endian(&mut buf, LittleEndian);
 
        writer.write(3, 0b100).unwrap(); // BFINAL = 0, BTYPE = 10
        writer.write_bit(false).unwrap();
        writer.write_bit(false).unwrap();
        writer.write_bit(true).unwrap();

        assert_eq!(writer.into_unwritten(), (6_u32, 0b100100_u8));
    }

    #[test]
    fn test_huffman_from_bitlen() {
        println!("{:?}", HuffmanCodes::build_from_codelens(&[2,1,3,3]).unwrap().dict);
        println!("{:?}", HuffmanCodes::build_from_codelens(&[3,3,3,3,3,2,4,4]).unwrap().dict);
    }

    #[test]
    fn test_length_repr() {
        println!("{:?}", LENGTH_REPR.iter().enumerate().collect::<Vec<_>>());
    }

    #[test]
    fn test_dist_repr() {
        println!("{:?}", DIST_REPR[8191]);
    }

    #[test]
    fn test_reverse_bits() {
        assert_eq!(reverse_bits(0b0010_1001, 5), 0b0001_0010);
    }
    
}