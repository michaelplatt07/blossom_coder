// All utils needed for running tests
use super::super::*;

// This is a util function that returns a ViewingWindow struct with default values that can be
// overriden because if we don't have this every test will break when adding a new field. It's
// worth noting I could have a constructor that defaults for the struct but for now this will do.
impl Default for ViewingWindow {
    fn default() -> Self {
        Self {
            absolute_line_num: 0,
            absolute_horz_pos: 0,
            relative_line_num: 0,
            current_lines: vec![],
            lines_before_scroll: vec![],
            lines_after_scroll: vec![],
            window_size: 10,
            update_offset: 0,
            current_mode: CurrentMode::MOVEMENT,
            current_update: Option::None,
        }
    }
}
