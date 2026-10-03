//! Processing's `noise()`: value noise over a table of 4096 random values with
//! cosine smoothing, summed over octaves. A port of `PApplet.noise`, including
//! its half-degree cosine table, so sketches look the way they do in Processing.

use rand::rngs::StdRng;
use rand::{RngExt, SeedableRng};

const YWRAPB: u32 = 4;
const YWRAP: u32 = 1 << YWRAPB;
const ZWRAPB: u32 = 8;
const ZWRAP: u32 = 1 << ZWRAPB;
const SIZE: u32 = 4095;

pub struct Noise {
    table: Vec<f32>,
    octaves: u32,
    falloff: f32,
}

impl Noise {
    pub fn new(seed: u64) -> Self {
        let mut rng = StdRng::seed_from_u64(seed);
        Self {
            table: (0..=SIZE).map(|_| rng.random::<f32>()).collect(),
            octaves: 4,
            falloff: 0.5,
        }
    }

    /// Reseeds the table, keeping the detail settings.
    pub fn seed(&mut self, seed: u64) {
        let mut rng = StdRng::seed_from_u64(seed);
        self.table.iter_mut().for_each(|v| *v = rng.random());
    }

    /// Non-positive values leave the current setting unchanged, as in Processing.
    pub fn detail(&mut self, octaves: i32, falloff: Option<f32>) {
        if octaves > 0 {
            self.octaves = octaves as u32;
        }
        if let Some(falloff) = falloff.filter(|f| *f > 0.0) {
            self.falloff = falloff;
        }
    }

    pub fn get(&self, x: f32, y: f32, z: f32) -> f32 {
        let (x, y, z) = (x.abs(), y.abs(), z.abs());
        let (mut xi, mut yi, mut zi) = (x as u32, y as u32, z as u32);
        let (mut xf, mut yf, mut zf) = (x - xi as f32, y - yi as f32, z - zi as f32);
        let at = |i: u32| self.table[(i & SIZE) as usize];

        let mut r = 0.0;
        let mut amplitude = 0.5;
        for _ in 0..self.octaves {
            let mut of = xi
                .wrapping_add(yi.wrapping_shl(YWRAPB))
                .wrapping_add(zi.wrapping_shl(ZWRAPB));
            let rxf = smooth(xf);
            let ryf = smooth(yf);

            let mut n1 = at(of);
            n1 += rxf * (at(of.wrapping_add(1)) - n1);
            let mut n2 = at(of.wrapping_add(YWRAP));
            n2 += rxf * (at(of.wrapping_add(YWRAP + 1)) - n2);
            n1 += ryf * (n2 - n1);

            of = of.wrapping_add(ZWRAP);
            n2 = at(of);
            n2 += rxf * (at(of.wrapping_add(1)) - n2);
            let mut n3 = at(of.wrapping_add(YWRAP));
            n3 += rxf * (at(of.wrapping_add(YWRAP + 1)) - n3);
            n2 += ryf * (n3 - n2);

            n1 += smooth(zf) * (n2 - n1);

            r += n1 * amplitude;
            amplitude *= self.falloff;
            (xi, yi, zi) = (xi.wrapping_shl(1), yi.wrapping_shl(1), zi.wrapping_shl(1));
            (xf, yf, zf) = (xf * 2.0, yf * 2.0, zf * 2.0);
            if xf >= 1.0 {
                xi = xi.wrapping_add(1);
                xf -= 1.0;
            }
            if yf >= 1.0 {
                yi = yi.wrapping_add(1);
                yf -= 1.0;
            }
            if zf >= 1.0 {
                zi = zi.wrapping_add(1);
                zf -= 1.0;
            }
        }
        r
    }
}

/// `0.5 * (1 - cos(t * PI))`, quantized to half a degree like Processing's `cosLUT`.
fn smooth(t: f32) -> f32 {
    let half_degrees = (t * 360.0) as u32 % 720;
    0.5 * (1.0 - (half_degrees as f32 * 0.5).to_radians().cos())
}
