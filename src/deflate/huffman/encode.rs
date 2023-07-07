use bitstream_io::{BitWriter, BitWrite, LittleEndian};
use std::error::Error;
use crate::deflate::lz77::Symbol;
use super::{HuffmanCodes};
use super::{MAX_BITS, FIXED_LITERAL_CODES, LENGTH_REPR, DIST_REPR};
use super::limited_codelens_from_freq;

/// Compress the block using huffman codes.
/// The input block does not include EndOfBlock.
/// 
/// This function isn't aware of whether the block is the last block or not, so it leaves BFINAL bit as 0. The caller must deal with it.
/// 
/// The second part `(bits, value)` of the returned tuple exists since the encoded block may not be byte-aligned.
/// For example, if the block is ...01011010 01001, then `(bits, value) = (5, 0b01001)`
pub(crate) fn huffman_encode_block(block: &[Symbol]) -> Result<(Vec<u8>, (u32, u8)), Box<dyn Error>> {
    
    /*let fixed = fixed_huffman_encode_block(block)?;
    let dynamic = dynamic_huffman_encode_block(block)?;

    if dynamic.0.len() < fixed.0.len() {
        Ok(dynamic)
    } else {
        Ok(fixed)
    }*/

    fixed_huffman_encode_block(block)
}

/// Compress the block using fixed huffman codes.
/// Return the compressed block, including the header, the compressed data and the end of block symbol
fn fixed_huffman_encode_block(block: &[Symbol]) -> Result<(Vec<u8>, (u32, u8)), Box<dyn Error>> {
    let mut buf = Vec::new();
    let mut writer = BitWriter::endian(&mut buf, LittleEndian);

    writer.write(3, 0b010)?; // The header. BFINAL = 0, BTYPE = 01

    // Encode using fixed huffman code
    for symbol in block {
        match *symbol {
            Symbol::Literal(lit) => {
                FIXED_LITERAL_CODES.encode_char(&mut writer, lit as u16)?;
            },
            Symbol::Pointer {length, distance} => {
                assert!(distance >= 1 && distance <= 32768);

                let (code, bits, extra) = LENGTH_REPR[length as usize + 3];
                FIXED_LITERAL_CODES.encode_char(&mut writer, code)?;
                writer.write(bits as u32, extra)?;

                let (code, bits, extra) = DIST_REPR[distance as usize];
                writer.write(5, code)?;
                writer.write(bits as u32, extra)?;
            }
        }
    }

    // Write the end of block symbol
    FIXED_LITERAL_CODES.encode_char(&mut writer, 256)?;

    let last_byte = writer.into_unwritten();
    Ok((buf, last_byte))
}

/// Compress the block using dynamic huffman codes, Return the compressed block.
/// The structure of a dynamic huffman compressed block (See RFC 1951, Section 3.2.7): 
/// - 3 bits: the header
/// - 5 bits: HLIT, number of Literal/Length codes - 257
/// - 5 bits: HDIST, number of Distance codes - 1
/// - 4 bits: HCLEN, number of Code Length codes - 4
/// - (HCLEN + 4) * 3 bits: the code lengths for the code length alphabet, in the order: 0,8,7,9,6,10,5,11,4,12,3,13,2,14,1,15. Zeros at the end are discarded.
/// - HLIT + 257 code lengths for the literal/length alphabet, encoded using the code length alphabet
/// - HDIST + 1 code lengths for the distance alphabet, encoded using the code length alphabet
/// - the compressed data
/// - the end of block symbol
fn dynamic_huffman_encode_block(block: &[Symbol]) -> Result<(Vec<u8>, (u32, u8)), Box<dyn Error>> {
    let mut buf = Vec::new();
    let mut writer = BitWriter::endian(&mut buf, LittleEndian);

    // write header
    writer.write(3, 0b100)?; // BFINAL = 0, BTYPE = 10

    // TODO: encode using dynamic huffman code

    let mut literal_freqencies = [0_u32; 286];
    let mut distance_freqencies = [0_u32; 30];

    // Count freqencies of literals/lengths and distances
    for symbol in block {
        match *symbol {
            Symbol::Literal(lit) => {
                literal_freqencies[lit as usize] += 1;
            },
            Symbol::Pointer {length, distance} => {
                assert!(distance >= 1 && distance <= 32768);
                literal_freqencies[LENGTH_REPR[length as usize].0 as usize] += 1;
                distance_freqencies[DIST_REPR[distance as usize].0 as usize] += 1;
            }
        }
    }

    let literal_codelens = {
        let mut codelens = limited_codelens_from_freq(&literal_freqencies, MAX_BITS);
        // remove zeros at the end
        while codelens[codelens.len() - 1] == 0 {
            codelens.pop();
        }
        codelens
    };

    let distance_codelens = {
        let mut codelens = limited_codelens_from_freq(&distance_freqencies, MAX_BITS);
        // remove zeros at the end
        while codelens[codelens.len() - 1] == 0 {
            codelens.pop();
        }
        codelens
    };

    // Write HLIT, HDIST
    let HLIT = literal_codelens.len() as u32 - 257;
    let HDIST = distance_codelens.len() as u32 - 1;
    writer.write(5, HLIT)?;
    writer.write(5, HDIST)?;

    // Write HCLEN and the code lengths for the code length alphabet, literal/length alphabet and distance alphabet
    //encode_codelens(&mut writer, &literal_codelens, &distance_codelens);

    let literal_codes = HuffmanCodes::build_from_codelens(&literal_codelens)?;
    let distance_codes = HuffmanCodes::build_from_codelens(&distance_codelens)?;

    // Encode block data using dynamic huffman codes
    for symbol in block {
        match *symbol {
            Symbol::Literal(lit) => {
                literal_codes.encode_char(&mut writer, lit as u16)?;
            },
            Symbol::Pointer {length, distance} => {
                assert!(distance >= 1 && distance <= 32768);

                let (code, bits, extra) = LENGTH_REPR[length as usize];
                literal_codes.encode_char(&mut writer, code)?;
                writer.write(bits as u32, extra)?;

                let (code, bits, extra) = DIST_REPR[distance as usize];
                distance_codes.encode_char(&mut writer, code as u16)?;
                writer.write(bits as u32, extra)?;
            }
        }
    }

    // Write the end of block symbol
    literal_codes.encode_char(&mut writer, 256)?;

    let last_byte = writer.into_unwritten();
    Ok((buf, last_byte))
}


/*

/// The function first calulates code lengths for the code length alphabet. 
/// Then it writes HCLEN and the code lengths for the code length alphabet.
/// Finally it writes the code lengths for the literal/length alphabet and the distance alphabet, which are encoded using the code length alphabet. 

fn encode_codelens<W: Write>(writer: &mut BitWriter<W, LittleEndian>, lit_codelens: &[u32], dist_codelens: &[u32]) -> Result<(), EncodeError> {
    if codelens.len() == 0 {
        return Err(EncodeError::from("encode_bitlens: empty bitlens"));
    }

    let mut len = codelens[0];
    let count = 1;

    for bits in &codelens[1..] {
        if bits > MAX_BITS {
            return Err(EncodeError::from("encode_bitlens: bit length > MAX_BITS"));
        }


        if *bits == len {
            count += 1;
            continue;
        } else if count > 0{
            // write into the bitstream
            
        }
    }

}

 */

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fixed_huffman() {
        println!("{:?}",LENGTH_REPR[7]);
        println!("{:?}", DIST_REPR[14]);

        let mut buf = Vec::new();
        let mut writer = BitWriter::endian(&mut buf, LittleEndian);
        let (code, bits, extra) = LENGTH_REPR[7];
        FIXED_LITERAL_CODES.encode_char(&mut writer, code).unwrap();
        writer.write(bits as u32, extra).unwrap();

        let (code, bits, extra) = DIST_REPR[14];
        writer.write(5, code).unwrap();
        writer.write(bits as u32, extra).unwrap();
        

        let a = writer.into_unwritten();
        println!("{:?} {:?}", buf, a);
    }
}