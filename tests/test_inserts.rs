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
        FileChange::Insert {
            insert_offset: 25 as u64,
            update_string: String::from("Hello").clone(),
        },
    );

    // Then the updates will contain the new value
    assert_eq!(
        file_info.updates,
        vec![FileChange::Insert {
            insert_offset: 25 as u64,
            update_string: String::from("Hello")
        }]
    );
}

#[test]
fn test_update_string_saves_to_file_info_multiple_times() {
    // Given a file that has been read
    let file_path = file_utils::get_file_path();
    let mut file_info = read_file(file_path);

    // When a string should be saved to the updates
    store_updates_for_save(
        &mut file_info,
        FileChange::Insert {
            insert_offset: 25 as u64,
            update_string: String::from("Hello"),
        },
    );

    // Then the updates will contain the new value
    assert_eq!(
        file_info.updates,
        vec![FileChange::Insert {
            insert_offset: 25 as u64,
            update_string: String::from("Hello")
        }]
    );

    // When a string should be saved to the updates
    store_updates_for_save(
        &mut file_info,
        FileChange::Insert {
            insert_offset: 86 as u64,
            update_string: String::from("World"),
        },
    );

    // Then the updates will contain the new value
    assert_eq!(
        file_info.updates,
        vec![
            FileChange::Insert {
                insert_offset: 25 as u64,
                update_string: String::from("Hello")
            },
            FileChange::Insert {
                insert_offset: 86 as u64,
                update_string: String::from("World")
            }
        ]
    );
}
