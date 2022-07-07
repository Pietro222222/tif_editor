use super::color::Color;
use super::pallete::Pallete;
use crate::area::Area;
use crate::area::Point;
use crate::color;
use crate::cursor::Cursor;
use crate::mode::Mode;
use anyhow::anyhow;
use anyhow::Context;
use anyhow::Result;
use libtif::image::TifImage;
use libtif::pixel::PixelColor;
use pancurses::COLOR_PAIR;
use pancurses::{initscr, Window};
use std::ops::Deref;

pub struct Editor {
    pub window: Window,
    pub tif_image: TifImage,
    pub mode: Mode,
    pub selected_color: PixelColor,
    pub cursor: Cursor,
    pub pallete: Pallete,
    pub area: Option<Area>,
}

impl Editor {
    pub fn new(tif_image: TifImage) -> Self {
        Self {
            window: initscr(),
            tif_image,
            mode: Mode::Selection,
            selected_color: PixelColor::Black,
            cursor: Cursor::new(),
            pallete: Pallete::new(),
            area: None,
        }
    }

    fn get_area(&self) -> Option<&Area> {
        self.area.as_ref()
    }

    fn get_mut_area(&mut self) -> Option<&mut Area> {
        self.area.as_mut()
    }

    pub fn get_mode(&self) -> Mode {
        self.mode
    }
    pub fn set_mode(&mut self, m: Mode) {
        if self.mode == Mode::Area && m != Mode::Area {
            self.draw_area_pixels().ok();
            self.cursor.toogle_hidden();
        }
        self.mode = m;
        self.draw_status();
        self.draw_help().ok(); //we dont really need to handle that error
        self.refresh();
    }
    pub fn get_image_size(&self) -> (u64, u8) {
        (self.tif_image.height, self.tif_image.width)
    }
    pub fn get_window_size(&self) -> (i32, i32) {
        self.window.get_max_yx()
    }
    pub fn set_selected_color(&mut self, color: PixelColor) {
        self.selected_color = color;
        self.draw_status();
        self.refresh();
    }

    pub fn area_mode(&mut self) {
        self.mode = Mode::Area;
        self.area = Some(Area::new(
            Point::new(self.cursor.pos.0, self.cursor.pos.1),
            Point::new(self.cursor.pos.0, self.cursor.pos.1),
        ));
        self.draw_help().ok();
        self.draw_status();
        self.cursor.toogle_hidden();
    }
    pub fn get_pix(&self, pos: &(usize, usize)) -> Option<&PixelColor> {
        let pix = self.tif_image.pixels.get(pos.0)?;
        pix.get(pos.1)
    }

    pub fn get_mut_pix(&mut self, pos: &(usize, usize)) -> Option<&mut PixelColor> {
        let pix = self.tif_image.pixels.get_mut(pos.0)?;
        pix.get_mut(pos.1)
    }

    pub fn redraw_pix(&self, pos: (usize, usize)) -> Result<()> {
        let pix = self.get_pix(&pos).context("out of bounds")?;

        let color = Color::from(pix);
        self.attrset(COLOR_PAIR(*color));
        self.mvaddch(pos.0 as i32, pos.1 as i32, ' ');
        self.attroff(COLOR_PAIR(*color));
        Ok(())
    }
    pub fn set_cursor_pos(&mut self, pos: (i32, i32)) -> Result<()> {
        let image = self.get_image_size();
        if !(pos.0 >= 0 && pos.0 <= image.0 as i32 - 1 && pos.1 >= 0 && pos.1 <= image.1 as i32 - 1)
        {
            return Err(anyhow!("out of image bounds"));
        }
        self.redraw_pix(self.cursor.coord_as_usize())?;
        self.cursor.set_pos(pos);
        self.cursor.draw(&*self);
        self.refresh();
        Ok(())
    }

    pub fn set_pix_at_cursor(&mut self, color: PixelColor) -> Result<()> {
        self.set_pix_color(self.cursor.coord_as_usize(), color)?;
        self.cursor.draw(&*self);
        self.refresh();
        Ok(())
    }

    fn set_pix_color(&mut self, pos: (usize, usize), color: PixelColor) -> Result<()> {
        let pix = self.get_mut_pix(&pos).context("out of bounds")?;
        *pix = color;
        self.redraw_pix(pos)?;
        self.refresh();
        Ok(())
    }

    pub fn is_terminal_size_enough(&self) -> Result<()> {
        let term = self.get_window_size();
        let image = self.get_image_size();
        if term.0 < image.0 as i32 + 9 {
            return Err(anyhow!("terminal's height is too small"));
        }
        if term.1 < image.1 as i32 + 10 {
            return Err(anyhow!("terminal's width is too small"));
        }

        Ok(())
    }

    fn draw_image(&self) {
        for (height, pixels) in self.tif_image.pixels.iter().enumerate() {
            for (width, pixel) in pixels.iter().enumerate() {
                self.attrset(COLOR_PAIR(*Color::from(pixel)));
                self.mvaddch(height as i32, width as i32, ' ');
                self.attrset(COLOR_PAIR(*Color::from(pixel)));
            }
        }
    }
    fn draw_border(&self) {
        let x_pos = self.tif_image.width;
        let y_pos = self.tif_image.height;
        self.attrset(COLOR_PAIR(*Color::from(&PixelColor::Red)));
        self.mvprintw(y_pos as i32, 0, String::from(" ").repeat(x_pos as usize));
        for i in 0..y_pos {
            self.mvaddch(i as i32, x_pos as i32, ' ');
        }
        self.attroff(COLOR_PAIR(*Color::from(&PixelColor::Red)));
    }
    fn draw_color_pallete(&self) {
        let y_pos = self.tif_image.height + 3;
        let mut pos = 0;
        for px in 0..8 {
            self.attrset(COLOR_PAIR(px));
            self.mvprintw(y_pos as i32, pos, "  ");
            self.attroff(COLOR_PAIR(px));
            self.mvaddch(y_pos as i32 + 1, pos + 1, (px + 49) as u8 as char);
            pos += 2;
        }
    }

    fn get_area_positions(&self) -> Result<((i32, i32), (i32, i32))> {
        if self.mode != Mode::Area {
            return Err(anyhow!("impossible to draw area. not in Area mode"));
        }
        let area = self.get_area().unwrap();
        let area_x = if area.starting_point.x >= area.final_point.x {
            (area.final_point.x, area.starting_point.x)
        } else {
            (area.starting_point.x, area.final_point.x)
        };
        let area_y = if area.starting_point.y >= area.final_point.y {
            (area.final_point.y, area.starting_point.y)
        } else {
            (area.starting_point.y, area.final_point.y)
        };
        Ok((area_y, area_x))
    }
    ///redraw all the pixels that were covered by the area selection
    pub fn draw_area_pixels(&self) -> Result<()> {
        let area_position = self.get_area_positions()?;
        for i in (area_position.0 .0)..=(area_position.0 .1) {
            for j in area_position.1 .0..=area_position.1 .1 {
                self.redraw_pix((i as usize, j as usize))?;
            }
        }
        Ok(())
    }

    pub fn draw_area(&self) -> Result<()> {
        let area_position = self.get_area_positions()?;
        self.attrset(COLOR_PAIR(9));
        for i in (area_position.0 .0)..=(area_position.0 .1) {
            for j in area_position.1 .0..=area_position.1 .1 {
                if (i >= 0 && i <= self.tif_image.height as i32 - 1)
                    && j >= 0
                    && j <= self.tif_image.width as i32 - 1
                {
                    self.mvaddch(i, j, '#');
                }
            }
        }
        self.attroff(COLOR_PAIR(9));
        self.refresh();
        Ok(())
    }
    ///set the color of the pixels selected in the area
    pub fn set_area_color(&mut self, color: PixelColor) -> Result<()> {
        let area_position = self.get_area_positions()?;
        for i in (area_position.0 .0)..=(area_position.0 .1) {
            for j in area_position.1 .0..=area_position.1 .1 {
                if (i >= 0 && i <= self.tif_image.height as i32 - 1)
                    && j >= 0
                    && j <= self.tif_image.width as i32 - 1
                {
                    self.set_pix_color((i as usize, j as usize), color)?;
                }
            }
        }
        Ok(())
    }

    fn set_area_pos(&mut self, pos: (i32, i32)) -> Result<()> {
        if self.mode != Mode::Area {
            return Err(anyhow!("impossible to draw area. not in Area mode"));
        }

        let point = self.get_area().unwrap().final_point;
        if point.x >= pos.1 || point.y >= pos.0 {
            self.draw_area_pixels()?;
        }

        self.get_mut_area().unwrap().set_final_point_pos(pos);
        self.draw_area()?;
        Ok(())
    }

    pub fn set_area_based_on_current_cursor_position(&mut self) -> Result<()> {
        self.set_area_pos(self.cursor.pos)
    }
    fn draw_status(&self) {
        let pos_y = self.tif_image.height as i32 + 7;
        self.mvprintw(
            pos_y,
            0,
            format!("MODE: {:?}               ", self.get_mode()),
        );
        self.mvprintw(
            pos_y + 1,
            0,
            format!("CURRENT COLOR: {:?}         ", self.selected_color),
        );
    }

    fn draw_cursor(&self) {
        self.cursor.draw(&*self);
    }

    pub fn draw_ui(&self) -> Result<()> {
        color::set_editor_up(&*self)?;
        self.draw_image();
        self.draw_border();
        self.draw_color_pallete();
        self.draw_cursor();
        self.draw_status();
        self.refresh();

        Ok(())
    }

    pub fn draw_help(&self) -> Result<()> {
        if self.tif_image.width as i32 + 50 > self.get_window_size().1 {
            return Err(anyhow!("window is too small"));
        }
        for j in 0..10 {
            for i in self.tif_image.width as i32 + 10..self.tif_image.width as i32 + 50 {
                self.mvaddch(j, i, ' ');
            }
        }
        match self.mode {
            Mode::Selection => {
                self.mvprintw(
                    1,
                    self.tif_image.width as i32 + 10,
                    "[I] -> insertion mode".to_uppercase(),
                );
                self.mvprintw(
                    3,
                    self.tif_image.width as i32 + 10,
                    "[S] -> Area Mode".to_uppercase(),
                );
                self.mvprintw(
                    5,
                    self.tif_image.width as i32 + 10,
                    "[1..8] -> select color".to_uppercase(),
                );
            }
            Mode::Insertion => {
                self.mvprintw(
                    1,
                    self.tif_image.width as i32 + 10,
                    "[ESC] -> selection mode".to_uppercase(),
                );
                self.mvprintw(
                    3,
                    self.tif_image.width as i32 + 10,
                    "[SPACE] -> paint the area".to_uppercase(),
                );
                self.mvprintw(
                    5,
                    self.tif_image.width as i32 + 10,
                    "[⬅ ➡ ⬆ ⬇] -> move without painting".to_uppercase(),
                );
                self.mvprintw(
                    7,
                    self.tif_image.width as i32 + 10,
                    "[wasd] -> move and paint".to_uppercase(),
                );
            }
            Mode::Area => {
                self.mvprintw(
                    1,
                    self.tif_image.width as i32 + 10,
                    "[ESC] -> selection mode".to_uppercase(),
                );
                self.mvprintw(
                    3,
                    self.tif_image.width as i32 + 10,
                    "[SPACE] -> draw in the selected area".to_uppercase(),
                );
                self.mvprintw(
                    5,
                    self.tif_image.width as i32 + 10,
                    "[WASD] -> move and sellect".to_uppercase(),
                );

            }
        }
        Ok(())
    }
}

impl Deref for Editor {
    type Target = Window;
    fn deref(&self) -> &Self::Target {
        &self.window
    }
}
