#[derive(Copy, Clone, Debug)]
pub struct Point {
    pub y: i32,
    pub x: i32,
}

impl Point {
    pub fn new(y: i32, x: i32) -> Self {
        Self { x, y }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Area {
    pub starting_point: Point,
    pub final_point: Point
}

impl Area {
    pub fn new(sp: Point, fp: Point) -> Self {
        Self {
            starting_point: sp,
            final_point: fp
        }
    }
    pub fn set_final_point_pos(&mut self, pos: (i32, i32)) {
        self.final_point.y = pos.0;
        self.final_point.x = pos.1;
    }
}
