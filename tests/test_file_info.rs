mod file_utils;

use blossom_coder::*;

#[test]
fn test_read_file() {
    let file_path = file_utils::get_file_path();
    let file_info = read_file(file_path);
    assert!(!file_info.file_path.is_empty());
    assert!(!file_info.indices.is_empty());

    assert_eq!(file_info.indices, vec![(10, 261), (20, 531), (30, 801)]);
}
