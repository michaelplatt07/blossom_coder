use ncurses::*;
use std::collections::HashMap;
use std::env;
use std::fs::{write, File, OpenOptions};
use std::io::{self, Seek, SeekFrom, Write};
use std::io::{BufRead, BufReader};
use terminal_size::{terminal_size, Height, Width};

const LINES_BEFORE_SCROLL: u64 = 2;
const LINE_CHUNK_SIZE: u64 = 10;
const LINE_CHUNKS_OUTSIDE_WINDOW: u64 = 2; // How many chunks of the file we load into memory
const VISIBLE_LINES_IN_WINDOW: u64 = 10;
// TODO(map) This should be removed when I move to lazy indexing. For now I want to read through
// everything in the file to make sure I can correctly build up the indices. Later I will only read
// files on a per chunk basis. This should be reflected in the loop that will only go over the next
// N lines.
const FILE_LENGTH: u64 = 1000;

#[derive(PartialEq, Eq, Debug)]
enum ScrollDirection {
    UP,
    DOWN,
    LEFT,
    RIGHT,
    NONE, // Special case of not needing to scroll
}

struct FileInfo {
    file_path: String,
    file: File,
    indices: Vec<(u64, u64)>,
    byte_offset_for_insert: u64,
}

struct ViewingWindow {
    absolute_line_num: u64,
    relative_line_num: u64,
    current_lines: Vec<String>,
    lines_before_scroll: Vec<String>,
    lines_after_scroll: Vec<String>,
    window_size: u64,
}

fn run_sparse_index(file_handle: &mut File) -> Vec<(u64, u64)> {
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

fn read_file_chunk(file_handle: &mut File, start_byte_offset: u64) -> Vec<String> {
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
    let mut _line_num = 0;
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

fn read_file(file_path: String) -> FileInfo {
    // This will need to be mutable at some point due to the growing size of the lines array
    let mut file_handle = File::open(&file_path).expect("Could not open file.");

    // Run the sparse indexing on the entire file which isn't great but that's ok for now.
    let _indices = run_sparse_index(&mut file_handle);

    let file_info = FileInfo {
        file_path: file_path,
        file: file_handle,
        indices: _indices,
        byte_offset_for_insert: 0,
    };
    return file_info;
}

fn draw_line_window(window_start: u64, window_end: u64, lines: &Vec<String>) {
    for i in window_start..window_end {
        addstr(&lines[i as usize]).unwrap();
    }
}

fn get_byte_offset_by_key(key: u64, indices: &Vec<(u64, u64)>) -> u64 {
    // This may be a temp method I will remove at some point. It is just a binary search algorithm
    // that returns the actual byte offset
    match indices.binary_search_by(|(k, _)| k.cmp(&key)) {
        Ok(i) => {
            // exact match
            return indices[i].1;
        }
        Err(i) => {
            // TODO(map) Consider making this safe
            // This shouldn't happen and isn't safe right now because we just panic but we can
            // handle cases where we don't find it moving forward
            panic!("This shouldn't happen");
        }
    }
}

fn look_up_nearest_index(cur_pos: u64, indices: &HashMap<u64, u64>) -> u64 {
    // Util method to quickly look up the nearest index to start seeking from
    return 0;
}

fn calc_byte_offset_for_insert(
    cur_pos: u64,
    viewing_window: &mut ViewingWindow,
    indices: &HashMap<u64, u64>,
) -> u64 {
    /*
     * This method finds the nearest offset based on the sparse parsing and then seeks in the file
     * from that spot to calculate the byte offset for where an insert should actually happen
     * within the file during insert mode. It also considers the most recently used offset point
     * from the sparse index to try and help optimize things a bit.
     */
    return 0;
}

/* Cases to implement
 * 1. Cursor in "middle" of window and just needs to move
 * 2. Cursor at "top" of window and there are lines above, don't move up and scroll
 * 3. Cursor at "top" and there are no lines, just need to move
 */
fn update_cursor_info(
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
            // Case of relative number not at bottom where scroll would trigger
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

fn scroll_window(
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

fn write_debug_file_info(contents: String) {
    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open("output.txt")
        .expect("Failed to open file");
    file.write_all(contents.as_bytes())
        .expect("Failed to write to file");
}

fn main() {
    // Set up
    let args: Vec<String> = env::args().collect();
    let file_path: String = args[1].parse().expect("Should be a path to a file");
    let mut file_info = read_file(file_path);
    let mut viewing_window = ViewingWindow {
        absolute_line_num: 0,
        relative_line_num: 0,
        current_lines: Vec::new(),
        lines_before_scroll: Vec::new(),
        lines_after_scroll: Vec::new(),
        window_size: 0,
    };

    // TODO(map) This isn't used yet but will be to handle multiple files. It might make sense to
    // have a structure as well in this case instead of a map to a key that contains a map
    let mut file_map: HashMap<String, HashMap<u64, u64>> = HashMap::new();

    // Only need to read the first chunk of lines and don't set the lines_before as this will be
    // the first time opening the file and should start at line 0
    viewing_window.current_lines = read_file_chunk(&mut file_info.file, 0);
    let mut byte_offset = get_byte_offset_by_key(
        viewing_window.current_lines.len() as u64,
        &file_info.indices,
    );
    let mut lines_after_scroll: Vec<String> = read_file_chunk(&mut file_info.file, byte_offset);
    byte_offset = get_byte_offset_by_key(
        (viewing_window.current_lines.len() as u64) + LINE_CHUNK_SIZE,
        &file_info.indices,
    );
    lines_after_scroll.append(&mut read_file_chunk(&mut file_info.file, byte_offset));
    viewing_window.lines_after_scroll = lines_after_scroll;

    // Cursor tracking
    let mut x_pos = 0;
    let mut y_pos = 0;
    let (Width(term_width), Height(term_height)) =
        terminal_size().expect("Could not get terminal size");

    // Misc
    let mut scroll_direction: ScrollDirection = ScrollDirection::NONE;
    let mut insert_mode: bool = false;

    /* Start ncurses. */
    initscr();
    curs_set(CURSOR_VISIBILITY::CURSOR_VISIBLE);
    noecho();

    /* Print the file contents to the folder and move the cursor to the upper left corner of the
     * terminal. */
    draw_line_window(0, VISIBLE_LINES_IN_WINDOW, &viewing_window.current_lines);
    mv(y_pos, x_pos);

    // Key input handler
    let mut ch = getch();
    while ch != 113 {
        if insert_mode == false {
            match ch {
                104 => {
                    // H Key input
                    if x_pos - 1 >= 0 {
                        x_pos -= 1;
                    }
                    scroll_direction = ScrollDirection::LEFT;
                }
                108 => {
                    // L Key input
                    if x_pos + 1 < term_width.into() {
                        x_pos += 1;
                    }
                    scroll_direction = ScrollDirection::RIGHT;
                }
                106 => {
                    // J Key input
                    // TODO(map) Uncomment once we fill the whole screen with lines of text
                    // if y_pos + 1 < term_height.into() {
                    if (y_pos as u64 + 1) < VISIBLE_LINES_IN_WINDOW {
                        scroll_direction = ScrollDirection::DOWN;
                    } else {
                        scroll_direction = ScrollDirection::NONE;
                    }
                }
                107 => {
                    // K Key input
                    if y_pos - 1 >= 0 {
                        scroll_direction = ScrollDirection::UP;
                    } else {
                        scroll_direction = ScrollDirection::NONE;
                    }
                }
                105 => {
                    // I Key input
                    print!("\x1b[4 q");
                    io::stdout().flush().unwrap();
                    insert_mode = true;
                    scroll_direction = ScrollDirection::NONE;
                }
                27 => {
                    // ESC Key input
                    print!("\x1b[2 q");
                    io::stdout().flush().unwrap();
                    insert_mode = false;
                    scroll_direction = ScrollDirection::NONE;
                }
                _ => {}
            }
        } else {
            match ch {
                27 => {
                    // ESC Key input
                    print!("\x1b[2 q");
                    io::stdout().flush().unwrap();
                    insert_mode = false;
                    scroll_direction = ScrollDirection::NONE;
                }
                _ => {
                    // Everything else should be included as typed and modify the document
                }
            }
        }

        // Handle scrolling
        let (y_movement, x_movement, new_rel, new_abs) =
            update_cursor_info(&mut viewing_window, &mut scroll_direction);
        viewing_window.relative_line_num = new_rel;
        y_pos = y_pos + y_movement;
        x_pos = x_pos + x_movement;
        viewing_window.absolute_line_num = new_abs;

        write_debug_file_info(format!(
            "Rel line num: {} | Absolute line num: {} | Byte Offset: {}\n",
            viewing_window.relative_line_num,
            viewing_window.absolute_line_num,
            file_info.byte_offset_for_insert
        ));

        let update_window: bool = scroll_window(&mut viewing_window, &mut scroll_direction);
        if update_window {
            mv(0, 0);
            draw_line_window(
                0,
                viewing_window.current_lines.len() as u64,
                &viewing_window.current_lines,
            );
        }

        // Move the cursor back before rendering
        mv(y_pos, x_pos);
        refresh();

        ch = getch();
    }

    /* Terminate ncurses. */
    endwin();
    print!("\x1b[2 q");
    io::stdout().flush().unwrap();
}
