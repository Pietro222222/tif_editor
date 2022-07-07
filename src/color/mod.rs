use std::ops::Deref;

use anyhow::anyhow;
use anyhow::Result;
use libtif::pixel::PixelColor;
use pancurses::COLOR_BLUE;
use pancurses::COLOR_WHITE;
use pancurses::Window;
use pancurses::curs_set;
use pancurses::has_colors;
use pancurses::init_pair;
use pancurses::noecho;
use pancurses::raw;
use pancurses::start_color;

pub struct Color(pub u32);

impl Deref for Color {
    type Target = u32;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<&PixelColor> for Color {
    fn from(px: &PixelColor) -> Self {
        Color (px.as_u8().into())
    }
}

impl Into<PixelColor> for Color {
    fn into(self) -> PixelColor {
        PixelColor::from(match *self -1 {
            0 => { 0x5b  },
            1 => {
                0x5c
            },
            2 => {
                0x5d
            },
            3 => {
                0x60
            },
            4 => {
                0x5a
            },
            5 => {
                0x5e
            },
            6 => {
                0x61
            },
            7 => {
                0x5f
            }
            _ => {
                0x5a
            }
        })
    }
}

pub fn set_editor_up(w: &Window) -> Result<()> {
    set_up_colors()?;
    curs_set(0);
    w.keypad(true);
    noecho();
    raw();
    w.nodelay(true);
    Ok(())
}


fn set_up_colors() -> Result<()> {
    if !has_colors() {
        return Err(anyhow!("colors arent supported"));
    }
    start_color();
    init_pair(0, 0, 0); //black
    init_pair(1, 1, 1); //red
    init_pair(2, 2, 2); //green
    init_pair(3, 3, 3); //yellow
    init_pair(4, 4, 4); //blue
    init_pair(5, 5, 5); //magenta
    init_pair(6, 6, 6); //cyan
    init_pair(7, 7, 7); //white
    init_pair(8, 0, 7);
    init_pair(9, COLOR_BLUE, COLOR_WHITE); //cursor
    Ok(())
}
