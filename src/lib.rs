use ncurses::*;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader};
use std::io::{Seek, SeekFrom, Write};

pub const LINES_BEFORE_SCROLL: u64 = 2;
pub const LINE_CHUNK_SIZE: u64 = 10;
pub const VISIBLE_LINES_IN_WINDOW: u64 = 10;
// TODO(map) This should be removed when I move to lazy indexing. For now I want to read through
// everything in the file to make sure I can correctly build up the indices. Later I will only read
// files on a per chunk basis. This should be reflected in the loop that will only go over the next
// N lines.
const FILE_LENGTH: u64 = 1000;

#[derive(PartialEq, Eq, Debug)]
pub enum ScrollDirection {
    UP,
    DOWN,
    LEFT,
    RIGHT,
    NONE, // Special case of not needing to scroll
}

#[derive(PartialEq, Eq, Debug)]
pub enum CurrentMode {
    INSERT,
    MOVEMENT,
    COMMAND, // This may not be needed but keeping it for now
}

pub struct FileInfo {
    pub file_path: String,
    pub file: File,
    pub indices: Vec<(u64, u64)>,
    pub updates: Vec<FileChange>,
}

pub struct ViewingWindow {
    pub absolute_line_num: u64,
    pub absolute_horz_pos: u64,
    pub relative_line_num: u64,
    pub current_lines: Vec<String>,
    pub lines_before_scroll: Vec<String>,
    pub lines_after_scroll: Vec<String>,
    pub window_size: u64,
    pub update_offset: u64,
    pub current_mode: CurrentMode,
    pub current_update: Option<FileChange>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum FileChange {
    Insert {
        insert_offset: u64,
        update_string: String,
    },
    Delete {
        delete_offset: u64,
        delete_end: u64,
        deleted_string: String,
    },
}

impl FileChange {
    pub fn push_char(&mut self, update_char: char) {
        match self {
            FileChange::Insert { update_string, .. } => {
                update_string.push(update_char);
            }
            FileChange::Delete { .. } => {}
        };
    }
}

pub fn run_sparse_index(file_handle: &mut File) -> Vec<(u64, u64)> {
    /*
     * It's worth noting that parsing the whole file is not efficient. There are better ways to do
     * this including a BTreeMap or something more complex like a structure that would start with a
     * Vec and promote itself to the BTreeMap once a threshold is met. Another thing that will have
     * to happen here is that there will need to be a map of files to their sparsely indexed maps
     */
    let mut buf_reader = BufReader::new(file_handle);
    let mut file_lines = Vec::new();
    let mut buffer = String::new();
    let mut bytes_read_for_offset: u64 = 0;
    let mut _line_num = 0;
    let mut _indices: Vec<(u64, u64)> = Vec::new();

    for idx in 0..FILE_LENGTH {
        let bytes_read = buf_reader
            .read_line(&mut buffer)
            .expect("Failed to read file") as u64;
        if bytes_read == 0 {
            break; // EOF reached
        }
        _line_num += 1;
        bytes_read_for_offset += bytes_read;
        file_lines.push(buffer.clone());
        buffer.clear();
        if _line_num == LINE_CHUNK_SIZE {
            _indices.push((idx + 1, bytes_read_for_offset));
            _line_num = 0;
        }
    }

    return _indices;
}

pub fn read_file_chunk(file_handle: &mut File, start_byte_offset: u64) -> Vec<String> {
    // TODO(map): Though doing a restart from the beginning does work, I shouldn't do this.
    // Instead, I should probably do an initial setup method and then take an offset into this
    // method to seek and start from the right place.

    // This is ugly but seems to be a thing for Rust. Use usize everywhere as an idiom but then
    // there are some scenarios where things don't want a usize like seeking below. In that case,
    // we apparently just need to cast and hope we don't panic. There is no architecture that holds
    // more than 64 bits but still doesn't feel great.
    file_handle
        .seek(SeekFrom::Start(start_byte_offset))
        .unwrap();

    let mut buf_reader = BufReader::new(file_handle);
    let mut file_lines = Vec::new();
    let mut buffer = String::new();

    for _ in 0..LINE_CHUNK_SIZE {
        let bytes_read = buf_reader
            .read_line(&mut buffer)
            .expect("Failed to read file");
        if bytes_read == 0 {
            break; // EOF reached
        }
        file_lines.push(buffer.clone());
        buffer.clear();
    }

    return file_lines;
}

pub fn get_bytes_from_index(
    file_handle: &mut File,
    start_byte_offset: u64,
    closest_line_num: u64,
    cur_x_pos: u64,
    cur_y_pos: u64,
) -> u64 {
    file_handle
        .seek(SeekFrom::Start(start_byte_offset))
        .unwrap();

    let mut buf_reader = BufReader::new(file_handle);
    let mut buffer = String::new();
    let extra_lines = cur_y_pos - closest_line_num;
    let mut extra_bytes_read: u64 = 0;

    for i in 0..extra_lines {
        let bytes_read = buf_reader
            .read_line(&mut buffer)
            .expect("Failed to read file");

        if bytes_read == 0 {
            break; // EOF reached
        }
        extra_bytes_read += bytes_read as u64;
        if i == extra_lines && cur_x_pos != 0 {
            // Don't clear the buffer in case we are on the last line and need to read the bytes of
            // a string up to an offset
            break;
        }
        buffer.clear();
    }

    // Need to check if the x position is not at the beginning of the line and read that whole
    // line. The above loop does not account for finding an exact match in the indices so we would
    // have an empty buffer which is not correct.
    if cur_x_pos != 0 {
        buf_reader
            .read_line(&mut buffer)
            .expect("Failed to read file");
    }

    if buffer.len() != 0 {
        let chars: Vec<char> = buffer.chars().collect();
        // We need to add the bytes for the offset in the x positon
        for i in 0..cur_x_pos {
            // let char = chars.nth(i as usize).expect("Failed to get char.");
            let char = chars[i as usize];
            extra_bytes_read += char.len_utf8() as u64;
        }
    }

    return start_byte_offset + extra_bytes_read;
}

pub fn draw_line_window(window_start: u64, window_end: u64, lines: &Vec<String>) {
    for i in window_start..window_end {
        addstr(&lines[i as usize]).unwrap();
    }
}

pub fn get_byte_offset_by_key(key: u64, indices: &Vec<(u64, u64)>) -> u64 {
    // This may be a temp method I will remove at some point. It is just a binary search algorithm
    // that returns the actual byte offset
    match indices.binary_search_by(|(k, _)| k.cmp(&key)) {
        Ok(i) => {
            // exact match
            return indices[i].1;
        }
        Err(_i) => {
            // TODO(map) Consider making this safe
            // This shouldn't happen and isn't safe right now because we just panic but we can
            // handle cases where we don't find it moving forward
            panic!("This shouldn't happen");
        }
    }
}

// TODO(map) Probably want to consolidate this method with the one above. Right now they are
// separate because I want a specific error if the key isn't found
pub fn look_up_nearest_index(cur_pos: u64, indices: &Vec<(u64, u64)>) -> (u64, u64) {
    // Util method to quickly look up the nearest index to start seeking from
    match indices.binary_search_by(|(k, _)| k.cmp(&cur_pos)) {
        Ok(i) => {
            // exact match
            return indices[i];
        }
        Err(0) => {
            // Base case of before the first index
            return (0, 0);
        }
        Err(i) => {
            return indices[i - 1];
        }
    }
}

pub fn calc_byte_offset_for_insert(
    file_info: &mut FileInfo,
    cur_x_pos: u64,
    cur_y_pos: u64,
) -> u64 {
    /*
     * This method finds the nearest offset based on the sparse parsing and then seeks in the file
     * from that spot to calculate the byte offset for where an insert should actually happen
     * within the file during insert mode. It also considers the most recently used offset point
     * from the sparse index to try and help optimize things a bit.
     */
    if cur_y_pos % LINE_CHUNK_SIZE == 0 && cur_x_pos == 0 {
        // We are on a key on the indices and the cursor hasn't been moved from the start of the
        // line so we can just get the byte offset and roll with it.
        return get_byte_offset_by_key(cur_y_pos, &file_info.indices);
    } else {
        // We need to get the nearest starting point and then potentially offset by some
        // additional bytes based on extra lines from the offset and how far right the cursor
        // has moved
        let (closest_line_num, start_byte_offset) =
            look_up_nearest_index(cur_y_pos, &file_info.indices);
        return get_bytes_from_index(
            &mut file_info.file,
            start_byte_offset,
            closest_line_num,
            cur_x_pos,
            cur_y_pos,
        );
    }
}

pub fn should_store_update(viewing_window: &mut ViewingWindow, esc_pressed: bool) -> bool {
    // Helper method that can determine if an update should be stored to the file info to later be
    // written. The cases where we would want to do this are:
    // 1. If the escape key is pressed when insert mode has previously been on
    // 2. When the backspace is pressed after the previous key pressed was counted as an insert
    // 3. When a key leads to an insert after the space key was pressed

    if esc_pressed && viewing_window.current_mode == CurrentMode::INSERT {
        return true;
    }

    // Default case should always return false to prevent unintended changes
    return false;
}

// This is a dumb mehtod but can be unit tested which I like being able to do so we are breaking
// it out for now. Maybe an integration test can do this better or something at some point
pub fn store_updates_for_save(file_info: &mut FileInfo, curr_update: FileChange) {
    file_info.updates.push(curr_update);
}

pub fn update_sparse_indices() {
    // TODO(map) Implement me.
    // This method should work by going from the nearest index where the first change was made and
    // recalculating the offset based on the text that was inserted. The logic will also need to
    // run the calculation update with additional bytes of data for each insert that happens. This
    // means that if you have offsets at every 10 bytes, and 5 bytes were added at the 15 byte
    // offset, that value would then become 20, the 20 would be 25, etc. If a user inserts at the
    // original 30 for additional 3 bytes, the 30 offset would become 30 + 5 for the original bytes
    // plus 3 more for the new bytes.
    //
    // One consideration is whether or not we should run this when an update is made to the
    // vector of inserts or when the save happens. If this happens after the save then we can limit
    // the overhead but the jumping around could be bad. If we do it every time we update the
    // string then we have higher overhead but this would ensure jumping around would be accurate.
    //
    // Seems like I will need to do this after every update given the fact that without this method
    // running the sparse indices will be broken on the next update
}

pub fn write_file_changes() {}

pub fn update_cursor_info(
    viewing_window: &mut ViewingWindow,
    scroll_direction: &mut ScrollDirection,
) -> (i32, i32, u64, u64) {
    match scroll_direction {
        ScrollDirection::UP => {
            // Case of relative number not at top where scroll would trigger
            if viewing_window.relative_line_num > LINES_BEFORE_SCROLL {
                return (
                    -1,
                    0,
                    viewing_window.relative_line_num - 1,
                    viewing_window.absolute_line_num - 1,
                );
            }
            // Case of number at top and there are no lines above
            if viewing_window.relative_line_num <= LINES_BEFORE_SCROLL
                && viewing_window.lines_before_scroll.len() == 0
            {
                return (
                    -1,
                    0,
                    viewing_window.relative_line_num - 1,
                    viewing_window.absolute_line_num - 1,
                );
            }
            return (
                0,
                0,
                viewing_window.relative_line_num,
                viewing_window.absolute_line_num - 1,
            );
        }
        ScrollDirection::DOWN => {
            // Case of relative number not at bottom where scroll would trigger, middle of the
            // screen
            if viewing_window.relative_line_num < VISIBLE_LINES_IN_WINDOW - LINES_BEFORE_SCROLL - 1
            {
                return (
                    1,
                    0,
                    viewing_window.relative_line_num + 1,
                    viewing_window.absolute_line_num + 1,
                );
            }
            // Case of number at bottom and there are no lines below
            if viewing_window.relative_line_num >= LINES_BEFORE_SCROLL
                && viewing_window.lines_after_scroll.len() == 0
            {
                return (
                    1,
                    0,
                    viewing_window.relative_line_num + 1,
                    viewing_window.absolute_line_num + 1,
                );
            }
            return (
                0,
                0,
                viewing_window.relative_line_num,
                viewing_window.absolute_line_num + 1,
            );
        }
        _ => {
            return (
                0,
                0,
                viewing_window.relative_line_num,
                viewing_window.absolute_line_num,
            );
        }
    }
}

pub fn scroll_window(
    viewing_window: &mut ViewingWindow,
    scroll_direction: &mut ScrollDirection,
) -> bool {
    // TODO(map) Look into VecDequeu for efficient vector modification
    // TODO(map) This only handles a single line scroll at a time. Will need to add some additional
    // logic in place for jumping and large scrolls
    match scroll_direction {
        ScrollDirection::DOWN => {
            if viewing_window.relative_line_num + 1 >= VISIBLE_LINES_IN_WINDOW - LINES_BEFORE_SCROLL
                && viewing_window.lines_after_scroll.len() > 0
            {
                // Remove the first line from the current lines vector and push it to the vector
                // that tracks the lines before the window
                viewing_window
                    .lines_before_scroll
                    .push(viewing_window.current_lines.remove(0));
                // Remove the first line from the vector that tracks lines after the window and
                // push it to the vector for the current window
                viewing_window
                    .current_lines
                    .push(viewing_window.lines_after_scroll.remove(0));

                return true;
            }
            return false;
        }
        ScrollDirection::UP => {
            if viewing_window.relative_line_num > 0 // Have to do this for protection from
                                                    // underflow panic
                && viewing_window.relative_line_num - 1 < LINES_BEFORE_SCROLL
                && viewing_window.lines_before_scroll.len() > 0
            {
                // Remove the last line from the current lines vector and push it to the vector
                // that tracks the lines after the window
                viewing_window.lines_after_scroll.insert(
                    0,
                    viewing_window
                        .current_lines
                        .remove(viewing_window.current_lines.len() - 1),
                );
                // Remove the last line from the vector that tracks lines before the window and
                // pushes it to the front of the current lines for the window
                viewing_window.current_lines.insert(
                    0,
                    viewing_window
                        .lines_before_scroll
                        .remove(viewing_window.lines_before_scroll.len() - 1),
                );

                return true;
            }
            return false;
        }
        _ => {
            return false;
        }
    }
}

pub fn write_debug_file_info(should_debug: bool, contents: String) {
    if should_debug {
        let mut file = OpenOptions::new()
            .append(true)
            .create(true)
            .open("output.txt")
            .expect("Failed to open file");
        file.write_all(contents.as_bytes())
            .expect("Failed to write to file");
    }
}

pub fn read_file(file_path: String) -> FileInfo {
    // This will need to be mutable at some point due to the growing size of the lines array
    let mut file_handle = File::open(&file_path).expect("Could not open file.");

    // Run the sparse indexing on the entire file which isn't great but that's ok for now.
    let _indices = run_sparse_index(&mut file_handle);

    let file_info = FileInfo {
        file_path: file_path,
        file: file_handle,
        indices: _indices,
        updates: Vec::new(),
    };
    return file_info;
}

// This is required to run unit tests apparently. I don't like having unit tests in the same file
// as my actual code though so I can use these two lines and write my module in a separate file.
#[cfg(test)]
mod tests;
