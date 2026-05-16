// All tests related to performing updates to the file
use super::super::*;

#[test]
fn test_get_byte_offset_by_key() {
    // Given a Vec of indices that exists and a key that matches an entry in the Vec
    let indices = vec![
        (0, 100),
        (10, 150),
        (20, 200),
        (30, 250),
        (40, 300),
        (50, 350),
    ];
    // When the call is made to find the byte offset
    let byte_offset = get_byte_offset_by_key(10, &indices);
    // Then the byte offset should be returned
    assert_eq!(byte_offset, 150);
}

#[test]
fn test_should_update_returns_true_for_escape_key_in_insert_mode() {
    // Given a viewing window that is currently in insert mode, meaning a user has typed
    let mut viewing_window = ViewingWindow {
        current_mode: CurrentMode::INSERT,
        ..Default::default()
    };

    //When the check is made on the escape key being pressed, we should resolve to true meaning we
    //should store the update
    assert_eq!(should_store_update(&mut viewing_window, true), true)
}
