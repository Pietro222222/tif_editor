use std::{fs::OpenOptions, io::Write};

use pancurses::{Window, COLOR_PAIR};

use crate::pixel::*;

#[derive(PartialEq, Eq, Debug)]
pub enum GridMode {
    Selection,
    Editing,
}

pub struct Grid {
    pub pixel: Vec<Vec<Pixel>>,
    pub height: i16,
    pub width: i16,
    pub currently_selected: (i16, i16),
    pub mode: GridMode,
    pub selected_color: Color,
}

impl Grid {
    pub fn new(size: (i16, i16)) -> Self {
        let mut vec: Vec<Vec<Pixel>> = vec![];
        for i in 0..(size.0) {
            vec.push(vec![]);
            for j in 0..(size.1) {
                let mut v = vec.get_mut(i as usize).unwrap();
                v.push(Pixel::new((i, j)));
            }
        }
        Self {
            pixel: vec,
            height: size.0,
            width: size.1,
            currently_selected: (0, 0),
            mode: GridMode::Selection,
            selected_color: Color::Black,
        }
    }
    pub fn draw(&self, w: &Window) {
        for i in &self.pixel {
            for j in i {
                j.draw_on_screen(w);
                if j.height == self.currently_selected.0 && j.width == self.currently_selected.1 {
                    w.mv(j.height as i32, j.width as i32);
                    w.attrset(COLOR_PAIR(9));
                    w.addch(' ');
                    w.attroff(COLOR_PAIR(9));
                }
            }
        }
        w.refresh();
    }

    pub fn inc_select_width(&mut self) {
        if self.currently_selected.1 < self.width - 1 {
            self.set_cursor_pos((self.currently_selected.0, self.currently_selected.1 + 1));
        }
    }
    pub fn dec_select_width(&mut self) {
        if self.currently_selected.1 > 0 {
            self.set_cursor_pos((self.currently_selected.0, self.currently_selected.1 - 1));
        }
    }
    pub fn inc_select_height(&mut self) {
        if self.currently_selected.0 < self.height - 1 {
            self.set_cursor_pos((self.currently_selected.0 + 1, self.currently_selected.1));
        }
    }
    pub fn dec_select_height(&mut self) {
        if self.currently_selected.0 > 0 {
            self.set_cursor_pos((self.currently_selected.0 - 1, self.currently_selected.1));
        }
    }

    pub fn set_cursor_pos(&mut self, new_pos: (i16, i16)) {
        self.currently_selected = new_pos;
    }
    pub fn change_color(&mut self, pos: (i16, i16), color: Color, w: &Window) {
        let v = self.pixel.get_mut(pos.0 as usize).unwrap();
        let pixel = v.get_mut(pos.1 as usize).unwrap();
        pixel.set_color(color);
        pixel.draw_on_screen(w);
        w.refresh();
    }

    pub fn draw_colors(&self, window: &Window) {
        use strum::IntoEnumIterator;
        let mut position = 0;
        for i in Color::iter() {
            window.mv(self.height as i32 + 3, i.as_u8() as i32 + position);
            window.attrset(COLOR_PAIR(i.as_u8() as u32 + 1));
            window.printw("  ");
            window.attroff(COLOR_PAIR(i.as_u8() as u32 + 1));
            window.mvprintw(
                self.height as i32 + 4,
                i.as_u8() as i32 + position,
                format!(" {}", i.as_u8() + 1),
            );
            position += 2;
        }
        window.refresh();
    }
    pub fn draw_selected_color(&self, window: &Window) {
        window.mvprintw(
            self.height as i32 + 6,
            0,
            format!("color: {:?}       ", self.selected_color),
        );
        window.refresh();
    }
    pub fn change_current_selected_color(&mut self, color: Color) {
        self.selected_color = color;
    }
    pub fn set_selected_color_in_pixel(&mut self, window: &Window) {
        self.change_color(self.currently_selected, self.selected_color, window);
    }
    pub fn draw_current_mode(&self, window: &Window) {
        window.mvprintw(
            self.height as i32 + 7,
            0,
            format!("mode: {:?}       ", self.mode),
        );
    }

    pub fn get_pixel_at(&self, pos: (i16, i16)) -> &Pixel {
        let v = self.pixel.get(pos.0 as usize).unwrap();
        let pixel = v.get(pos.1 as usize).unwrap();
        return pixel;
    }

    pub fn get_header(&self) -> Vec<u8> {
        vec![0x2e, 0x54, 0x49, 0x46, 0x20, self.width as u8]
    }
    pub fn append_pixels<'a>(&self, buffer: &'a mut Vec<u8>) {
        let mut current_color = Color::Black;
        let mut pixels = 0;

        for i in 0..self.height {
            for j in 0..self.width {
                let pix = self.get_pixel_at((i, j));
                if pix.color == current_color {
                    pixels += 1;
                    if pixels >= 255 {
                        buffer.push(current_color.to_tif_color());
                        buffer.push(pixels);
                        pixels = 0;
                    }
                } else {
                    if pixels > 0 {
                        buffer.push(current_color.to_tif_color());
                        buffer.push(pixels);
                    }
                    current_color = pix.color;
                    pixels = 1;
                }
            }
        }
        if pixels > 0 {
            buffer.push(current_color.to_tif_color());
            buffer.push(pixels);
        }
    }
    pub fn save_to_file(&self, filename: String) {
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .append(false)
            .open(filename)
            .unwrap();
        let mut buffer = self.get_header();
        self.append_pixels(&mut buffer);
        file.write_all(&buffer);
    }
}
