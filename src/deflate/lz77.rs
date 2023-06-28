use std::collections::HashMap;
use std::io::{Read, Write};
use super::Symbol;

const WINDOW_SIZE: usize = 32768;
const LOOKAHEAD_SIZE: usize = 258;

// TODO: implement lazy matching
fn lz77_lazy_encode(input: &[u8]) -> Vec<Symbol> {
    let mut output = Vec::new();
    // let mut hashmap = HashMap::new();

    let mut lookahead_start: usize = 0;

    while lookahead_start < input.len() {
        let window_start = if lookahead_start > WINDOW_SIZE {
            lookahead_start - WINDOW_SIZE
        } else {
            0
        };
        let lookahead_end = if lookahead_start + LOOKAHEAD_SIZE > input.len() {
            input.len()
        } else {
            lookahead_start + LOOKAHEAD_SIZE
        };

        let (len, dist) = lazy_match(
            &input[window_start..lookahead_start], 
            &input[lookahead_start..lookahead_end]
        );

        if len > 0 {
            output.push(Symbol::Pointer {
                length: (len - 3) as u8,
                distance: dist as u16
            });
            lookahead_start += len;
        } else {
            output.push(Symbol::Literal(input[lookahead_start]));
            lookahead_start += 1;
        }
    }

    output
    
}


fn lazy_match(window: &[u8], lookahead: &[u8]) -> (usize, usize) {
    let mut best_match = (0, 0); // (length, distance)

    for i in 0..window.len() {
        let mut match_length = 0;
        for j in 0..lookahead.len() {
            if window[i + j] == lookahead[j] {
                match_length += 1;
            } else {
                break;
            }
        }
        if match_length >= 3 && match_length > best_match.0 {
            best_match = (match_length, lookahead.len() - i);
        }
    }

    best_match
}