use processing::{Color, Graphics, Result, Settings, Sketch};

struct UpdatePixels {
    first_frame: bool,
}

impl Sketch for UpdatePixels {
    fn settings() -> Settings {
        Settings::size(100, 100)
    }

    fn setup(_g: &mut Graphics) -> Result<Self> {
        Ok(UpdatePixels { first_frame: true })
    }

    fn draw(&mut self, g: &mut Graphics) {
        g.background(0);

        let red = vec![Color::rgb(1.0, 0.0, 0.0); 10 * 10];
        let blue = vec![Color::rgb(0.0, 0.0, 1.0); 10 * 10];
        g.set_pixels_region(20, 20, 10, 10, &red).unwrap();
        g.set_pixels_region(60, 60, 10, 10, &blue).unwrap();

        if self.first_frame {
            self.first_frame = false;
            print_pixels(g);
        }
    }
}

fn print_pixels(g: &Graphics) {
    let pixels = g.pixels().unwrap();
    let width = g.pixel_width() as usize;
    eprintln!("Total pixels: {}", pixels.len());

    for row in pixels.chunks(width) {
        for pixel in row {
            if pixel.red() > 0.5 {
                eprint!("R");
            } else if pixel.blue() > 0.5 {
                eprint!("B");
            } else if pixel.alpha() > 0.5 {
                eprint!(".");
            } else {
                eprint!(" ");
            }
        }
        eprintln!();
    }

    eprintln!("\nSample pixels:");
    eprintln!("(25, 25): {:?}", pixels[25 * width + 25]);
    eprintln!("(65, 65): {:?}", pixels[65 * width + 65]);
    eprintln!("(0, 0): {:?}", pixels[0]);
    eprintln!("(50, 50): {:?}", pixels[50 * width + 50]);
}

fn main() -> Result<()> {
    processing::run::<UpdatePixels>()
}
