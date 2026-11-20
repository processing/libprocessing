use bevy::color::{ColorToComponents, LinearRgba};
use bevy::math::{Mat4, Vec2, Vec3, Vec4};
use bevy::prelude::Entity;
use processing_core::error::Result;
use processing_render::shader_value::ShaderValue;
use processing_render::{
    buffer_create, buffer_create_with_data, buffer_destroy, buffer_read, buffer_size, buffer_write,
    compute_create, compute_destroy, compute_dispatch, compute_set, material_create_custom,
    material_create_pbr, material_create_unlit, material_destroy, material_set,
    material_set_albedo_buffer, material_set_double_sided, material_set_emissive_buffer,
    shader_create, shader_destroy, shader_load,
};

use crate::graphics::expect;
use crate::{AsImage, Color};

/// WGSL or WESL.
#[derive(Debug)]
pub struct Shader(pub(crate) Entity);

impl Shader {
    pub fn new(source: &str) -> Result<Self> {
        shader_create(source).map(Self)
    }

    pub fn load(path: &str) -> Result<Self> {
        shader_load(path).map(Self)
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        let _ = shader_destroy(self.0);
    }
}

#[derive(Debug)]
pub struct Material(pub(crate) Entity);

impl Material {
    pub fn pbr() -> Result<Self> {
        material_create_pbr().map(Self)
    }

    pub fn unlit() -> Result<Self> {
        material_create_unlit().map(Self)
    }

    pub fn new(shader: &Shader) -> Result<Self> {
        material_create_custom(shader.0).map(Self)
    }

    pub fn set(&mut self, name: &str, value: impl IntoShaderValue) {
        expect(material_set(self.0, name, value.into_shader_value()));
    }

    /// One RGBA per particle.
    pub fn albedo_buffer(&mut self, buffer: &Buffer) {
        expect(material_set_albedo_buffer(self.0, buffer.entity));
    }

    /// One RGBA per particle.
    pub fn emissive_buffer(&mut self, buffer: &Buffer) {
        expect(material_set_emissive_buffer(self.0, buffer.entity));
    }

    pub fn double_sided(&mut self, value: bool) {
        expect(material_set_double_sided(self.0, value));
    }
}

impl Drop for Material {
    fn drop(&mut self) {
        let _ = material_destroy(self.0);
    }
}

#[derive(Debug)]
pub struct Buffer {
    pub(crate) entity: Entity,
    owned: bool,
}

impl Buffer {
    /// Zeroed, `size` in bytes.
    pub fn new(size: u64) -> Result<Self> {
        buffer_create(size).map(Self::owned)
    }

    pub fn from_slice<T: BufferElement>(data: &[T]) -> Result<Self> {
        buffer_create_with_data(T::to_bytes(data)).map(Self::owned)
    }

    fn owned(entity: Entity) -> Self {
        Self {
            entity,
            owned: true,
        }
    }

    /// Not destroyed on drop.
    pub(crate) fn borrowed(entity: Entity) -> Self {
        Self {
            entity,
            owned: false,
        }
    }

    /// In bytes.
    pub fn size(&self) -> u64 {
        expect(buffer_size(self.entity))
    }

    pub fn write<T: BufferElement>(&mut self, data: &[T]) {
        expect(buffer_write(self.entity, T::to_bytes(data)));
    }

    pub fn read<T: BufferElement>(&self) -> Result<Vec<T>> {
        Ok(T::from_bytes(&buffer_read(self.entity)?))
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        if self.owned {
            let _ = buffer_destroy(self.entity);
        }
    }
}

mod sealed {
    pub trait Sealed {}
}

/// `f32`, `i32`, `u32`, arrays of 2-4 of them, `Vec2`-`Vec4`, or `Color` (linear RGBA).
pub trait BufferElement: sealed::Sealed + Sized {
    #[doc(hidden)]
    const FLOAT: bool;
    #[doc(hidden)]
    const COMPONENTS: usize;
    #[doc(hidden)]
    fn to_bytes(data: &[Self]) -> Vec<u8>;
    #[doc(hidden)]
    fn from_bytes(bytes: &[u8]) -> Vec<Self>;
}

macro_rules! pod_element {
    ($($t:ty => $float:expr, $n:expr);* $(;)?) => {$(
        impl sealed::Sealed for $t {}
        impl BufferElement for $t {
            const FLOAT: bool = $float;
            const COMPONENTS: usize = $n;
            fn to_bytes(data: &[Self]) -> Vec<u8> {
                bytemuck::cast_slice(data).to_vec()
            }
            fn from_bytes(bytes: &[u8]) -> Vec<Self> {
                bytemuck::pod_collect_to_vec(bytes)
            }
        }
    )*};
}

macro_rules! buffer_element {
    ($($t:ty => $float:expr),*) => {$(
        pod_element!($t => $float, 1; [$t; 2] => $float, 2; [$t; 3] => $float, 3; [$t; 4] => $float, 4);
    )*};
}
buffer_element!(f32 => true, i32 => false, u32 => false);
pod_element!(Vec2 => true, 2; Vec3 => true, 3; Vec4 => true, 4);

impl sealed::Sealed for Color {}
impl BufferElement for Color {
    const FLOAT: bool = true;
    const COMPONENTS: usize = 4;
    fn to_bytes(data: &[Self]) -> Vec<u8> {
        let linear: Vec<[f32; 4]> = data.iter().map(|c| c.to_linear().to_f32_array()).collect();
        bytemuck::cast_slice(&linear).to_vec()
    }
    fn from_bytes(bytes: &[u8]) -> Vec<Self> {
        bytemuck::pod_collect_to_vec::<u8, [f32; 4]>(bytes)
            .into_iter()
            .map(|c| Color::from_linear(LinearRgba::from_f32_array(c)))
            .collect()
    }
}

#[derive(Debug)]
pub struct Compute(pub(crate) Entity);

impl Compute {
    pub fn new(shader: &Shader) -> Result<Self> {
        compute_create(shader.0).map(Self)
    }

    pub fn set(&mut self, name: &str, value: impl IntoShaderValue) {
        expect(compute_set(self.0, name, value.into_shader_value()));
    }

    /// Blocks until done.
    pub fn dispatch(&mut self, x: u32, y: u32, z: u32) {
        expect(compute_dispatch(self.0, x, y, z));
    }
}

impl Drop for Compute {
    fn drop(&mut self) {
        let _ = compute_destroy(self.0);
    }
}

pub trait IntoShaderValue {
    #[doc(hidden)]
    fn into_shader_value(self) -> ShaderValue;
}

impl IntoShaderValue for f32 {
    fn into_shader_value(self) -> ShaderValue {
        ShaderValue::Float(self)
    }
}

impl IntoShaderValue for f64 {
    fn into_shader_value(self) -> ShaderValue {
        ShaderValue::Float(self as f32)
    }
}

impl IntoShaderValue for i32 {
    fn into_shader_value(self) -> ShaderValue {
        ShaderValue::Int(self)
    }
}

impl IntoShaderValue for u32 {
    fn into_shader_value(self) -> ShaderValue {
        ShaderValue::UInt(self)
    }
}

impl IntoShaderValue for bool {
    fn into_shader_value(self) -> ShaderValue {
        ShaderValue::UInt(self as u32)
    }
}

impl IntoShaderValue for (f32, f32) {
    fn into_shader_value(self) -> ShaderValue {
        ShaderValue::Float2([self.0, self.1])
    }
}

impl IntoShaderValue for (f32, f32, f32) {
    fn into_shader_value(self) -> ShaderValue {
        ShaderValue::Float3([self.0, self.1, self.2])
    }
}

impl IntoShaderValue for (f32, f32, f32, f32) {
    fn into_shader_value(self) -> ShaderValue {
        ShaderValue::Float4([self.0, self.1, self.2, self.3])
    }
}

impl IntoShaderValue for Vec2 {
    fn into_shader_value(self) -> ShaderValue {
        ShaderValue::Float2(self.to_array())
    }
}

impl IntoShaderValue for Vec3 {
    fn into_shader_value(self) -> ShaderValue {
        ShaderValue::Float3(self.to_array())
    }
}

impl IntoShaderValue for Vec4 {
    fn into_shader_value(self) -> ShaderValue {
        ShaderValue::Float4(self.to_array())
    }
}

impl IntoShaderValue for Mat4 {
    fn into_shader_value(self) -> ShaderValue {
        ShaderValue::Mat4(self.to_cols_array())
    }
}

impl IntoShaderValue for &Buffer {
    fn into_shader_value(self) -> ShaderValue {
        ShaderValue::Buffer(self.entity)
    }
}

impl<T: AsImage> IntoShaderValue for &T {
    fn into_shader_value(self) -> ShaderValue {
        ShaderValue::Texture(self.image_entity())
    }
}
