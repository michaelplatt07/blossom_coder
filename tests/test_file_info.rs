mod file_utils;

use blossom_coder::*;

#[test]
fn test_read_file() {
    let file_path = file_utils::get_file_path();
    let file_info = read_file(file_path);
    assert!(!file_info.file_path.is_empty());
    assert!(!file_info.indices.is_empty());

    assert_eq!(
        file_info.indices,
        vec![(10, 210), (20, 420), (30, 630), (40, 840), (50, 1050)]
    );
}

#[test]
fn test_get_byte_offset_at_key_in_file() {
    // Given a file that has been read
    let file_path = file_utils::get_file_path();
    let mut file_info = read_file(file_path);

    // When the method is called to get the offset for the insert and the y position of the cursor
    // falls on an index and the x position of the cursor is the left most position
    let insert_offset = calc_byte_offset_for_insert(&mut file_info, 0, 10);

    // Then the insert_offset will be updated to the where the cursor was
    assert_eq!(insert_offset, 210);
}

#[test]
fn test_get_byte_offset_at_key_in_file_offset_cursor() {
    // Given a file that has been read
    let file_path = file_utils::get_file_path();
    let mut file_info = read_file(file_path);

    // When the method is called to get the offset for the insert and the y position of the cursor
    // falls on an index and the x position of the cursor has moved to the right in the file
    let insert_offset = calc_byte_offset_for_insert(&mut file_info, 5, 10);

    // Then the insert_offset will be updated to the where the cursor was
    assert_eq!(insert_offset, 215);
}

#[test]
fn test_get_byte_offset_at_random_position_in_file_no_offset_cursor() {
    // Given a file that has been read
    let file_path = file_utils::get_file_path();
    let mut file_info = read_file(file_path);

    // When the method is called to get the offset for the insert and the y position of the cursor
    // does not fall on an index and the x position of the cursor is the left mos position
    let insert_offset = calc_byte_offset_for_insert(&mut file_info, 0, 13);

    // Then the insert_offset will be updated to the where the cursor was
    assert_eq!(insert_offset, 273);
}

#[test]
fn test_get_byte_offset_at_random_position_in_file_offset_cursor() {
    // Given a file that has been read
    let file_path = file_utils::get_file_path();
    let mut file_info = read_file(file_path);

    // When the method is called to get the offset for the insert and the y position of the cursor
    // falls on an index and the x position of the cursor has moved to the right in the file
    let insert_offset = calc_byte_offset_for_insert(&mut file_info, 5, 13);

    // Then the insert_offset will be updated to the where the cursor was
    assert_eq!(insert_offset, 278);
}

#[test]
fn test_update_string_saves_to_file_info() {
    // Given a file that has been read
    let file_path = file_utils::get_file_path();
    let mut file_info = read_file(file_path);

    // When a string shuld be saved to the updates
    store_updates_for_save(&mut file_info, 25 as u64, String::from("Hello").clone());

    // Then the updates will contain the new value
    assert_eq!(file_info.updates, vec![(25 as u64, String::from("Hello"))]);
}

#[test]
fn test_update_string_saves_to_file_info_multiple_times() {
    // Given a file that has been read
    let file_path = file_utils::get_file_path();
    let mut file_info = read_file(file_path);

    // When a string shuld be saved to the updates
    store_updates_for_save(&mut file_info, 25 as u64, String::from("Hello").clone());

    // Then the updates will contain the new value
    assert_eq!(file_info.updates, vec![(25 as u64, String::from("Hello"))]);

    // When a string shuld be saved to the updates
    store_updates_for_save(&mut file_info, 86 as u64, String::from("World").clone());

    // Then the updates will contain the new value
    assert_eq!(
        file_info.updates,
        vec![
            (25 as u64, String::from("Hello")),
            (86 as u64, String::from("World"))
        ]
    );
}
