use processing::prelude::*;

const MODES: [BlendMode; 10] = [
    BlendMode::Blend,
    BlendMode::Add,
    BlendMode::Subtract,
    BlendMode::Darkest,
    BlendMode::Lightest,
    BlendMode::Difference,
    BlendMode::Exclusion,
    BlendMode::Multiply,
    BlendMode::Screen,
    BlendMode::Replace,
];

struct BlendModes {
    index: usize,
}

impl Sketch for BlendModes {
    fn settings() -> Settings {
        Settings::size(500, 500)
    }

    fn setup(_g: &mut Graphics) -> Result<Self> {
        Ok(BlendModes { index: 0 })
    }

    fn draw(&mut self, g: &mut Graphics) {
        if g.key_just_pressed(Key::Right) || g.key_just_pressed(Key::Space) {
            self.index = (self.index + 1) % MODES.len();
            eprintln!("{}", MODES[self.index].name());
        } else if g.key_just_pressed(Key::Left) {
            self.index = (self.index + MODES.len() - 1) % MODES.len();
            eprintln!("{}", MODES[self.index].name());
        }

        g.background((0.15, 0.15, 0.15));
        g.no_stroke();
        g.blend_mode(MODES[self.index]);

        g.fill((0.9, 0.2, 0.2, 0.75));
        g.rect(80, 100, 200, 250);

        g.fill((0.2, 0.8, 0.2, 0.75));
        g.rect(180, 80, 200, 250);

        g.fill((0.2, 0.3, 0.9, 0.75));
        g.rect(130, 200, 200, 200);
    }
}

fn main() -> Result<()> {
    processing::run::<BlendModes>()
}
