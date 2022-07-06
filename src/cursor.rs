use pancurses::{Window, COLOR_PAIR};

pub struct Cursor {
    pub pos: (i32, i32),
}

impl Cursor {
    pub fn new() -> Self {
        Self {
            pos: (0, 0)
        }
    }
    pub fn coord_as_usize(&self) -> (usize, usize) {
        (self.pos.0 as usize, self.pos.1 as usize)
    }
    pub fn draw(&self, w: &Window) {
        w.attrset(COLOR_PAIR(9));
        w.mvaddch(self.pos.0, self.pos.1, '#');
        w.attroff(COLOR_PAIR(9));
    }
    pub fn set_pos(&mut self, pos: (i32, i32)) {
        self.pos = pos;
    }
}
