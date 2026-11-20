use processing::prelude::*;

struct Input;

impl Sketch for Input {
    fn settings() -> Settings {
        Settings::size(400, 400)
    }

    fn setup(_g: &mut Graphics) -> Result<Self> {
        Ok(Input)
    }

    fn draw(&mut self, g: &mut Graphics) {
        g.background((0.15, 0.15, 0.2));

        if g.mouse_pressed() {
            g.fill((0.55, 0.25, 0.2));
        } else {
            g.fill(255);
        }
        g.rect(g.mouse_x() - 25.0, g.mouse_y() - 25.0, 50, 50);

        if g.key_down(Key::Escape) {
            g.exit();
        }
    }
}

fn main() -> Result<()> {
    processing::run::<Input>()
}
