mod file_utils;

use blossom_coder::*;

#[test]
fn test_update_string_saves() {
    // Given a file that has been read
    let file_path = file_utils::get_file_path();
    let mut file_info = read_file(file_path);

    // When a string should be saved to the updates
    store_updates_for_save(
        &mut file_info,
        FileChange::Delete {
            delete_offset: 25 as u64,
            delete_end: 30 as u64,
            deleted_string: String::from("Asdfg"),
        },
    );

    // Then the updates will contain the new value
    assert_eq!(
        file_info.updates,
        vec![FileChange::Delete {
            delete_offset: 25 as u64,
            delete_end: 30 as u64,
            deleted_string: String::from("Asdfg"),
        }]
    );
}
