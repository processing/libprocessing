use processing::{Graphics, Result, Settings, Sketch, TextAlignH, TextAlignV};

struct Text;

impl Sketch for Text {
    fn settings() -> Settings {
        Settings::size(600, 400)
    }

    fn setup(_g: &mut Graphics) -> Result<Self> {
        Ok(Text)
    }

    fn draw(&mut self, g: &mut Graphics) {
        g.background(255);
        g.fill(0);

        g.text_size(32);
        g.text("Hello, Processing!", 50, 100);

        g.text_size(18);
        g.text("Text rendering with parley + skrifa + lyon", 50, 160);

        g.text_size(16);
        g.text(
            "This is a longer paragraph of text that should wrap within its bounding box. \
             The text uses parley for layout, skrifa for glyph outlines, and lyon for tessellation.",
            50,
            220,
        )
        .bounds(300, 200);

        g.text_align((TextAlignH::Center, TextAlignV::Top));
        g.text_size(24);
        g.text("Centered", 450, 100).max_width(200);
    }
}

fn main() -> Result<()> {
    processing::run::<Text>()
}
