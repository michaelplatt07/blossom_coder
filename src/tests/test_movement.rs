// All tests related to moving the cursor around the screen
use super::super::*;

#[test]
fn test_update_cursor_info_no_direction() {
    let mut viewing_window = ViewingWindow::default();
    let mut scroll_direction: ScrollDirection = ScrollDirection::NONE;
    assert_eq!(
        update_cursor_info(&mut viewing_window, &mut scroll_direction),
        (0, 0, 0, 0)
    );
}

#[test]
fn test_update_cursor_info_move_up_update_all_info() {
    // Given a viewing window where the user has scrolled into the middle of the screen
    let mut viewing_window = ViewingWindow {
        absolute_line_num: LINES_BEFORE_SCROLL + 3,
        relative_line_num: LINES_BEFORE_SCROLL + 3,
        ..Default::default()
    };
    // When a user attempts to scroll up
    let mut scroll_direction: ScrollDirection = ScrollDirection::UP;
    // THen all the cursor info should be updated
    assert_eq!(
        update_cursor_info(&mut viewing_window, &mut scroll_direction),
        (-1, 0, 4, 4)
    );
}

#[test]
fn test_update_cursor_info_move_up_update_absolute_but_not_relative() {
    // Given a viewing window where the user has scrolled down enough that the absolute line number
    // has gotten higher than the relative line number and the user has started scrolling up again
    let mut viewing_window = ViewingWindow {
        absolute_line_num: LINES_BEFORE_SCROLL + 3,
        relative_line_num: LINES_BEFORE_SCROLL,
        lines_before_scroll: vec![
            "Line 1".to_string(),
            "Line 2".to_string(),
            "Line 3".to_string(),
        ],
        ..Default::default()
    };
    // When a user attempts to scroll up and there are lines off the screen
    let mut scroll_direction: ScrollDirection = ScrollDirection::UP;
    // Then the new cursor info should update the absolute line number and nothing else
    assert_eq!(
        update_cursor_info(&mut viewing_window, &mut scroll_direction),
        (0, 0, 2, 4)
    );
}

#[test]
fn test_update_cursor_info_move_up_update_all_info_near_top_of_file() {
    // Given a viewing window where the user is near the top of the file and there are no lines off
    // the top of the screen
    let mut viewing_window = ViewingWindow {
        absolute_line_num: LINES_BEFORE_SCROLL,
        relative_line_num: LINES_BEFORE_SCROLL,
        ..Default::default()
    };
    // When a user attempts to scroll up
    let mut scroll_direction: ScrollDirection = ScrollDirection::UP;
    // Then the new cursor info should update all the info
    assert_eq!(
        update_cursor_info(&mut viewing_window, &mut scroll_direction),
        (-1, 0, 1, 1)
    );
}

#[test]
fn test_update_cursor_info_move_down_update_all_info() {
    // Given a viewing window where the user has scrolled into the middle of the screen
    let mut viewing_window = ViewingWindow {
        absolute_line_num: LINES_BEFORE_SCROLL + 3,
        relative_line_num: LINES_BEFORE_SCROLL + 3,
        ..Default::default()
    };
    // When a user attempts to scroll down
    let mut scroll_direction: ScrollDirection = ScrollDirection::DOWN;
    // Then all the cursor info should be updated
    assert_eq!(
        update_cursor_info(&mut viewing_window, &mut scroll_direction),
        (1, 0, 6, 6)
    );
}

#[test]
fn test_update_cursor_info_move_down_update_absolute_but_not_relative() {
    // Given a viewing window where the user has scrolled down to the bottom of the viewing window
    // but there are still lines off screen
    let mut viewing_window = ViewingWindow {
        absolute_line_num: VISIBLE_LINES_IN_WINDOW - LINES_BEFORE_SCROLL,
        relative_line_num: VISIBLE_LINES_IN_WINDOW - LINES_BEFORE_SCROLL,
        lines_after_scroll: vec![
            "Line 1".to_string(),
            "Line 2".to_string(),
            "Line 3".to_string(),
        ],
        ..Default::default()
    };
    // When a user attempts to scroll down
    let mut scroll_direction: ScrollDirection = ScrollDirection::DOWN;
    // Then the new cursor info should update the absolute line number and nothing else
    assert_eq!(
        update_cursor_info(&mut viewing_window, &mut scroll_direction),
        (0, 0, 8, 9)
    );
}

#[test]
fn test_update_cursor_info_move_down_update_all_info_near_bottom_of_file() {
    // Given a viewing window where the user is near the bottom of the file and there are no lines off
    // the bottom of the screen
    let mut viewing_window = ViewingWindow {
        absolute_line_num: VISIBLE_LINES_IN_WINDOW - 2,
        relative_line_num: VISIBLE_LINES_IN_WINDOW - 2,
        ..Default::default()
    };
    // When a user attempts to scroll down
    let mut scroll_direction: ScrollDirection = ScrollDirection::DOWN;
    // Then the new cursor info should update all the info
    assert_eq!(
        update_cursor_info(&mut viewing_window, &mut scroll_direction),
        (1, 0, 9, 9)
    );
}

#[test]
fn test_scroll_window_down_does_not_scroll() {
    // Given a viewing window where there is no need to scroll the actual window because there was
    // just a cursor move
    let current_lines: Vec<String> = vec!["Line 1".to_string()];
    let lines_before_scroll: Vec<String> = vec!["Line 2".to_string()];
    let lines_after_scroll: Vec<String> = vec!["Line 3".to_string()];
    let mut viewing_window = ViewingWindow {
        absolute_line_num: 5,
        relative_line_num: 5,
        current_lines: current_lines.clone(),
        lines_before_scroll: lines_before_scroll.clone(),
        lines_after_scroll: lines_after_scroll.clone(),
        ..Default::default()
    };
    // When the scroll window method is called
    let mut scroll_direction: ScrollDirection = ScrollDirection::DOWN;
    // The viewing window lines are not updated and false is returned
    assert_eq!(
        scroll_window(&mut viewing_window, &mut scroll_direction),
        false
    );
    assert_eq!(viewing_window.current_lines, current_lines);
    assert_eq!(viewing_window.lines_before_scroll, lines_before_scroll);
    assert_eq!(viewing_window.lines_after_scroll, lines_after_scroll);
}

#[test]
fn test_scroll_window_down_scrolls() {
    // Given a viewing window where the user is reaching a point a scroll would happen
    let current_lines: Vec<String> = vec!["Line 1".to_string()];
    let lines_before_scroll: Vec<String> = vec!["Line 2".to_string()];
    let lines_after_scroll: Vec<String> = vec!["Line 3".to_string()];
    let mut viewing_window = ViewingWindow {
        absolute_line_num: VISIBLE_LINES_IN_WINDOW - LINES_BEFORE_SCROLL,
        relative_line_num: VISIBLE_LINES_IN_WINDOW - LINES_BEFORE_SCROLL,
        current_lines: current_lines.clone(),
        lines_before_scroll: lines_before_scroll.clone(),
        lines_after_scroll: lines_after_scroll.clone(),
        ..Default::default()
    };
    // When the scroll window method is called
    let mut scroll_direction: ScrollDirection = ScrollDirection::DOWN;
    // The viewing window lines are not updated and false is returned
    assert_eq!(
        scroll_window(&mut viewing_window, &mut scroll_direction),
        true
    );
    assert_eq!(viewing_window.current_lines.len(), 1);
    assert_eq!(viewing_window.current_lines, vec!["Line 3".to_string()]);
    assert_eq!(viewing_window.lines_before_scroll.len(), 2);
    assert_eq!(
        viewing_window.lines_before_scroll,
        vec!["Line 2".to_string(), "Line 1".to_string()]
    );
    assert_eq!(viewing_window.lines_after_scroll.len(), 0);
}

#[test]
fn test_scroll_window_up_does_not_scroll() {
    // Given a viewing window where there is no need to scroll the actual window because there was
    // just a cursor move
    let current_lines: Vec<String> = vec!["Line 1".to_string()];
    let lines_before_scroll: Vec<String> = vec!["Line 2".to_string()];
    let lines_after_scroll: Vec<String> = vec!["Line 3".to_string()];
    let mut viewing_window = ViewingWindow {
        absolute_line_num: 5,
        relative_line_num: 5,
        current_lines: current_lines.clone(),
        lines_before_scroll: lines_before_scroll.clone(),
        lines_after_scroll: lines_after_scroll.clone(),
        ..Default::default()
    };
    // When the scroll window method is called
    let mut scroll_direction: ScrollDirection = ScrollDirection::UP;
    // The viewing window lines are not updated and false is returned
    assert_eq!(
        scroll_window(&mut viewing_window, &mut scroll_direction),
        false
    );
    assert_eq!(viewing_window.current_lines, current_lines);
    assert_eq!(viewing_window.lines_before_scroll, lines_before_scroll);
    assert_eq!(viewing_window.lines_after_scroll, lines_after_scroll);
}

#[test]
fn test_scroll_window_up_scrolls() {
    // Given a viewing window where the user is reaching a point a scroll would happen
    let current_lines: Vec<String> = vec!["Line 1".to_string()];
    let lines_before_scroll: Vec<String> = vec!["Line 2".to_string()];
    let lines_after_scroll: Vec<String> = vec!["Line 3".to_string()];
    let mut viewing_window = ViewingWindow {
        absolute_line_num: LINES_BEFORE_SCROLL,
        relative_line_num: LINES_BEFORE_SCROLL,
        current_lines: current_lines.clone(),
        lines_before_scroll: lines_before_scroll.clone(),
        lines_after_scroll: lines_after_scroll.clone(),
        ..Default::default()
    };
    // When the scroll window method is called
    let mut scroll_direction: ScrollDirection = ScrollDirection::UP;
    // The viewing window lines are not updated and false is returned
    assert_eq!(
        scroll_window(&mut viewing_window, &mut scroll_direction),
        true
    );
    assert_eq!(viewing_window.current_lines.len(), 1);
    assert_eq!(viewing_window.current_lines, vec!["Line 2".to_string()]);
    assert_eq!(viewing_window.lines_before_scroll.len(), 0);
    assert_eq!(viewing_window.lines_after_scroll.len(), 2);
    assert_eq!(
        viewing_window.lines_after_scroll,
        vec!["Line 1".to_string(), "Line 3".to_string()]
    );
}
