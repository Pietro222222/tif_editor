use std::{fs::OpenOptions, io::Write};

use anyhow::Result;
use color::Color;
use editor::Editor;
use libtif::{image::TifImage, pixel::PixelColor};
use mode::Mode;
use pancurses::{endwin, Input};
use clap::Parser;

mod color;
mod cursor;
mod editor;
mod mode;
mod pallete;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long, value_parser)]
    file: String,

    #[clap(short, long, value_parser)]
    create: bool,

    #[clap(short, long, value_parser)]
    height: i32,

    #[clap(short, long, value_parser)]
    width: u8
}

fn main() -> Result<()> {

    let args = Args::parse();

    let tif = if !args.create {
        TifImage::parse_from_file(args.file.to_owned()).expect("Couldnt parse file!")
    }else {
        TifImage {
            height: args.height as u64,
            width: args.width as u8,
            pixels: vec![vec![PixelColor::Black;args.width as usize]; args.height as usize]
        }
    };

    let mut editor = Editor::new(tif);
    editor.is_terminal_size_enough()?;
    editor.draw_ui().ok();
    //    editor.set_cursor_pos((2, 3))?;
    'editor: loop {
        if let Some(c) = editor.getch() {
            let mut cursor_pos = editor.cursor.pos;
            match c {
                Input::KeyLeft => {
                    cursor_pos.1 -= 1;
                    editor.set_cursor_pos(cursor_pos).ok(); //using the result of this function might end up in unncessary crashes
                }
                Input::KeyRight => {
                    cursor_pos.1 += 1;
                    editor.set_cursor_pos(cursor_pos).ok();
                }
                Input::KeyUp => {
                    cursor_pos.0 -= 1;
                    editor.set_cursor_pos(cursor_pos).ok();
                }
                Input::KeyDown => {
                    cursor_pos.0 += 1;
                    editor.set_cursor_pos(cursor_pos).ok();
                }
                Input::Character(c) => {
                    if c == '\x1b' {
                        editor.set_mode(Mode::Selection);
                    } else if c == 'i' && editor.get_mode() == Mode::Selection {
                        editor.set_mode(Mode::Insertion);
                    } else if c == 'q' && editor.get_mode() == Mode::Selection {
                        break 'editor;
                    }else if matches!(c, '1'..='8') && editor.get_mode() == Mode::Selection {
                        let color = c as u8 - 48;
                        editor.set_selected_color(Color(color as u32).into());
                    }

                    if editor.get_mode() == Mode::Insertion {
                        match c.to_ascii_lowercase() {
                            'a' => {
                                cursor_pos.1 -= 1;
                                editor.set_cursor_pos(cursor_pos).ok(); //using the result of this function might end up in unncessary crashes
                                editor.set_pix_at_cursor(editor.selected_color)?;
                            }
                            'd' => {
                                cursor_pos.1 += 1;
                                editor.set_cursor_pos(cursor_pos).ok();
                                editor.set_pix_at_cursor(editor.selected_color)?;
                            }
                            'w' => {
                                cursor_pos.0 -= 1;
                                editor.set_cursor_pos(cursor_pos).ok();
                                editor.set_pix_at_cursor(editor.selected_color)?;
                            }
                            's' => {
                                cursor_pos.0 += 1;
                                editor.set_cursor_pos(cursor_pos).ok();
                                editor.set_pix_at_cursor(editor.selected_color)?;
                            }
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        }
    }
    endwin();
    let mut file = OpenOptions::new().truncate(true).create(true).write(true).open(args.file)?;
    file.write(&editor.tif_image.save())?;
    Ok(())
}
