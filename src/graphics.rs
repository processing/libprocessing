use std::fmt::Display;

use bevy::math::{Affine3A, Vec3, Vec4};
use bevy::prelude::Entity;
use bevy::render::render_resource::TextureFormat;
use processing_core::error::Result;
use processing_input::{
    input_key, input_key_is_down, input_key_is_pressed, input_key_just_pressed, input_mouse_button,
    input_mouse_is_pressed, input_mouse_wheel, input_mouse_x, input_mouse_y, input_moved_x,
    input_moved_y, input_pmouse_x, input_pmouse_y,
};
use processing_render::render::command::{
    BlendMode, DrawCommand, LightKind, ShapeKind, ShapeMode, StrokeCapMode, StrokeJoinMode,
    TextAlignH, TextAlignV, TextStyle, TextWrapMode,
};
use processing_render::{
    camera_reset, camera_set_center, camera_set_distance, delta_time, elapsed_time, frame_count,
    geometry_create_from_mesh, graphics_apply_filter, graphics_begin_draw, graphics_create,
    graphics_destroy, graphics_disable_camera_controller, graphics_end_draw, graphics_flush,
    graphics_free_camera, graphics_get_matrix, graphics_light, graphics_mode_2d, graphics_mode_3d,
    graphics_orbit_camera, graphics_ortho, graphics_pan_camera, graphics_perspective,
    graphics_readback, graphics_record_command, graphics_screen_point, graphics_set_color_mode,
    graphics_text_ascent, graphics_text_descent, graphics_text_to_model, graphics_text_width,
    graphics_update, graphics_update_region, light_create_directional, light_create_point,
    light_create_spot, light_destroy, surface_create_offscreen, surface_destroy,
    surface_physical_height, surface_physical_width, transform_look_at, transform_set_position,
};

use crate::color::{ColorArg, ColorSpace, ColorState, IntoColorMax};
use crate::draw::{Arc, BeginDraw, Detail, ImageDraw, LightDraw, ParticlesDraw, Push, Rect, Text};
use crate::input::{Key, MouseButton};
use crate::shape::Shape;
use crate::{AsImage, Color, Font, IntoExtent, IntoPoint, Scalar};
use crate::{Buffer, Filter, Geometry, Material, Particles};

pub(crate) fn expect<T>(r: Result<T>) -> T {
    r.unwrap_or_else(|e| panic!("processing: {e}"))
}

/// The sketch window, or an offscreen surface.
pub struct Graphics {
    pub(crate) entity: Entity,
    pub(crate) surface: Entity,
    width: u32,
    height: u32,
    color: ColorState,
    style_stack: Vec<ColorState>,
    offscreen: bool,
    exit_requested: bool,
}

impl Graphics {
    pub(crate) fn new(entity: Entity, surface: Entity, width: u32, height: u32) -> Self {
        Self {
            entity,
            surface,
            width,
            height,
            color: ColorState::default(),
            style_stack: Vec::new(),
            offscreen: false,
            exit_requested: false,
        }
    }

    /// Processing's `createGraphics`.
    pub fn create(width: u32, height: u32) -> Result<Self> {
        let format = TextureFormat::Rgba8UnormSrgb;
        let surface = surface_create_offscreen(width, height, 1.0, format)?;
        let entity = graphics_create(surface, width, height, format)?;
        let mut g = Self::new(entity, surface, width, height);
        g.offscreen = true;
        Ok(g)
    }

    pub fn begin_draw(&mut self) -> BeginDraw<'_> {
        assert!(
            self.offscreen,
            "processing: begin_draw is for Graphics::create surfaces; the sketch window is drawn every frame"
        );
        self.begin_frame();
        BeginDraw::new(self)
    }

    pub(crate) fn record(&mut self, cmd: DrawCommand) {
        expect(graphics_record_command(self.entity, cmd));
    }

    pub(crate) fn begin_frame(&mut self) {
        self.style_stack.clear();
        expect(graphics_begin_draw(self.entity));
    }

    pub(crate) fn end_frame(&mut self) {
        expect(graphics_end_draw(self.entity));
    }

    pub(crate) fn resolve(&self, color: impl Into<ColorArg>) -> bevy::color::Color {
        self.color.resolve(color.into()).0
    }

    pub fn width(&self) -> f32 {
        self.width as f32
    }

    pub fn height(&self) -> f32 {
        self.height as f32
    }

    /// In physical pixels.
    pub fn pixel_width(&self) -> u32 {
        if self.offscreen {
            return self.width;
        }
        expect(surface_physical_width(self.surface))
    }

    /// In physical pixels.
    pub fn pixel_height(&self) -> u32 {
        if self.offscreen {
            return self.height;
        }
        expect(surface_physical_height(self.surface))
    }

    /// Starts at 1.
    pub fn frame_count(&self) -> u32 {
        expect(frame_count())
    }

    /// In seconds.
    pub fn delta_time(&self) -> f32 {
        expect(delta_time())
    }

    /// In seconds.
    pub fn elapsed_time(&self) -> f32 {
        expect(elapsed_time())
    }

    pub fn millis(&self) -> u32 {
        (expect(elapsed_time()) * 1000.0) as u32
    }

    pub fn exit(&mut self) {
        self.exit_requested = true;
    }

    pub(crate) fn exit_requested(&self) -> bool {
        self.exit_requested
    }

    pub fn color_mode(&mut self, space: ColorSpace) -> ColorModeBuilder<'_> {
        ColorModeBuilder {
            g: self,
            space,
            max: None,
        }
    }

    pub fn color(&self, color: impl Into<ColorArg>) -> Color {
        self.color.resolve(color.into())
    }

    pub fn background(&mut self, color: impl Into<ColorArg>) {
        let c = self.resolve(color);
        self.record(DrawCommand::BackgroundColor(c));
    }

    pub fn background_image(&mut self, image: &impl AsImage) {
        self.record(DrawCommand::BackgroundImage(image.image_entity()));
    }

    pub fn fill(&mut self, color: impl Into<ColorArg>) {
        let c = self.resolve(color);
        self.record(DrawCommand::Fill(c));
    }

    pub fn no_fill(&mut self) {
        self.record(DrawCommand::NoFill);
    }

    pub fn stroke(&mut self, color: impl Into<ColorArg>) {
        let c = self.resolve(color);
        self.record(DrawCommand::StrokeColor(c));
    }

    pub fn no_stroke(&mut self) {
        self.record(DrawCommand::NoStroke);
    }

    pub fn stroke_weight(&mut self, weight: impl Scalar) {
        self.record(DrawCommand::StrokeWeight(weight.to_f32()));
    }

    pub fn stroke_cap(&mut self, cap: StrokeCapMode) {
        self.record(DrawCommand::StrokeCap(cap));
    }

    pub fn stroke_join(&mut self, join: StrokeJoinMode) {
        self.record(DrawCommand::StrokeJoin(join));
    }

    pub fn tint(&mut self, color: impl Into<ColorArg>) {
        let c = self.resolve(color);
        self.record(DrawCommand::Tint(c));
    }

    pub fn no_tint(&mut self) {
        self.record(DrawCommand::NoTint);
    }

    pub fn blend_mode(&mut self, mode: BlendMode) {
        self.record(DrawCommand::BlendMode(mode.to_blend_state()));
    }

    pub fn rect_mode(&mut self, mode: ShapeMode) {
        self.record(DrawCommand::RectMode(mode));
    }

    pub fn ellipse_mode(&mut self, mode: ShapeMode) {
        self.record(DrawCommand::EllipseMode(mode));
    }

    pub fn image_mode(&mut self, mode: ShapeMode) {
        self.record(DrawCommand::ImageMode(mode));
    }

    pub fn rect(
        &mut self,
        x: impl Scalar,
        y: impl Scalar,
        w: impl Scalar,
        h: impl Scalar,
    ) -> Rect<'_> {
        Rect::new(self, x.to_f32(), y.to_f32(), w.to_f32(), h.to_f32())
    }

    pub fn square(&mut self, x: impl Scalar, y: impl Scalar, size: impl Scalar) -> Rect<'_> {
        let s = size.to_f32();
        Rect::new(self, x.to_f32(), y.to_f32(), s, s)
    }

    pub fn ellipse(&mut self, x: impl Scalar, y: impl Scalar, w: impl Scalar, h: impl Scalar) {
        self.record(DrawCommand::Ellipse {
            cx: x.to_f32(),
            cy: y.to_f32(),
            w: w.to_f32(),
            h: h.to_f32(),
        });
    }

    pub fn circle(&mut self, x: impl Scalar, y: impl Scalar, diameter: impl Scalar) {
        let d = diameter.to_f32();
        self.ellipse(x, y, d, d);
    }

    pub fn arc(
        &mut self,
        x: impl Scalar,
        y: impl Scalar,
        w: impl Scalar,
        h: impl Scalar,
        start: impl Scalar,
        stop: impl Scalar,
    ) -> Arc<'_> {
        Arc::new(
            self,
            [x.to_f32(), y.to_f32(), w.to_f32(), h.to_f32()],
            start.to_f32(),
            stop.to_f32(),
        )
    }

    pub fn line(&mut self, x1: impl Scalar, y1: impl Scalar, x2: impl Scalar, y2: impl Scalar) {
        self.record(DrawCommand::Line {
            x1: x1.to_f32(),
            y1: y1.to_f32(),
            x2: x2.to_f32(),
            y2: y2.to_f32(),
        });
    }

    pub fn point(&mut self, x: impl Scalar, y: impl Scalar) {
        self.record(DrawCommand::Point {
            x: x.to_f32(),
            y: y.to_f32(),
        });
    }

    pub fn triangle(
        &mut self,
        x1: impl Scalar,
        y1: impl Scalar,
        x2: impl Scalar,
        y2: impl Scalar,
        x3: impl Scalar,
        y3: impl Scalar,
    ) {
        self.record(DrawCommand::Triangle {
            x1: x1.to_f32(),
            y1: y1.to_f32(),
            x2: x2.to_f32(),
            y2: y2.to_f32(),
            x3: x3.to_f32(),
            y3: y3.to_f32(),
        });
    }

    pub fn quad(
        &mut self,
        x1: impl Scalar,
        y1: impl Scalar,
        x2: impl Scalar,
        y2: impl Scalar,
        x3: impl Scalar,
        y3: impl Scalar,
        x4: impl Scalar,
        y4: impl Scalar,
    ) {
        self.record(DrawCommand::Quad {
            x1: x1.to_f32(),
            y1: y1.to_f32(),
            x2: x2.to_f32(),
            y2: y2.to_f32(),
            x3: x3.to_f32(),
            y3: y3.to_f32(),
            x4: x4.to_f32(),
            y4: y4.to_f32(),
        });
    }

    pub fn bezier(
        &mut self,
        x1: impl Scalar,
        y1: impl Scalar,
        x2: impl Scalar,
        y2: impl Scalar,
        x3: impl Scalar,
        y3: impl Scalar,
        x4: impl Scalar,
        y4: impl Scalar,
    ) {
        self.record(DrawCommand::Bezier {
            x1: x1.to_f32(),
            y1: y1.to_f32(),
            x2: x2.to_f32(),
            y2: y2.to_f32(),
            x3: x3.to_f32(),
            y3: y3.to_f32(),
            x4: x4.to_f32(),
            y4: y4.to_f32(),
        });
    }

    pub fn curve(
        &mut self,
        x1: impl Scalar,
        y1: impl Scalar,
        x2: impl Scalar,
        y2: impl Scalar,
        x3: impl Scalar,
        y3: impl Scalar,
        x4: impl Scalar,
        y4: impl Scalar,
    ) {
        self.record(DrawCommand::Curve {
            x1: x1.to_f32(),
            y1: y1.to_f32(),
            x2: x2.to_f32(),
            y2: y2.to_f32(),
            x3: x3.to_f32(),
            y3: y3.to_f32(),
            x4: x4.to_f32(),
            y4: y4.to_f32(),
        });
    }

    pub fn begin_shape(&mut self, kind: ShapeKind) -> Shape<'_> {
        Shape::new(self, kind)
    }

    pub fn image(&mut self, image: &impl AsImage, x: impl Scalar, y: impl Scalar) -> ImageDraw<'_> {
        ImageDraw::new(self, image.image_entity(), x.to_f32(), y.to_f32())
    }

    pub fn text(&mut self, content: impl Display, x: impl Scalar, y: impl Scalar) -> Text<'_> {
        Text::new(self, content.to_string(), x.to_f32(), y.to_f32())
    }

    pub fn text_font(&mut self, font: Font) {
        self.record(DrawCommand::TextFont(Some(font.0)));
    }

    pub fn text_size(&mut self, size: impl Scalar) {
        self.record(DrawCommand::TextSize(size.to_f32()));
    }

    pub fn text_leading(&mut self, leading: impl Scalar) {
        self.record(DrawCommand::TextLeading(leading.to_f32()));
    }

    pub fn text_align(&mut self, align: impl Into<TextAlign>) {
        let TextAlign(h, v) = align.into();
        self.record(DrawCommand::TextAlign { h, v });
    }

    pub fn text_style(&mut self, style: TextStyle) {
        self.record(DrawCommand::TextStyle(style));
    }

    pub fn text_wrap(&mut self, mode: TextWrapMode) {
        self.record(DrawCommand::TextWrap(mode));
    }

    pub fn text_width(&self, content: &str) -> f32 {
        expect(graphics_text_width(self.entity, content))
    }

    pub fn text_ascent(&self) -> f32 {
        expect(graphics_text_ascent(self.entity))
    }

    pub fn text_descent(&self) -> f32 {
        expect(graphics_text_descent(self.entity))
    }

    pub fn push(&mut self) -> Push<'_> {
        self.push_style();
        self.push_matrix();
        Push::new(self)
    }

    pub fn push_matrix(&mut self) {
        self.record(DrawCommand::PushMatrix);
    }

    pub fn pop_matrix(&mut self) {
        self.record(DrawCommand::PopMatrix);
    }

    pub fn push_style(&mut self) {
        self.style_stack.push(self.color);
        self.record(DrawCommand::PushStyle);
    }

    pub fn pop_style(&mut self) {
        if let Some(color) = self.style_stack.pop() {
            self.set_color_state(color);
        }
        self.record(DrawCommand::PopStyle);
    }

    pub fn translate(&mut self, offset: impl IntoPoint) {
        self.record(DrawCommand::Translate(offset.into_point()));
    }

    pub fn rotate(&mut self, angle: impl Scalar) {
        self.rotate_axis(angle, Vec3::Z);
    }

    pub fn rotate_x(&mut self, angle: impl Scalar) {
        self.rotate_axis(angle, Vec3::X);
    }

    pub fn rotate_y(&mut self, angle: impl Scalar) {
        self.rotate_axis(angle, Vec3::Y);
    }

    pub fn rotate_z(&mut self, angle: impl Scalar) {
        self.rotate_axis(angle, Vec3::Z);
    }

    pub fn rotate_axis(&mut self, angle: impl Scalar, axis: impl IntoPoint) {
        self.record(DrawCommand::Rotate {
            angle: angle.to_f32(),
            axis: axis.into_point(),
        });
    }

    pub fn scale(&mut self, factor: impl IntoExtent) {
        self.record(DrawCommand::Scale(factor.into_extent()));
    }

    pub fn shear_x(&mut self, angle: impl Scalar) {
        self.record(DrawCommand::ShearX {
            angle: angle.to_f32(),
        });
    }

    pub fn shear_y(&mut self, angle: impl Scalar) {
        self.record(DrawCommand::ShearY {
            angle: angle.to_f32(),
        });
    }

    pub fn reset_matrix(&mut self) {
        self.record(DrawCommand::ResetMatrix);
    }

    pub fn apply_matrix(&mut self, matrix: Affine3A) {
        self.record(DrawCommand::ApplyMatrix(matrix));
    }

    pub fn matrix(&self) -> bevy::math::Mat4 {
        expect(graphics_get_matrix(self.entity))
    }

    pub fn screen_point(&self, point: impl IntoPoint) -> Vec3 {
        expect(graphics_screen_point(self.entity, point.into_point()))
    }

    pub fn mode_3d(&mut self) {
        expect(graphics_mode_3d(self.entity));
    }

    pub fn mode_2d(&mut self) {
        expect(graphics_mode_2d(self.entity));
    }

    pub fn cube(&mut self, size: impl IntoExtent) {
        let s = size.into_extent();
        self.record(DrawCommand::Box {
            width: s.x,
            height: s.y,
            depth: s.z,
        });
    }

    pub fn sphere(&mut self, radius: impl Scalar) -> Detail<'_, 2> {
        Detail::new(
            self,
            [radius.to_f32(), 0.0, 0.0],
            [32, 16],
            |[radius, _, _], [sectors, stacks]| DrawCommand::Sphere {
                radius,
                sectors,
                stacks,
            },
        )
    }

    pub fn cylinder(&mut self, radius: impl Scalar, height: impl Scalar) -> Detail<'_, 1> {
        let size = [radius.to_f32(), height.to_f32(), 0.0];
        Detail::new(self, size, [24], |[radius, height, _], [detail]| {
            DrawCommand::Cylinder {
                radius,
                height,
                detail,
            }
        })
    }

    pub fn cone(&mut self, radius: impl Scalar, height: impl Scalar) -> Detail<'_, 1> {
        let size = [radius.to_f32(), height.to_f32(), 0.0];
        Detail::new(self, size, [24], |[radius, height, _], [detail]| {
            DrawCommand::Cone {
                radius,
                height,
                detail,
            }
        })
    }

    pub fn torus(&mut self, radius: impl Scalar, tube_radius: impl Scalar) -> Detail<'_, 2> {
        let size = [radius.to_f32(), tube_radius.to_f32(), 0.0];
        Detail::new(
            self,
            size,
            [32, 16],
            |[radius, tube_radius, _], [major, minor]| DrawCommand::Torus {
                radius,
                tube_radius,
                major_segments: major,
                minor_segments: minor,
            },
        )
    }

    pub fn capsule(&mut self, radius: impl Scalar, length: impl Scalar) -> Detail<'_, 1> {
        let size = [radius.to_f32(), length.to_f32(), 0.0];
        Detail::new(self, size, [16], |[radius, length, _], [detail]| {
            DrawCommand::Capsule {
                radius,
                length,
                detail,
            }
        })
    }

    pub fn conical_frustum(
        &mut self,
        radius_top: impl Scalar,
        radius_bottom: impl Scalar,
        height: impl Scalar,
    ) -> Detail<'_, 1> {
        let size = [radius_top.to_f32(), radius_bottom.to_f32(), height.to_f32()];
        Detail::new(
            self,
            size,
            [24],
            |[radius_top, radius_bottom, height], [detail]| DrawCommand::ConicalFrustum {
                radius_top,
                radius_bottom,
                height,
                detail,
            },
        )
    }

    pub fn plane(&mut self, width: impl Scalar, height: impl Scalar) {
        self.record(DrawCommand::Plane {
            width: width.to_f32(),
            height: height.to_f32(),
        });
    }

    pub fn tetrahedron(&mut self, radius: impl Scalar) {
        self.record(DrawCommand::Tetrahedron {
            radius: radius.to_f32(),
        });
    }

    pub fn roughness(&mut self, value: impl Scalar) {
        self.record(DrawCommand::Roughness(value.to_f32()));
    }

    pub fn metallic(&mut self, value: impl Scalar) {
        self.record(DrawCommand::Metallic(value.to_f32()));
    }

    pub fn emissive(&mut self, color: impl Into<ColorArg>) {
        let c = self.resolve(color);
        self.record(DrawCommand::Emissive(c));
    }

    pub fn unlit(&mut self) {
        self.record(DrawCommand::Unlit);
    }

    pub fn texture(&mut self, image: &impl AsImage) {
        self.record(DrawCommand::Texture(image.image_entity()));
    }

    pub fn no_texture(&mut self) {
        self.record(DrawCommand::NoTexture);
    }

    pub fn camera_position(&mut self, eye: impl IntoPoint) {
        expect(transform_set_position(self.entity, eye.into_point()));
    }

    pub fn camera_look_at(&mut self, target: impl IntoPoint) {
        expect(transform_look_at(self.entity, target.into_point()));
    }

    pub fn perspective(
        &mut self,
        fov: impl Scalar,
        aspect: impl Scalar,
        near: impl Scalar,
        far: impl Scalar,
    ) {
        let near = near.to_f32();
        expect(graphics_perspective(
            self.entity,
            fov.to_f32(),
            aspect.to_f32(),
            near,
            far.to_f32(),
            Vec4::new(0.0, 0.0, -1.0, -near),
        ));
    }

    pub fn ortho(
        &mut self,
        left: impl Scalar,
        right: impl Scalar,
        bottom: impl Scalar,
        top: impl Scalar,
        near: impl Scalar,
        far: impl Scalar,
    ) {
        expect(graphics_ortho(
            self.entity,
            left.to_f32(),
            right.to_f32(),
            bottom.to_f32(),
            top.to_f32(),
            near.to_f32(),
            far.to_f32(),
        ));
    }

    pub fn orbit_camera(&mut self) {
        expect(graphics_orbit_camera(self.entity));
    }

    pub fn free_camera(&mut self) {
        expect(graphics_free_camera(self.entity));
    }

    pub fn pan_camera(&mut self) {
        expect(graphics_pan_camera(self.entity));
    }

    pub fn disable_camera(&mut self) {
        expect(graphics_disable_camera_controller(self.entity));
    }

    pub fn camera_distance(&mut self, distance: impl Scalar) {
        expect(camera_set_distance(self.entity, distance.to_f32()));
    }

    pub fn camera_center(&mut self, center: impl IntoPoint) {
        expect(camera_set_center(self.entity, center.into_point()));
    }

    pub fn camera_reset(&mut self) {
        expect(camera_reset(self.entity));
    }

    pub fn light(&mut self, light: &Light) {
        expect(graphics_light(self.entity, light.0));
    }

    pub fn directional_light(
        &mut self,
        color: impl Into<ColorArg>,
        illuminance: impl Scalar,
    ) -> LightDraw<'_> {
        let color = self.resolve(color);
        LightDraw::new(
            self,
            LightKind::Directional {
                color,
                illuminance: illuminance.to_f32(),
            },
        )
    }

    pub fn point_light(
        &mut self,
        color: impl Into<ColorArg>,
        intensity: impl Scalar,
        range: impl Scalar,
        radius: impl Scalar,
    ) -> LightDraw<'_> {
        let color = self.resolve(color);
        LightDraw::new(
            self,
            LightKind::Point {
                color,
                intensity: intensity.to_f32(),
                range: range.to_f32(),
                radius: radius.to_f32(),
            },
        )
    }

    pub fn spot_light(
        &mut self,
        color: impl Into<ColorArg>,
        intensity: impl Scalar,
        range: impl Scalar,
        radius: impl Scalar,
        inner_angle: impl Scalar,
        outer_angle: impl Scalar,
    ) -> LightDraw<'_> {
        let color = self.resolve(color);
        LightDraw::new(
            self,
            LightKind::Spot {
                color,
                intensity: intensity.to_f32(),
                range: range.to_f32(),
                radius: radius.to_f32(),
                inner_angle: inner_angle.to_f32(),
                outer_angle: outer_angle.to_f32(),
            },
        )
    }

    pub fn create_directional_light(
        &mut self,
        color: impl Into<ColorArg>,
        illuminance: impl Scalar,
    ) -> Light {
        let c = self.resolve(color);
        Light(expect(light_create_directional(
            self.entity,
            c,
            illuminance.to_f32(),
        )))
    }

    pub fn create_point_light(
        &mut self,
        color: impl Into<ColorArg>,
        intensity: impl Scalar,
        range: impl Scalar,
        radius: impl Scalar,
    ) -> Light {
        let c = self.resolve(color);
        Light(expect(light_create_point(
            self.entity,
            c,
            intensity.to_f32(),
            range.to_f32(),
            radius.to_f32(),
        )))
    }

    pub fn create_spot_light(
        &mut self,
        color: impl Into<ColorArg>,
        intensity: impl Scalar,
        range: impl Scalar,
        radius: impl Scalar,
        inner_angle: impl Scalar,
        outer_angle: impl Scalar,
    ) -> Light {
        let c = self.resolve(color);
        Light(expect(light_create_spot(
            self.entity,
            c,
            intensity.to_f32(),
            range.to_f32(),
            radius.to_f32(),
            inner_angle.to_f32(),
            outer_angle.to_f32(),
        )))
    }

    pub fn model(&mut self, geometry: &Geometry) {
        self.record(DrawCommand::Geometry(geometry.0));
    }

    pub fn material(&mut self, material: &Material) {
        self.record(DrawCommand::Material(material.0));
    }

    /// One color per particle.
    pub fn fill_buffer(&mut self, buffer: &Buffer) {
        self.record(DrawCommand::FillBuffer(buffer.entity));
    }

    /// Points unless given a geometry.
    pub fn particles(&mut self, particles: &Particles) -> ParticlesDraw<'_> {
        ParticlesDraw::new(self, particles.0)
    }

    pub fn filter(&mut self, filter: Filter) {
        let entity = expect(filter.entity());
        expect(graphics_apply_filter(self.entity, entity));
    }

    pub fn text_to_model(
        &self,
        content: &str,
        x: impl Scalar,
        y: impl Scalar,
        depth: impl Scalar,
    ) -> Result<Geometry> {
        let mesh =
            graphics_text_to_model(self.entity, content, x.to_f32(), y.to_f32(), depth.to_f32())?;
        geometry_create_from_mesh(mesh).map(Geometry)
    }

    /// Row-major.
    pub fn pixels(&self) -> Result<Vec<Color>> {
        Ok(graphics_readback(self.entity)?
            .into_iter()
            .map(Color::from_linear)
            .collect())
    }

    /// `pixels.len()` must be `pixel_width() * pixel_height()`.
    pub fn set_pixels(&mut self, pixels: &[Color]) -> Result<()> {
        let linear: Vec<_> = pixels.iter().map(|c| c.to_linear()).collect();
        graphics_flush(self.entity)?;
        graphics_update(self.entity, &linear)
    }

    pub fn set_pixels_region(
        &mut self,
        x: u32,
        y: u32,
        w: u32,
        h: u32,
        pixels: &[Color],
    ) -> Result<()> {
        let linear: Vec<_> = pixels.iter().map(|c| c.to_linear()).collect();
        graphics_flush(self.entity)?;
        graphics_update_region(self.entity, x, y, w, h, &linear)
    }

    pub fn mouse_x(&self) -> f32 {
        expect(input_mouse_x(self.surface)).clamp(0.0, self.width())
    }

    pub fn mouse_y(&self) -> f32 {
        expect(input_mouse_y(self.surface)).clamp(0.0, self.height())
    }

    pub fn pmouse_x(&self) -> f32 {
        expect(input_pmouse_x(self.surface)).clamp(0.0, self.width())
    }

    pub fn pmouse_y(&self) -> f32 {
        expect(input_pmouse_y(self.surface)).clamp(0.0, self.height())
    }

    pub fn moved_x(&self) -> f32 {
        expect(input_moved_x())
    }

    pub fn moved_y(&self) -> f32 {
        expect(input_moved_y())
    }

    pub fn mouse_wheel(&self) -> f32 {
        expect(input_mouse_wheel())
    }

    pub fn mouse_pressed(&self) -> bool {
        expect(input_mouse_is_pressed())
    }

    pub fn mouse_button(&self) -> Option<MouseButton> {
        expect(input_mouse_button()).map(MouseButton::from)
    }

    pub fn key_pressed(&self) -> bool {
        expect(input_key_is_pressed())
    }

    pub fn key(&self) -> Option<char> {
        expect(input_key())
    }

    pub fn key_code(&self) -> Option<Key> {
        expect(processing_input::input_key_code()).and_then(Key::from_code)
    }

    pub fn key_down(&self, key: Key) -> bool {
        expect(input_key_is_down(key.code()))
    }

    pub fn key_just_pressed(&self, key: Key) -> bool {
        expect(input_key_just_pressed(key.code()))
    }

    fn set_color_state(&mut self, color: ColorState) {
        self.color = color;
        expect(graphics_set_color_mode(self.entity, color.mode));
    }
}

impl AsImage for Graphics {
    fn image_entity(&self) -> Entity {
        assert!(
            self.offscreen,
            "processing: only Graphics::create surfaces can be drawn as images"
        );
        self.surface
    }
}

impl Drop for Graphics {
    fn drop(&mut self) {
        let _ = graphics_destroy(self.entity);
        if self.offscreen {
            let _ = surface_destroy(self.surface);
        }
    }
}

pub struct ColorModeBuilder<'g> {
    g: &'g mut Graphics,
    space: ColorSpace,
    max: Option<[f32; 4]>,
}

impl ColorModeBuilder<'_> {
    /// `max(255)`, `max((360, 100, 100))` or `max((360, 100, 100, 1))`.
    pub fn max(mut self, max: impl IntoColorMax) -> Self {
        self.max = Some(max.into_color_max(self.space));
        self
    }
}

impl Drop for ColorModeBuilder<'_> {
    fn drop(&mut self) {
        use processing_render::color::ColorMode;
        let state = match self.max {
            Some([m1, m2, m3, ma]) => ColorState {
                mode: ColorMode::new(self.space, m1, m2, m3, ma),
                explicit_max: true,
            },
            None => ColorState {
                mode: ColorMode::with_defaults(self.space),
                explicit_max: false,
            },
        };
        self.g.set_color_state(state);
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TextAlign(TextAlignH, TextAlignV);

impl From<TextAlignH> for TextAlign {
    fn from(h: TextAlignH) -> Self {
        Self(h, TextAlignV::default())
    }
}

impl From<(TextAlignH, TextAlignV)> for TextAlign {
    fn from((h, v): (TextAlignH, TextAlignV)) -> Self {
        Self(h, v)
    }
}

#[derive(Debug)]
pub struct Light(pub(crate) Entity);

impl Light {
    pub fn position(&self, position: impl IntoPoint) -> &Self {
        expect(transform_set_position(self.0, position.into_point()));
        self
    }

    pub fn look_at(&self, target: impl IntoPoint) -> &Self {
        expect(transform_look_at(self.0, target.into_point()));
        self
    }
}

impl Drop for Light {
    fn drop(&mut self) {
        let _ = light_destroy(self.0);
    }
}
