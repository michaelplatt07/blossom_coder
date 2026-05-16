use blossom_coder::*;
use clap::Parser;
use ncurses::*;
use std::io::{self, Write};
use terminal_size::{terminal_size, Height, Width};

#[derive(Parser)]
#[command(name = "Blossom", about = "Simple CLI file editor")]
struct CliArgs {
    #[arg()]
    file_path: String,

    #[arg(short, long, default_value_t = false)]
    should_debug: bool,
}

fn main() {
    // Set up
    let args = CliArgs::parse();
    let mut file_info = read_file(args.file_path);
    let mut viewing_window = ViewingWindow {
        absolute_line_num: 0,
        absolute_horz_pos: 0,
        relative_line_num: 0,
        current_lines: Vec::new(),
        lines_before_scroll: Vec::new(),
        lines_after_scroll: Vec::new(),
        window_size: 0,
        update_offset: 0,
        current_mode: CurrentMode::MOVEMENT,
        current_update: Option::None,
    };

    // TODO(map) This isn't used yet but will be to handle multiple files. It might make sense to
    // have a structure as well in this case instead of a map to a key that contains a map
    // let file_map: HashMap<String, HashMap<u64, u64>> = HashMap::new();

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
    let (Width(term_width), Height(_term_height)) =
        terminal_size().expect("Could not get terminal size");

    // Misc
    let mut scroll_direction: ScrollDirection = ScrollDirection::NONE;

    /* Start ncurses. */
    initscr();
    curs_set(CURSOR_VISIBILITY::CURSOR_VISIBLE);
    noecho();

    /* Print the file contents to the folder and move the cursor to the upper left corner of the
     * terminal. */
    draw_line_window(0, VISIBLE_LINES_IN_WINDOW, &viewing_window.current_lines);
    mv(y_pos, x_pos);

    // Key input handler
    let mut ch = getch(); // TODO(map) Maybe just convert this instead to the char and compare?
    write_debug_file_info(args.should_debug, format!("Key pressed: {}\n", ch,));
    while ch != 113 {
        // TODO(map) This might be good to move out at some point for testing but right now I would
        // need to pass a lot of different values and return a bunch which feels like it would lend
        // itself nicely to a struct that contains everything.
        if viewing_window.current_mode == CurrentMode::MOVEMENT {
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
                    viewing_window.current_mode = CurrentMode::INSERT;
                    scroll_direction = ScrollDirection::NONE;
                    // Get the current byte offset and store it in the Viewing Window so we know
                    // where the insert began.
                    viewing_window.update_offset = calc_byte_offset_for_insert(
                        &mut file_info,
                        viewing_window.absolute_horz_pos,
                        viewing_window.absolute_line_num,
                    );
                }
                27 => {
                    // ESC Key input
                    print!("\x1b[2 q");
                    io::stdout().flush().unwrap();
                    viewing_window.current_mode = CurrentMode::MOVEMENT;
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

                    // Grab the flag for if the update should store before toggling the mode off
                    let should_store_update = should_store_update(&mut viewing_window, true);

                    // Set insert mode off
                    viewing_window.current_mode = CurrentMode::MOVEMENT;
                    scroll_direction = ScrollDirection::NONE;

                    // TODO(map) Should we include the check for if a current update is present?
                    // Already passing the viewing window
                    let curr_update_option: Option<FileChange> =
                        viewing_window.current_update.take();
                    let curr_update_present: bool = curr_update_option.is_some();
                    if should_store_update && curr_update_present {
                        let curr_update = curr_update_option.unwrap();
                        store_updates_for_save(&mut file_info, curr_update.clone());
                        update_sparse_indices();
                        write_debug_file_info(
                            args.should_debug,
                            format!("Current Upate: {:?}\n", curr_update),
                        );
                    }

                    // Need to reset the string here otherwise we would be appending the next line
                    // to a new string
                    // viewing_window.update_string = String::from("");
                    // write_debug_file_info(
                    //     should_debug,
                    //     format!("String after update: {}\n", viewing_window.update_string),
                    // );
                }
                127 => {
                    // Backspace key pressed
                    if x_pos > 0 {
                        viewing_window.current_lines[viewing_window.relative_line_num as usize]
                            .remove((x_pos - 1) as usize);
                        mv(0, 0);
                        draw_line_window(
                            0,
                            viewing_window.current_lines.len() as u64,
                            &viewing_window.current_lines,
                        );
                        x_pos = x_pos - 1;
                    }
                }
                _ => {
                    // Everything else should be included as typed and modify the document

                    // Track the typed characters as a part of an Insert type
                    if viewing_window.current_update.is_none() {
                        // This is the first key typed as part of the update. We need ot create a
                        // FileChange object to ensure it is present before inserting
                        viewing_window.current_update = Some(FileChange::Insert {
                            insert_offset: viewing_window.update_offset,
                            update_string: String::from(char::from_u32(ch as u32).unwrap()),
                        });
                    } else {
                        if let Some(update) = viewing_window.current_update.as_mut() {
                            update.push_char(char::from_u32(ch as u32).unwrap());
                        };
                    }

                    // Add the characters to draw to the screen
                    viewing_window.current_lines[viewing_window.relative_line_num as usize]
                        .insert(x_pos as usize, char::from_u32(ch as u32).unwrap());
                    mv(0, 0);
                    draw_line_window(
                        0,
                        viewing_window.current_lines.len() as u64,
                        &viewing_window.current_lines,
                    );
                    x_pos = x_pos + 1;
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
        viewing_window.absolute_horz_pos = x_pos as u64;

        write_debug_file_info(
            args.should_debug,
            format!(
            "Rel line num: {} | Absolute line num: {} | Absolute Horz Pos: {} | Byte offset: {}\n",
            viewing_window.relative_line_num,
            viewing_window.absolute_line_num,
            viewing_window.absolute_horz_pos,
            viewing_window.update_offset
        ),
        );

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
