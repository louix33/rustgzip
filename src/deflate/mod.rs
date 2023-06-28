mod decoder;
mod encoder;
mod lz77;
mod huffman;

#[derive(Debug, Clone, Copy)]
enum Symbol {
    Literal(u8),
    Pointer { 
        length: u8,  // 3-258, represented by 0-255
        distance: u16 // 1-32768
    },
    EndOfBlock
}
