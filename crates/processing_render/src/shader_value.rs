use bevy::prelude::*;

#[derive(Debug, Clone)]
pub enum ShaderValue {
    Float(f32),
    Float2([f32; 2]),
    Float3([f32; 3]),
    Float4([f32; 4]),
    Int(i32),
    Int2([i32; 2]),
    Int3([i32; 3]),
    Int4([i32; 4]),
    UInt(u32),
    Mat4([f32; 16]),
    Texture(Entity),
    Buffer(Entity),
}

impl ShaderValue {
    pub fn to_bytes(&self) -> Option<Vec<u8>> {
        match self {
            ShaderValue::Float(v) => Some(v.to_le_bytes().to_vec()),
            ShaderValue::Float2(v) => Some(v.iter().flat_map(|f| f.to_le_bytes()).collect()),
            ShaderValue::Float3(v) => Some(v.iter().flat_map(|f| f.to_le_bytes()).collect()),
            ShaderValue::Float4(v) => Some(v.iter().flat_map(|f| f.to_le_bytes()).collect()),
            ShaderValue::Int(v) => Some(v.to_le_bytes().to_vec()),
            ShaderValue::Int2(v) => Some(v.iter().flat_map(|i| i.to_le_bytes()).collect()),
            ShaderValue::Int3(v) => Some(v.iter().flat_map(|i| i.to_le_bytes()).collect()),
            ShaderValue::Int4(v) => Some(v.iter().flat_map(|i| i.to_le_bytes()).collect()),
            ShaderValue::UInt(v) => Some(v.to_le_bytes().to_vec()),
            ShaderValue::Mat4(v) => Some(v.iter().flat_map(|f| f.to_le_bytes()).collect()),
            ShaderValue::Texture(_) | ShaderValue::Buffer(_) => None,
        }
    }

    pub fn byte_size(&self) -> Option<usize> {
        match self {
            ShaderValue::Float(_) | ShaderValue::Int(_) | ShaderValue::UInt(_) => Some(4),
            ShaderValue::Float2(_) | ShaderValue::Int2(_) => Some(8),
            ShaderValue::Float3(_) | ShaderValue::Int3(_) => Some(12),
            ShaderValue::Float4(_) | ShaderValue::Int4(_) => Some(16),
            ShaderValue::Mat4(_) => Some(64),
            ShaderValue::Texture(_) | ShaderValue::Buffer(_) => None,
        }
    }

    pub fn read_from_bytes(&self, bytes: &[u8]) -> Option<ShaderValue> {
        fn f32s<const N: usize>(bytes: &[u8]) -> Option<[f32; N]> {
            let mut arr = [0f32; N];
            for i in 0..N {
                arr[i] = f32::from_le_bytes(bytes[i * 4..(i + 1) * 4].try_into().ok()?);
            }
            Some(arr)
        }
        fn i32s<const N: usize>(bytes: &[u8]) -> Option<[i32; N]> {
            let mut arr = [0i32; N];
            for i in 0..N {
                arr[i] = i32::from_le_bytes(bytes[i * 4..(i + 1) * 4].try_into().ok()?);
            }
            Some(arr)
        }
        match self {
            ShaderValue::Float(_) => Some(ShaderValue::Float(f32::from_le_bytes(
                bytes[..4].try_into().ok()?,
            ))),
            ShaderValue::Float2(_) => Some(ShaderValue::Float2(f32s::<2>(bytes)?)),
            ShaderValue::Float3(_) => Some(ShaderValue::Float3(f32s::<3>(bytes)?)),
            ShaderValue::Float4(_) => Some(ShaderValue::Float4(f32s::<4>(bytes)?)),
            ShaderValue::Int(_) => Some(ShaderValue::Int(i32::from_le_bytes(
                bytes[..4].try_into().ok()?,
            ))),
            ShaderValue::Int2(_) => Some(ShaderValue::Int2(i32s::<2>(bytes)?)),
            ShaderValue::Int3(_) => Some(ShaderValue::Int3(i32s::<3>(bytes)?)),
            ShaderValue::Int4(_) => Some(ShaderValue::Int4(i32s::<4>(bytes)?)),
            ShaderValue::UInt(_) => Some(ShaderValue::UInt(u32::from_le_bytes(
                bytes[..4].try_into().ok()?,
            ))),
            ShaderValue::Mat4(_) => Some(ShaderValue::Mat4(f32s::<16>(bytes)?)),
            ShaderValue::Texture(_) | ShaderValue::Buffer(_) => None,
        }
    }
}
