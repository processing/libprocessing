//! Draw-on-drop builders.

use std::ops::{Deref, DerefMut};

use bevy::prelude::{Entity, Transform, Vec3};
use processing_render::geometry::Topology;
use processing_render::graphics_immediate_light;
use processing_render::render::command::{ArcMode, DrawCommand, LightKind};

use crate::graphics::expect;
use crate::{Geometry, Graphics, IntoPoint, Scalar};

pub struct Rect<'g> {
    g: &'g mut Graphics,
    rect: [f32; 4],
    radii: [f32; 4],
}

impl<'g> Rect<'g> {
    pub(crate) fn new(g: &'g mut Graphics, x: f32, y: f32, w: f32, h: f32) -> Self {
        Self {
            g,
            rect: [x, y, w, h],
            radii: [0.0; 4],
        }
    }

    pub fn radius(mut self, r: impl Scalar) -> Self {
        self.radii = [r.to_f32(); 4];
        self
    }

    /// Top-left, top-right, bottom-right, bottom-left.
    pub fn radii(
        mut self,
        tl: impl Scalar,
        tr: impl Scalar,
        br: impl Scalar,
        bl: impl Scalar,
    ) -> Self {
        self.radii = [tl.to_f32(), tr.to_f32(), br.to_f32(), bl.to_f32()];
        self
    }
}

impl Drop for Rect<'_> {
    fn drop(&mut self) {
        let [x, y, w, h] = self.rect;
        self.g.record(DrawCommand::Rect {
            x,
            y,
            w,
            h,
            radii: self.radii,
        });
    }
}

pub struct Arc<'g> {
    g: &'g mut Graphics,
    bounds: [f32; 4],
    start: f32,
    stop: f32,
    mode: ArcMode,
}

impl<'g> Arc<'g> {
    pub(crate) fn new(g: &'g mut Graphics, bounds: [f32; 4], start: f32, stop: f32) -> Self {
        Self {
            g,
            bounds,
            start,
            stop,
            mode: ArcMode::default(),
        }
    }

    pub fn mode(mut self, mode: ArcMode) -> Self {
        self.mode = mode;
        self
    }
}

impl Drop for Arc<'_> {
    fn drop(&mut self) {
        let [cx, cy, w, h] = self.bounds;
        self.g.record(DrawCommand::Arc {
            cx,
            cy,
            w,
            h,
            start: self.start,
            stop: self.stop,
            mode: self.mode,
        });
    }
}

pub struct LightDraw<'g> {
    g: &'g mut Graphics,
    kind: LightKind,
    transform: Transform,
}

impl<'g> LightDraw<'g> {
    pub(crate) fn new(g: &'g mut Graphics, kind: LightKind) -> Self {
        Self {
            g,
            kind,
            transform: Transform::default(),
        }
    }

    pub fn position(mut self, position: impl IntoPoint) -> Self {
        self.transform.translation = position.into_point();
        self
    }

    pub fn look_at(mut self, target: impl IntoPoint) -> Self {
        self.transform.look_at(target.into_point(), Vec3::Y);
        self
    }
}

impl Drop for LightDraw<'_> {
    fn drop(&mut self) {
        expect(graphics_immediate_light(
            self.g.entity,
            self.kind,
            self.transform,
        ));
    }
}

pub struct ParticlesDraw<'g> {
    g: &'g mut Graphics,
    particles: Entity,
    geometry: Option<Entity>,
    topology: Topology,
}

impl<'g> ParticlesDraw<'g> {
    pub(crate) fn new(g: &'g mut Graphics, particles: Entity) -> Self {
        Self {
            g,
            particles,
            geometry: None,
            topology: Topology::PointList,
        }
    }

    pub fn geometry(mut self, geometry: &Geometry) -> Self {
        self.geometry = Some(geometry.0);
        self
    }

    pub fn topology(mut self, topology: Topology) -> Self {
        self.topology = topology;
        self
    }
}

impl Drop for ParticlesDraw<'_> {
    fn drop(&mut self) {
        self.g.record(DrawCommand::Particles {
            particles: self.particles,
            geometry: self.geometry,
            topology: self.topology,
        });
    }
}

pub struct ImageDraw<'g> {
    g: &'g mut Graphics,
    entity: Entity,
    x: f32,
    y: f32,
    size: Option<[f32; 2]>,
    source: Option<[f32; 4]>,
}

impl<'g> ImageDraw<'g> {
    pub(crate) fn new(g: &'g mut Graphics, entity: Entity, x: f32, y: f32) -> Self {
        Self {
            g,
            entity,
            x,
            y,
            size: None,
            source: None,
        }
    }

    pub fn size(mut self, w: impl Scalar, h: impl Scalar) -> Self {
        self.size = Some([w.to_f32(), h.to_f32()]);
        self
    }

    /// Pixel region of the image.
    pub fn source(
        mut self,
        x: impl Scalar,
        y: impl Scalar,
        w: impl Scalar,
        h: impl Scalar,
    ) -> Self {
        self.source = Some([x.to_f32(), y.to_f32(), w.to_f32(), h.to_f32()]);
        self
    }
}

impl Drop for ImageDraw<'_> {
    fn drop(&mut self) {
        let size = self.size.map(|[w, h]| (w, h));
        let source = self.source.map(|[x, y, w, h]| (x, y, w, h));
        self.g.record(DrawCommand::Image {
            entity: self.entity,
            dx: self.x,
            dy: self.y,
            d_width: size.map(|s| s.0),
            d_height: size.map(|s| s.1),
            sx: source.map(|s| s.0),
            sy: source.map(|s| s.1),
            s_width: source.map(|s| s.2),
            s_height: source.map(|s| s.3),
        });
    }
}

pub struct Text<'g> {
    g: &'g mut Graphics,
    content: String,
    x: f32,
    y: f32,
    z: f32,
    max_w: Option<f32>,
    max_h: Option<f32>,
}

impl<'g> Text<'g> {
    pub(crate) fn new(g: &'g mut Graphics, content: String, x: f32, y: f32) -> Self {
        Self {
            g,
            content,
            x,
            y,
            z: 0.0,
            max_w: None,
            max_h: None,
        }
    }

    pub fn z(mut self, z: impl Scalar) -> Self {
        self.z = z.to_f32();
        self
    }

    pub fn max_width(mut self, w: impl Scalar) -> Self {
        self.max_w = Some(w.to_f32());
        self
    }

    pub fn bounds(mut self, w: impl Scalar, h: impl Scalar) -> Self {
        self.max_w = Some(w.to_f32());
        self.max_h = Some(h.to_f32());
        self
    }
}

impl Drop for Text<'_> {
    fn drop(&mut self) {
        self.g.record(DrawCommand::Text {
            content: std::mem::take(&mut self.content),
            x: self.x,
            y: self.y,
            z: self.z,
            max_w: self.max_w,
            max_h: self.max_h,
        });
    }
}

pub struct Detail<'g, const N: usize> {
    g: &'g mut Graphics,
    size: [f32; 3],
    detail: [u32; N],
    build: fn([f32; 3], [u32; N]) -> DrawCommand,
}

impl<'g, const N: usize> Detail<'g, N> {
    pub(crate) fn new(
        g: &'g mut Graphics,
        size: [f32; 3],
        detail: [u32; N],
        build: fn([f32; 3], [u32; N]) -> DrawCommand,
    ) -> Self {
        Self {
            g,
            size,
            detail,
            build,
        }
    }
}

impl Detail<'_, 1> {
    pub fn detail(mut self, segments: u32) -> Self {
        self.detail = [segments];
        self
    }
}

impl Detail<'_, 2> {
    pub fn detail(mut self, a: u32, b: u32) -> Self {
        self.detail = [a, b];
        self
    }
}

impl<const N: usize> Drop for Detail<'_, N> {
    fn drop(&mut self) {
        self.g.record((self.build)(self.size, self.detail));
    }
}

/// Pops when dropped.
pub struct Push<'g> {
    g: &'g mut Graphics,
}

impl<'g> Push<'g> {
    pub(crate) fn new(g: &'g mut Graphics) -> Self {
        Self { g }
    }
}

impl Deref for Push<'_> {
    type Target = Graphics;

    fn deref(&self) -> &Graphics {
        self.g
    }
}

impl DerefMut for Push<'_> {
    fn deref_mut(&mut self) -> &mut Graphics {
        self.g
    }
}

impl Drop for Push<'_> {
    fn drop(&mut self) {
        self.g.pop_matrix();
        self.g.pop_style();
    }
}

/// Ends drawing when dropped.
pub struct BeginDraw<'g> {
    g: &'g mut Graphics,
}

impl<'g> BeginDraw<'g> {
    pub(crate) fn new(g: &'g mut Graphics) -> Self {
        Self { g }
    }
}

impl Deref for BeginDraw<'_> {
    type Target = Graphics;

    fn deref(&self) -> &Graphics {
        self.g
    }
}

impl DerefMut for BeginDraw<'_> {
    fn deref_mut(&mut self) -> &mut Graphics {
        self.g
    }
}

impl Drop for BeginDraw<'_> {
    fn drop(&mut self) {
        self.g.end_frame();
    }
}
