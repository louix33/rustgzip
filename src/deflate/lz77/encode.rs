use super::Symbol;
use std::error::Error;
use std::io::{BufRead, ErrorKind};
use super::Lz77Status;
use crate::circular_buf::CircularBuf;

/// Encode the input data using lz77 algorithm
pub(crate) fn lz77_encode_block<R: BufRead>(reader: &mut R, window: &mut CircularBuf<u8>, lookahead: &mut CircularBuf<u8>, blksize: usize) -> Result<Lz77Status, Box::<dyn Error>> {
    let mut output = Vec::new();

    let mut ended = false;
    let mut buf = [0u8; 1];

    loop {
        // Fill the lookahead buffer
        while !ended && !lookahead.is_full() {
            match reader.read_exact(&mut buf) {
                Ok(_) => lookahead.push_back(buf[0]),
                Err(err) => {
                    if err.kind() == ErrorKind::UnexpectedEof {
                        ended = true;
                        break;
                    } else {
                        return Err(Box::new(err));
                    }
                }
            };
        }

        if lookahead.is_empty() {
            break;
        }

        match longest_match(&window, &lookahead) {
            Some((length, distance)) => {
                // Found a match
                assert!(length >= 3 && length <= 258);
                output.push(Symbol::Pointer { length: (length - 3) as u8, distance: distance as u16 });
                for _ in 0..length {
                    window.push_back(lookahead.pop_front().unwrap());
                }
            },
            None => {
                // No match found
                let c = lookahead.pop_front().unwrap();
                output.push(Symbol::Literal(c));
                window.push_back(c);
            }
        }

        if output.len() == blksize {
            break;
        }
    }

    if lookahead.is_empty() {
        Ok(Lz77Status::LastBlock(output))
    } else {
        Ok(Lz77Status::Normal(output))
    }
}


/// A trival implementation of longest match, may be slow. 
/// To be optimized
fn longest_match(window: &CircularBuf<u8>, lookahead: &CircularBuf<u8>) -> Option<(usize, usize)> {
    let mut best_match = (0, 0); // (length, distance)

    for i in 0..window.len() {
        let mut match_length = 0;
        for j in 0..lookahead.len() {
            if i + j >= window.len() {
                break;
            } else if window[i + j] == lookahead[j] {
                match_length += 1;
            } else {
                break;
            }
        }
        if match_length >= 3 && match_length > best_match.0 {
            best_match = (match_length, window.len() - i);
        }
    }

    if best_match.0 >= 3 {
        Some(best_match)
    } else {
        None
    }
}


#[cfg(test)]
mod tests {
    use crate::deflate::lz77::{WINDOW_SIZE, LOOKAHEAD_SIZE};

    use super::*;
    use std::io::BufReader;

    #[test]
    fn test_longest_match() {
        let mut window = CircularBuf::with_capacity(WINDOW_SIZE);
        let mut lookahead = CircularBuf::with_capacity(LOOKAHEAD_SIZE);

        let l = "Hello, world!\nHello, Rust!\nRust is the best language!\n".as_bytes();

        for b in l {
            lookahead.push_back(*b);
        }

        for _ in 0..14 {
            window.push_back(lookahead.pop_front().unwrap());
        }

        assert_eq!(longest_match(&window, &lookahead), Some((7, 14)));
    }

    #[test]
    fn test_lz77_encode() {
        let input = Vec::from("Hello, world!\nHello, Rust!\nRust is the best language!\n");
        let mut reader = BufReader::new(input.as_slice());
        let mut window = CircularBuf::with_capacity(WINDOW_SIZE);
        let mut lookahead = CircularBuf::with_capacity(LOOKAHEAD_SIZE);

        let block = lz77_encode_block(&mut reader, &mut window, &mut lookahead, 65535)
            .unwrap();

        match block {
            Lz77Status::LastBlock(blk) => {
                for s in blk {
                    print!("{}, ", s);
                }
            }
            _ => panic!("Should be the last block")
        };
    }
}

