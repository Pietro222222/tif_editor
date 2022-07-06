use libtif::pixel::PixelColor;

pub struct Pallete {
    pub colors: Vec<PixelColor>
}

impl Pallete {
    pub fn new() -> Self {
        use PixelColor::*;
        Self {
            colors: vec![Black, Red, Green, Yellow, Blue, Magenta, Cyan, White],
        }
    }
}
