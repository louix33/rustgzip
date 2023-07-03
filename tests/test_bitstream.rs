use bitstream_io::{BitWriter, LittleEndian, BitWrite, BigEndian};

#[test]
fn test_to_written() {
    let mut buf = Vec::new();
    let mut buf2 = Vec::new();
    let mut writer_be = BitWriter::endian(&mut buf, BigEndian);
    let mut writer_le = BitWriter::endian(&mut buf2, LittleEndian);

    writer_be.write(18, 0b0011_0011_1100_1100_10).unwrap();
    writer_le.write(18, 0b0011_0011_1100_1100_10).unwrap();
    let unwritten_be = writer_be.into_unwritten();
    let unwritten_le = writer_le.into_unwritten();

    assert_eq!(buf, vec![0b0011_0011, 0b1100_1100]);
    assert_eq!(buf2, vec![0b00_1100_10, 0b11_0011_11]);
    assert_eq!(unwritten_be, (2_u32, 0b10_u8));
    assert_eq!(unwritten_le, (2_u32, 0b00_u8));
}