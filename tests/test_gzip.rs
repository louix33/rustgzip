use rustgzip::compress_to_gzip;


#[test]
fn test_gzip() {
    compress_to_gzip("tests/example1", "tests/example1.gz").unwrap();
    compress_to_gzip("tests/stdio.h", "tests/stdio.h.gz").unwrap();
}