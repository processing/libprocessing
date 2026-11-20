use processing::prelude::*;

const FILTERS: [Filter; 8] = [
    Filter::Invert,
    Filter::Gray,
    Filter::Threshold(0.5),
    Filter::Posterize(4),
    Filter::Opaque,
    Filter::Erode,
    Filter::Dilate,
    Filter::Blur(4.0),
];

struct Filters;

impl Sketch for Filters {
    fn settings() -> Settings {
        Settings::size(400, 400)
    }

    fn setup(_g: &mut Graphics) -> Result<Self> {
        Ok(Filters)
    }

    fn draw(&mut self, g: &mut Graphics) {
        g.background((0.1, 0.1, 0.15));
        g.no_stroke();

        g.fill((1.0, 0.2, 0.2));
        g.rect(40, 40, 160, 160);
        g.fill((0.2, 1.0, 0.2));
        g.rect(200, 40, 160, 160);
        g.fill((0.2, 0.4, 1.0));
        g.rect(40, 200, 160, 160);
        g.fill((0.95, 0.85, 0.2));
        g.rect(200, 200, 160, 160);

        let seconds = g.millis() / 1000;
        let filter = FILTERS[(seconds / 2) as usize % FILTERS.len()];
        g.filter(filter);

        g.fill((0.0, 0.0, 1.0));
        g.rect(160, 160, 80, 80);
    }
}

fn main() -> Result<()> {
    processing::run::<Filters>()
}
