use pancurses::{Window, COLOR_PAIR};
use strum::IntoEnumIterator; // 0.17.1
use strum_macros::EnumIter; // 0.17.1

#[derive(Clone, Copy, EnumIter, Debug)]
pub enum Color {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
}

impl Color {
    pub fn as_u8(&self) -> u8 {
        *self as u8
    }
    pub fn to_tif_color(&self) -> u8 {
        match *self {
            Color::Black => 0x5b,
            Color::Blue => 0x5a,
            Color::Cyan => 0x61,
            Color::Green => 0x5d,
            Color::Magenta => 0x5e,
            Color::Red => 0x5c,
            Color::White => 0x5f,
            Color::Yellow => 0x60,
        }
    }
}

#[derive(Clone)]
pub struct Pixel {
    is_draw: bool,
    pub color: Color,
    pub height: i16,
    pub width: i16,
}

impl Pixel {
    pub fn new(pos: (i16, i16)) -> Pixel {
        Pixel {
            is_draw: false,
            color: Color::Black,
            height: pos.0,
            width: pos.1,
        }
    }
    pub fn set_color(&mut self, color: Color) {
        self.is_draw = true;
        self.color = color;
    }
    pub fn unset_color(&mut self) {
        self.is_draw = false;
    }

    pub fn draw_on_screen(&self, w: &Window) {
        w.mv(self.height as i32, self.width as i32);

        w.attrset(COLOR_PAIR(self.color.as_u8() as u32 + 1));
        w.addch('#');
        w.attroff(COLOR_PAIR(self.color.as_u8() as u32 + 1));
    }
}
