use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

use bevy::prelude::{Entity, Transform};
use processing_render::geometry::Topology;
use processing_render::render::command::{ArcMode, CommandSlot, DrawCommand, LightKind};
use processing_render::{graphics_immediate_light, transform_look_at, transform_set_position};

use crate::graphics::expect;
use crate::{Geometry, Graphics, IntoPoint, Scalar};

pub struct Rect<'g> {
    g: &'g Graphics,
    slot: CommandSlot,
}

impl<'g> Rect<'g> {
    pub(crate) fn new(g: &'g Graphics, x: f32, y: f32, w: f32, h: f32) -> Self {
        let slot = g.record_slot(DrawCommand::Rect {
            x,
            y,
            w,
            h,
            radii: [0.0; 4],
        });
        Self { g, slot }
    }

    pub fn radius(self, r: impl Scalar) -> Self {
        self.set_radii([r.to_f32(); 4])
    }

    /// Top-left, top-right, bottom-right, bottom-left.
    pub fn radii(self, tl: impl Scalar, tr: impl Scalar, br: impl Scalar, bl: impl Scalar) -> Self {
        self.set_radii([tl.to_f32(), tr.to_f32(), br.to_f32(), bl.to_f32()])
    }

    fn set_radii(self, value: [f32; 4]) -> Self {
        self.g.amend(self.slot, |cmd| {
            let DrawCommand::Rect { radii, .. } = cmd else {
                unreachable!()
            };
            *radii = value;
        });
        self
    }
}

pub struct Arc<'g> {
    g: &'g Graphics,
    slot: CommandSlot,
}

impl<'g> Arc<'g> {
    pub(crate) fn new(g: &'g Graphics, bounds: [f32; 4], start: f32, stop: f32) -> Self {
        let [cx, cy, w, h] = bounds;
        let slot = g.record_slot(DrawCommand::Arc {
            cx,
            cy,
            w,
            h,
            start,
            stop,
            mode: ArcMode::default(),
        });
        Self { g, slot }
    }

    pub fn mode(self, value: ArcMode) -> Self {
        self.g.amend(self.slot, |cmd| {
            let DrawCommand::Arc { mode, .. } = cmd else {
                unreachable!()
            };
            *mode = value;
        });
        self
    }
}

pub struct LightDraw<'g> {
    light: Entity,
    _g: PhantomData<&'g Graphics>,
}

impl<'g> LightDraw<'g> {
    pub(crate) fn new(g: &'g Graphics, kind: LightKind) -> Self {
        let light = expect(graphics_immediate_light(
            g.entity,
            kind,
            Transform::default(),
        ));
        Self {
            light,
            _g: PhantomData,
        }
    }

    pub fn position(self, position: impl IntoPoint) -> Self {
        expect(transform_set_position(self.light, position.into_point()));
        self
    }

    pub fn look_at(self, target: impl IntoPoint) -> Self {
        expect(transform_look_at(self.light, target.into_point()));
        self
    }
}

pub struct ParticlesDraw<'g> {
    g: &'g Graphics,
    slot: CommandSlot,
}

impl<'g> ParticlesDraw<'g> {
    pub(crate) fn new(g: &'g Graphics, particles: Entity) -> Self {
        let slot = g.record_slot(DrawCommand::Particles {
            particles,
            geometry: None,
            topology: Topology::PointList,
        });
        Self { g, slot }
    }

    pub fn geometry(self, value: &Geometry) -> Self {
        self.g.amend(self.slot, |cmd| {
            let DrawCommand::Particles { geometry, .. } = cmd else {
                unreachable!()
            };
            *geometry = Some(value.0);
        });
        self
    }

    pub fn topology(self, value: Topology) -> Self {
        self.g.amend(self.slot, |cmd| {
            let DrawCommand::Particles { topology, .. } = cmd else {
                unreachable!()
            };
            *topology = value;
        });
        self
    }
}

pub struct ImageDraw<'g> {
    g: &'g Graphics,
    slot: CommandSlot,
}

impl<'g> ImageDraw<'g> {
    pub(crate) fn new(g: &'g Graphics, entity: Entity, x: f32, y: f32) -> Self {
        let slot = g.record_slot(DrawCommand::Image {
            entity,
            dx: x,
            dy: y,
            d_width: None,
            d_height: None,
            sx: None,
            sy: None,
            s_width: None,
            s_height: None,
        });
        Self { g, slot }
    }

    pub fn size(self, w: impl Scalar, h: impl Scalar) -> Self {
        let (w, h) = (w.to_f32(), h.to_f32());
        self.g.amend(self.slot, |cmd| {
            let DrawCommand::Image {
                d_width, d_height, ..
            } = cmd
            else {
                unreachable!()
            };
            *d_width = Some(w);
            *d_height = Some(h);
        });
        self
    }

    /// Pixel region of the image.
    pub fn source(self, x: impl Scalar, y: impl Scalar, w: impl Scalar, h: impl Scalar) -> Self {
        let (x, y, w, h) = (x.to_f32(), y.to_f32(), w.to_f32(), h.to_f32());
        self.g.amend(self.slot, |cmd| {
            let DrawCommand::Image {
                sx,
                sy,
                s_width,
                s_height,
                ..
            } = cmd
            else {
                unreachable!()
            };
            *sx = Some(x);
            *sy = Some(y);
            *s_width = Some(w);
            *s_height = Some(h);
        });
        self
    }
}

pub struct Text<'g> {
    g: &'g Graphics,
    slot: CommandSlot,
}

impl<'g> Text<'g> {
    pub(crate) fn new(g: &'g Graphics, content: String, x: f32, y: f32) -> Self {
        let slot = g.record_slot(DrawCommand::Text {
            content,
            x,
            y,
            z: 0.0,
            max_w: None,
            max_h: None,
        });
        Self { g, slot }
    }

    pub fn z(self, value: impl Scalar) -> Self {
        let value = value.to_f32();
        self.g.amend(self.slot, |cmd| {
            let DrawCommand::Text { z, .. } = cmd else {
                unreachable!()
            };
            *z = value;
        });
        self
    }

    pub fn max_width(self, w: impl Scalar) -> Self {
        let w = w.to_f32();
        self.g.amend(self.slot, |cmd| {
            let DrawCommand::Text { max_w, .. } = cmd else {
                unreachable!()
            };
            *max_w = Some(w);
        });
        self
    }

    pub fn bounds(self, w: impl Scalar, h: impl Scalar) -> Self {
        let (w, h) = (w.to_f32(), h.to_f32());
        self.g.amend(self.slot, |cmd| {
            let DrawCommand::Text { max_w, max_h, .. } = cmd else {
                unreachable!()
            };
            *max_w = Some(w);
            *max_h = Some(h);
        });
        self
    }
}

pub struct Detail<'g, const N: usize> {
    g: &'g Graphics,
    slot: CommandSlot,
    size: [f32; 3],
    build: fn([f32; 3], [u32; N]) -> DrawCommand,
}

impl<'g, const N: usize> Detail<'g, N> {
    pub(crate) fn new(
        g: &'g Graphics,
        size: [f32; 3],
        detail: [u32; N],
        build: fn([f32; 3], [u32; N]) -> DrawCommand,
    ) -> Self {
        let slot = g.record_slot(build(size, detail));
        Self {
            g,
            slot,
            size,
            build,
        }
    }

    fn set_detail(self, detail: [u32; N]) -> Self {
        let rebuilt = (self.build)(self.size, detail);
        self.g.amend(self.slot, |cmd| *cmd = rebuilt);
        self
    }
}

impl Detail<'_, 1> {
    pub fn detail(self, segments: u32) -> Self {
        self.set_detail([segments])
    }
}

impl Detail<'_, 2> {
    pub fn detail(self, a: u32, b: u32) -> Self {
        self.set_detail([a, b])
    }
}

/// Pops when dropped.
#[must_use]
pub struct Push<'g> {
    g: &'g mut Graphics,
}

impl<'g> Push<'g> {
    pub(crate) fn new(g: &'g mut Graphics) -> Self {
        Self { g }
    }

    pub fn pop(self) {}
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
#[must_use]
pub struct BeginDraw<'g> {
    g: &'g mut Graphics,
}

impl<'g> BeginDraw<'g> {
    pub(crate) fn new(g: &'g mut Graphics) -> Self {
        Self { g }
    }

    pub fn end_draw(self) {}
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
