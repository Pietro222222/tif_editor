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
mod area;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long, value_parser)]
    file: String,

    #[clap(short, long, value_parser)]
    create: bool,

    #[clap(short, long, value_parser, default_value_t=1)]
    height: i32,

    #[clap(short, long, value_parser, default_value_t=1)]
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
    editor.draw_ui()?;
    editor.draw_help().ok(); //dont handle this error
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
                    }else if matches!(c, '0'..='8') && editor.get_mode() == Mode::Selection {
                        let color = c as u8 - 48;
                        editor.set_selected_color(Color(color as u32).into());
                    }else if c == ' ' && editor.get_mode() == Mode::Insertion {
                        editor.set_pix_at_cursor(editor.selected_color)?;
                    }else if c == 's' && editor.get_mode() == Mode::Selection {
                        editor.area_mode();
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
                    }else if editor.get_mode() == Mode::Area {
                        match c.to_ascii_lowercase() {
                            'a' => {
                                cursor_pos.1 -= 1;
                                editor.set_cursor_pos(cursor_pos).ok(); //using the result of this function might end up in unncessary crashes
                                editor.set_area_based_on_current_cursor_position()?;
                                editor.draw_area()?;
                            }
                            'd' => {
                                cursor_pos.1 += 1;
                                editor.set_cursor_pos(cursor_pos).ok();
                                editor.set_area_based_on_current_cursor_position()?;
                                editor.draw_area()?;
                            }
                            'w' => {
                                cursor_pos.0 -= 1;
                                editor.set_cursor_pos(cursor_pos).ok();
                                editor.set_area_based_on_current_cursor_position()?;
                                editor.draw_area()?;
                            }
                            's' => {
                                cursor_pos.0 += 1;
                                editor.set_cursor_pos(cursor_pos).ok();
                                editor.set_area_based_on_current_cursor_position()?;
                                editor.draw_area()?;
                            },
                            ' ' => {
                                editor.set_area_color(editor.selected_color)?;
                                editor.set_mode(Mode::Selection);
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
