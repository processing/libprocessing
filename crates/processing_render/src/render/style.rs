use std::ops::{Deref, DerefMut};

use bevy::{math::Affine2, prelude::*, render::render_resource::BlendState};

use super::command::{ShapeMode, TextAlignH, TextAlignV, TextStyle, TextWrapMode};
use super::material::MaterialKey;
use super::primitive::StrokeConfig;

#[derive(Debug, Clone, Copy)]
pub enum Fill {
    None,
    Color(Color),
    /// Per-instance albedo buffer for [`Particles`](crate::particles::Particles) draws.
    Buffer(Entity),
}

impl Fill {
    pub fn color(self) -> Option<Color> {
        match self {
            Fill::Color(color) => Some(color),
            Fill::None | Fill::Buffer(_) => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Style {
    pub fill: Fill,
    pub stroke_color: Option<Color>,
    pub stroke_weight: f32,
    pub stroke_config: StrokeConfig,
    pub material_key: MaterialKey,
    pub blend_state: Option<BlendState>,
    pub tint_color: Option<Color>,
    pub image_mode: ShapeMode,
    pub rect_mode: ShapeMode,
    pub ellipse_mode: ShapeMode,
    pub text_font_family: Option<String>,
    pub text_style: TextStyle,
    pub text_weight: Option<f32>,
    pub text_variations: Vec<([u8; 4], f32)>,
    pub text_features: Vec<([u8; 4], u16)>,
    pub text_size: f32,
    pub text_align_h: TextAlignH,
    pub text_align_v: TextAlignV,
    pub text_leading: Option<f32>,
    pub text_wrap: TextWrapMode,
    pub text_glyph_colors: Option<Vec<Color>>,
}

impl Default for Style {
    fn default() -> Self {
        Self {
            fill: Fill::Color(Color::WHITE),
            stroke_color: Some(Color::BLACK),
            stroke_weight: 1.0,
            stroke_config: StrokeConfig::default(),
            material_key: MaterialKey::Color {
                transparent: false,
                background_image: None,
                uv_transform: Affine2::IDENTITY,
                blend_state: None,
            },
            blend_state: None,
            tint_color: None,
            image_mode: ShapeMode::Corner,
            rect_mode: ShapeMode::Corner,
            ellipse_mode: ShapeMode::Center,
            text_font_family: None,
            text_style: TextStyle::Normal,
            text_weight: None,
            text_variations: Vec::new(),
            text_features: Vec::new(),
            text_size: 12.0,
            text_align_h: TextAlignH::Left,
            text_align_v: TextAlignV::Baseline,
            text_leading: None,
            text_wrap: TextWrapMode::Word,
            text_glyph_colors: None,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct StyleStack {
    current: Style,
    stack: Vec<Style>,
}

impl StyleStack {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self) {
        self.stack.push(self.current.clone());
    }

    pub fn pop(&mut self) {
        if let Some(style) = self.stack.pop() {
            self.current = style;
        }
    }

    /// The current style persists across frames; only the saved stack is per-frame.
    pub fn clear_saved(&mut self) {
        self.stack.clear();
    }
}

impl Deref for StyleStack {
    type Target = Style;

    fn deref(&self) -> &Style {
        &self.current
    }
}

impl DerefMut for StyleStack {
    fn deref_mut(&mut self) -> &mut Style {
        &mut self.current
    }
}
