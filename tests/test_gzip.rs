use rustgzip::compress_to_gzip;


#[test]
fn test_gzip() {
    compress_to_gzip("tests/example", "tests/example.gz").unwrap();
}