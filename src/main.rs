use grid::{Grid, GridMode};
use pancurses::{
    curs_set, endwin, has_colors, init_pair, initscr, napms, noecho, raw, start_color, Input,
    Window,
};
use std::{env, process::exit};
mod grid;
mod pixel;
use pixel::Color;

fn init() -> Window {
    let mut window = initscr();
    window.keypad(true);
    noecho();
    raw();
    window.nodelay(true);

    /*INIT COLORS*/
    if !has_colors() {
        println!("YOUR TERMINAL DOESNT SUPPORT COLORS!");
        exit(0);
    }
    start_color();
    curs_set(0);
    init_pair(1, 0, 0); //black
    init_pair(2, 1, 1); //red
    init_pair(3, 2, 2); //green
    init_pair(4, 3, 3); //yellow
    init_pair(5, 4, 4); //blue
    init_pair(6, 5, 5); //magenta
    init_pair(7, 6, 6); //cyan
    init_pair(8, 7, 7); //white
    init_pair(9, 0, 7);
    return window;
}

fn parse_args() -> (i16, i16) {
    let (mut y, mut x) = (50, 50);
    let args = env::args().collect::<Vec<String>>();
    if args.len() >= 3 {
        y = args[1].parse::<i16>().expect("COULD NOT PARSE NUMBER!");
        x = args[2].parse::<i16>().expect("COULD NOT PARSE NUMBER!");
    }
    (y, x)
}

fn main() {
    let window = init();
    let args = parse_args();
    if args.0 as i32 + 7 > window.get_max_y() || (args.1 as i32 > window.get_max_x() && args.1 > 16)
    {
        println!("YOUR TERMINAL IS TOO SMALL!");
        return;
    }

    let mut grid = Grid::new(args);
    'mainloop: loop {
        if let Some(ch) = window.getch() {
            match ch {
                Input::KeyLeft => grid.dec_select_width(),
                Input::KeyRight => grid.inc_select_width(),
                Input::KeyUp => grid.dec_select_height(),
                Input::KeyDown => grid.inc_select_height(),
                Input::Character(c) => {
                    if c == 'q' || c == 'Q' {
                        break 'mainloop;
                    } else if c == 'i' {
                        grid.mode = GridMode::Editing;
                    } else if c == '\x1b' {
                        grid.mode = GridMode::Selection;
                    } else if c == ' ' && grid.mode == GridMode::Editing {
                        grid.set_selected_color_in_pixel(&window);
                    } else if c == 'w' || c == 'W' {
                        grid.dec_select_height();
                        if grid.mode == GridMode::Editing {
                            grid.set_selected_color_in_pixel(&window);
                        }
                    } else if c == 's' || c == 'S' {
                        grid.inc_select_height();
                        if grid.mode == GridMode::Editing {
                            grid.set_selected_color_in_pixel(&window);
                        }
                    } else if c == 'a' || c == 'D' {
                        grid.dec_select_width();
                        if grid.mode == GridMode::Editing {
                            grid.set_selected_color_in_pixel(&window);
                        }
                    } else if c == 'd' || c == 'D' {
                        grid.inc_select_width();
                        if grid.mode == GridMode::Editing {
                            grid.set_selected_color_in_pixel(&window);
                        }
                    }
                    if let Ok(num) = c.to_string().parse::<u8>() {
                        if grid.mode == GridMode::Selection {
                            let new_color = match num {
                                1 => Color::Black,
                                2 => Color::Red,
                                3 => Color::Green,
                                4 => Color::Yellow,
                                5 => Color::Blue,
                                6 => Color::Magenta,
                                7 => Color::Cyan,
                                8 => Color::White,
                                _ => grid.selected_color,
                            };
                            grid.selected_color = new_color;
                        }
                    }
                }
                _ => {}
            }
        }
        grid.draw(&window);
        grid.draw_colors(&window);
        grid.draw_selected_color(&window);
        grid.draw_current_mode(&window);
        napms(20);
    }
    grid.save_to_file("image.tif".into());
    endwin();
}
