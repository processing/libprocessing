//! `begin_shape` / `end_shape` as a value: vertices only exist inside a shape.

use std::ops::{Deref, DerefMut};

use processing_render::render::command::{DrawCommand, ShapeKind};

use crate::{Graphics, Scalar};

/// Returned by [`Graphics::begin_shape`]. Ends with [`Shape::end`], [`Shape::close`], or when dropped.
pub struct Shape<'g> {
    g: &'g mut Graphics,
    close: bool,
}

impl<'g> Shape<'g> {
    pub(crate) fn new(g: &'g mut Graphics, kind: ShapeKind) -> Self {
        g.record(DrawCommand::BeginShape { kind });
        Self { g, close: false }
    }

    pub fn vertex(&mut self, x: impl Scalar, y: impl Scalar) {
        self.g.record(DrawCommand::ShapeVertex {
            x: x.to_f32(),
            y: y.to_f32(),
        });
    }

    pub fn bezier_vertex(
        &mut self,
        cx1: impl Scalar,
        cy1: impl Scalar,
        cx2: impl Scalar,
        cy2: impl Scalar,
        x: impl Scalar,
        y: impl Scalar,
    ) {
        self.g.record(DrawCommand::ShapeBezierVertex {
            cx1: cx1.to_f32(),
            cy1: cy1.to_f32(),
            cx2: cx2.to_f32(),
            cy2: cy2.to_f32(),
            x: x.to_f32(),
            y: y.to_f32(),
        });
    }

    pub fn quadratic_vertex(
        &mut self,
        cx: impl Scalar,
        cy: impl Scalar,
        x: impl Scalar,
        y: impl Scalar,
    ) {
        self.g.record(DrawCommand::ShapeQuadraticVertex {
            cx: cx.to_f32(),
            cy: cy.to_f32(),
            x: x.to_f32(),
            y: y.to_f32(),
        });
    }

    pub fn curve_vertex(&mut self, x: impl Scalar, y: impl Scalar) {
        self.g.record(DrawCommand::ShapeCurveVertex {
            x: x.to_f32(),
            y: y.to_f32(),
        });
    }

    /// Starts a hole; it ends when the returned guard is dropped.
    pub fn contour(&mut self) -> Contour<'_, 'g> {
        self.g.record(DrawCommand::BeginContour);
        Contour { shape: self }
    }

    /// Ends the shape open; same as dropping it.
    pub fn end(self) {}

    /// Ends the shape, joining the last vertex to the first.
    pub fn close(mut self) {
        self.close = true;
    }
}

impl Drop for Shape<'_> {
    fn drop(&mut self) {
        self.g.record(DrawCommand::EndShape { close: self.close });
    }
}

/// Returned by [`Shape::contour`].
pub struct Contour<'s, 'g> {
    shape: &'s mut Shape<'g>,
}

impl<'g> Deref for Contour<'_, 'g> {
    type Target = Shape<'g>;

    fn deref(&self) -> &Shape<'g> {
        self.shape
    }
}

impl<'g> DerefMut for Contour<'_, 'g> {
    fn deref_mut(&mut self) -> &mut Shape<'g> {
        self.shape
    }
}

impl Drop for Contour<'_, '_> {
    fn drop(&mut self) {
        self.shape.g.record(DrawCommand::EndContour);
    }
}
