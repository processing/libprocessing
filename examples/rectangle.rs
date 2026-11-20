use processing::prelude::*;

struct Rectangle;

impl Sketch for Rectangle {
    fn settings() -> Settings {
        Settings::size(400, 400)
    }

    fn setup(_g: &mut Graphics) -> Result<Self> {
        Ok(Rectangle)
    }

    fn draw(&mut self, g: &mut Graphics) {
        g.rect(10, 10, 100, 100);
    }
}

fn main() -> Result<()> {
    processing::run::<Rectangle>()
}
