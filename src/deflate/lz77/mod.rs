pub(super) mod encode;

use std::fmt::Display;

pub(super) const WINDOW_SIZE: usize = 32768;
pub(super) const LOOKAHEAD_SIZE: usize = 258;

#[derive(Debug, Clone, Copy)]
pub(crate) enum Symbol {
    Literal(u8),
    Pointer { 
        length: u8,  // 3-258, represented by 0-255
        distance: u16 // 1-32768
    }
}

impl Display for Symbol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Symbol::Literal(c) => write!(f, "{:02X}", c),
            Symbol::Pointer { length, distance } => write!(f, "({},{})", *length as u16 + 3, distance)
        }
    }
}

pub(crate) enum Lz77Status {
    Normal(Vec<Symbol>),
    LastBlock(Vec<Symbol>)
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;

    #[test]
    fn test_symbol_display() {
        assert_eq!(format!("{}", Symbol::Literal(10)), "0A");
        assert_eq!(format!("{}", Symbol::Pointer { length: 3, distance: 6 }), "(3,6)");
        // assert_eq!(format!("{}", vec![Symbol::Literal(10),Symbol::Pointer { length: 3, distance: 6 },Symbol::EndOfBlock]), "(255,32768)");
    }

    #[test]
    fn test_read_eof() {
        let mut reader = std::io::BufReader::new(std::io::Cursor::new(vec![2u8; 1]));
        let mut buf = [0u8; 1];

        let mut r = reader.read_exact(&mut buf);
        assert!(r.is_ok());
        assert_eq!(buf, [2u8]);

        r = reader.read_exact(&mut buf);
        assert!(r.is_err());
        assert_eq!(r.unwrap_err().kind(), std::io::ErrorKind::UnexpectedEof);

        r = reader.read_exact(&mut buf);
        assert!(r.is_err());
        assert_eq!(r.unwrap_err().kind(), std::io::ErrorKind::UnexpectedEof);
    }

}