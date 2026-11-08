use bevy::math::{Vec2, Vec3, Vec4};
use bevy::prelude::Entity;
use processing_core::error::Result;
use processing_render::geometry::{AttributeFormat, Topology};
use processing_render::shader_value::ShaderValue;
use processing_render::{
    geometry_attribute_age, geometry_attribute_color, geometry_attribute_create,
    geometry_attribute_float, geometry_attribute_float2, geometry_attribute_float3,
    geometry_attribute_float4, geometry_attribute_life, geometry_attribute_normal,
    geometry_attribute_position, geometry_attribute_rotation, geometry_attribute_scale,
    geometry_attribute_uv, geometry_attribute_velocity, geometry_box, geometry_color,
    geometry_create, geometry_create_with_layout, geometry_destroy, geometry_get_indices,
    geometry_get_positions, geometry_grid, geometry_index, geometry_index_count,
    geometry_layout_add_attribute, geometry_layout_add_color, geometry_layout_add_normal,
    geometry_layout_add_position, geometry_layout_add_uv, geometry_layout_create,
    geometry_layout_destroy, geometry_normal, geometry_set_color, geometry_set_normal,
    geometry_set_uv, geometry_set_vertex, geometry_sphere, geometry_uv, geometry_vertex,
    geometry_vertex_count,
};

use crate::color::{ColorArg, ColorState};
use crate::graphics::expect;
use crate::{IntoExtent, IntoPoint, IntoShaderValue, Scalar};

/// A retained mesh, drawn with `g.model(..)`.
#[derive(Debug)]
pub struct Geometry(pub(crate) Entity);

impl Geometry {
    pub fn new(topology: Topology) -> Result<Self> {
        geometry_create(topology).map(Self)
    }

    pub fn with_layout(layout: &Layout, topology: Topology) -> Result<Self> {
        geometry_create_with_layout(layout.0, topology).map(Self)
    }

    pub fn cube(size: impl IntoExtent) -> Result<Self> {
        let s = size.into_extent();
        geometry_box(s.x, s.y, s.z).map(Self)
    }

    pub fn sphere(radius: impl Scalar, sectors: u32, stacks: u32) -> Result<Self> {
        geometry_sphere(radius.to_f32(), sectors, stacks).map(Self)
    }

    pub fn grid(nx: u32, ny: u32, nz: u32, spacing: impl Scalar) -> Result<Self> {
        geometry_grid(nx, ny, nz, spacing.to_f32()).map(Self)
    }

    pub fn vertex(&mut self, position: impl IntoPoint) {
        expect(geometry_vertex(self.0, position.into_point()));
    }

    pub fn normal(&mut self, normal: impl IntoPoint) {
        expect(geometry_normal(self.0, normal.into_point()));
    }

    pub fn color(&mut self, color: impl Into<ColorArg>) {
        expect(geometry_color(self.0, color_vec4(color)));
    }

    pub fn uv(&mut self, u: impl Scalar, v: impl Scalar) {
        expect(geometry_uv(self.0, u.to_f32(), v.to_f32()));
    }

    pub fn attribute(&mut self, attribute: Attribute, value: impl AttributeValue) {
        let entity = self.0;
        expect(match value.into_value() {
            Value::Float(v) => geometry_attribute_float(entity, attribute.0, v),
            Value::Float2(v) => geometry_attribute_float2(entity, attribute.0, v.x, v.y),
            Value::Float3(v) => geometry_attribute_float3(entity, attribute.0, v.x, v.y, v.z),
            Value::Float4(v) => geometry_attribute_float4(entity, attribute.0, v.x, v.y, v.z, v.w),
        });
    }

    pub fn index(&mut self, i: u32) {
        expect(geometry_index(self.0, i));
    }

    pub fn set_vertex(&mut self, i: u32, position: impl IntoPoint) {
        expect(geometry_set_vertex(self.0, i, position.into_point()));
    }

    pub fn set_normal(&mut self, i: u32, normal: impl IntoPoint) {
        expect(geometry_set_normal(self.0, i, normal.into_point()));
    }

    pub fn set_color(&mut self, i: u32, color: impl Into<ColorArg>) {
        expect(geometry_set_color(self.0, i, color_vec4(color)));
    }

    pub fn set_uv(&mut self, i: u32, u: impl Scalar, v: impl Scalar) {
        expect(geometry_set_uv(
            self.0,
            i,
            Vec2::new(u.to_f32(), v.to_f32()),
        ));
    }

    pub fn vertex_count(&self) -> u32 {
        expect(geometry_vertex_count(self.0))
    }

    pub fn index_count(&self) -> u32 {
        expect(geometry_index_count(self.0))
    }

    pub fn positions(&self) -> Vec<Vec3> {
        let count = self.vertex_count() as usize;
        let positions = expect(geometry_get_positions(self.0, 0, count));
        positions.into_iter().map(Vec3::from_array).collect()
    }

    /// Empty if the mesh has none.
    pub fn indices(&self) -> Vec<u32> {
        let count = self.index_count() as usize;
        expect(geometry_get_indices(self.0, 0, count))
    }

    /// For `compute.set(..)`.
    pub fn attribute_data(&self, attribute: Attribute) -> GeometryData {
        GeometryData(ShaderValue::MeshAttribute(self.0, attribute.0))
    }

    /// For `compute.set(..)`.
    pub fn index_data(&self) -> GeometryData {
        GeometryData(ShaderValue::MeshIndex(self.0))
    }
}

impl Drop for Geometry {
    fn drop(&mut self) {
        let _ = geometry_destroy(self.0);
    }
}

#[derive(Debug)]
pub struct GeometryData(ShaderValue);

impl IntoShaderValue for GeometryData {
    fn into_shader_value(self) -> ShaderValue {
        self.0
    }
}

fn color_vec4(color: impl Into<ColorArg>) -> Vec4 {
    let c = ColorState::default().resolve(color.into());
    Vec4::new(c.red(), c.green(), c.blue(), c.alpha())
}

#[derive(Debug)]
pub struct Layout(Entity);

impl Layout {
    pub fn new() -> Result<Self> {
        geometry_layout_create().map(Self)
    }

    pub fn position(self) -> Self {
        expect(geometry_layout_add_position(self.0));
        self
    }

    pub fn normal(self) -> Self {
        expect(geometry_layout_add_normal(self.0));
        self
    }

    pub fn color(self) -> Self {
        expect(geometry_layout_add_color(self.0));
        self
    }

    pub fn uv(self) -> Self {
        expect(geometry_layout_add_uv(self.0));
        self
    }

    pub fn attribute(self, attribute: Attribute) -> Self {
        expect(geometry_layout_add_attribute(self.0, attribute.0));
        self
    }
}

impl Drop for Layout {
    fn drop(&mut self) {
        let _ = geometry_layout_destroy(self.0);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Attribute(pub(crate) Entity);

impl Attribute {
    pub fn new(name: &str, format: AttributeFormat) -> Result<Self> {
        geometry_attribute_create(name, format).map(Self)
    }

    pub fn position() -> Self {
        Self(geometry_attribute_position())
    }

    pub fn normal() -> Self {
        Self(geometry_attribute_normal())
    }

    pub fn color() -> Self {
        Self(geometry_attribute_color())
    }

    pub fn uv() -> Self {
        Self(geometry_attribute_uv())
    }

    pub fn rotation() -> Self {
        Self(geometry_attribute_rotation())
    }

    pub fn scale() -> Self {
        Self(geometry_attribute_scale())
    }

    pub fn life() -> Self {
        Self(geometry_attribute_life())
    }

    pub fn velocity() -> Self {
        Self(geometry_attribute_velocity())
    }

    pub fn age() -> Self {
        Self(geometry_attribute_age())
    }
}

#[doc(hidden)]
pub enum Value {
    Float(f32),
    Float2(Vec2),
    Float3(Vec3),
    Float4(Vec4),
}

/// A number, or a 2-4 tuple or vector.
pub trait AttributeValue {
    #[doc(hidden)]
    fn into_value(self) -> Value;
}

impl<T: Scalar> AttributeValue for T {
    fn into_value(self) -> Value {
        Value::Float(self.to_f32())
    }
}

impl<A: Scalar, B: Scalar> AttributeValue for (A, B) {
    fn into_value(self) -> Value {
        Value::Float2(Vec2::new(self.0.to_f32(), self.1.to_f32()))
    }
}

impl<A: Scalar, B: Scalar, C: Scalar> AttributeValue for (A, B, C) {
    fn into_value(self) -> Value {
        Value::Float3(Vec3::new(self.0.to_f32(), self.1.to_f32(), self.2.to_f32()))
    }
}

impl<A: Scalar, B: Scalar, C: Scalar, D: Scalar> AttributeValue for (A, B, C, D) {
    fn into_value(self) -> Value {
        Value::Float4(Vec4::new(
            self.0.to_f32(),
            self.1.to_f32(),
            self.2.to_f32(),
            self.3.to_f32(),
        ))
    }
}

impl AttributeValue for Vec2 {
    fn into_value(self) -> Value {
        Value::Float2(self)
    }
}

impl AttributeValue for Vec3 {
    fn into_value(self) -> Value {
        Value::Float3(self)
    }
}

impl AttributeValue for Vec4 {
    fn into_value(self) -> Value {
        Value::Float4(self)
    }
}
