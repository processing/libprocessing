use processing::prelude::*;

struct BackgroundImage {
    logo: Image,
}

impl Sketch for BackgroundImage {
    fn settings() -> Settings {
        Settings::size(400, 400)
    }

    fn setup(_g: &mut Graphics) -> Result<Self> {
        let logo = Image::load("images/logo.png")?;
        Ok(BackgroundImage { logo })
    }

    fn draw(&mut self, g: &mut Graphics) {
        g.background_image(&self.logo);
    }
}

fn main() -> Result<()> {
    processing::run::<BackgroundImage>()
}
