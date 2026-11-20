use processing::prelude::*;

struct WebcamSketch {
    camera: Webcam,
}

impl Sketch for WebcamSketch {
    fn settings() -> Settings {
        Settings::size(640, 480)
    }

    fn setup(_g: &mut Graphics) -> Result<Self> {
        let camera = Webcam::new()?;
        Ok(WebcamSketch { camera })
    }

    fn draw(&mut self, g: &mut Graphics) {
        if self.camera.connected() {
            g.background_image(&self.camera);
        }
    }
}

fn main() -> Result<()> {
    processing::run::<WebcamSketch>()
}
