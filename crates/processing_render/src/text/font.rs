use std::sync::{Arc, Mutex};

use bevy::prelude::*;
use parley::{FontContext, LayoutContext};

/// A font entity component storing the font's family name.
#[derive(Component)]
pub struct Font {
    pub family_name: String,
}

/// Shared text context resource containing parley's font and layout contexts.
#[derive(Resource, Clone)]
pub struct TextContext {
    inner: Arc<Mutex<TextContextInner>>,
}

struct TextContextInner {
    pub font_cx: FontContext,
    pub layout_cx: LayoutContext<Color>,
}

impl TextContext {
    pub fn new() -> Self {
        let mut font_cx = FontContext::default();

        // Register the embedded NotoSans as default font
        font_cx
            .collection
            .register_fonts(notosans::REGULAR_TTF.to_vec().into(), None);

        Self {
            inner: Arc::new(Mutex::new(TextContextInner {
                font_cx,
                layout_cx: LayoutContext::new(),
            })),
        }
    }

    /// Access the font and layout contexts together via a closure.
    /// We split the struct to avoid double-mutable-borrow issues.
    pub fn with<R>(&self, f: impl FnOnce(&mut FontContext, &mut LayoutContext<Color>) -> R) -> R {
        let mut inner = self.inner.lock().unwrap();
        let TextContextInner {
            ref mut font_cx,
            ref mut layout_cx,
        } = *inner;
        f(font_cx, layout_cx)
    }

    /// Load a font file and register it with the font context.
    /// Returns the primary family name of the loaded font, if available.
    pub fn load_font(&self, data: Vec<u8>) -> Option<String> {
        let mut inner = self.inner.lock().unwrap();
        let families = inner
            .font_cx
            .collection
            .register_fonts(data.into(), None);
        families.first().and_then(|(fam_id, _)| {
            inner
                .font_cx
                .collection
                .family_name(*fam_id)
                .map(|s| s.to_string())
        })
    }

    /// List all available font family names (system + registered).
    pub fn list_fonts(&self) -> Vec<String> {
        let mut inner = self.inner.lock().unwrap();
        inner
            .font_cx
            .collection
            .family_names()
            .map(|s| s.to_string())
            .collect()
    }

    /// Check if a font family name is available.
    pub fn has_font(&self, name: &str) -> bool {
        let mut inner = self.inner.lock().unwrap();
        inner.font_cx.collection.family_id(name).is_some()
    }
}

impl Default for TextContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Info about a variable font axis.
#[derive(Debug, Clone)]
pub struct FontAxisInfo {
    /// Four-character tag (e.g. "wght", "wdth").
    pub tag: String,
    /// Minimum axis value.
    pub min: f32,
    /// Maximum axis value.
    pub max: f32,
    /// Default axis value.
    pub default: f32,
}

/// Font metadata.
#[derive(Debug, Clone, Default)]
pub struct FontMetadata {
    pub family: String,
    pub style: String,
    pub weight: f32,
    pub width: f32,
    pub is_variable: bool,
}

impl TextContext {
    /// Query variable font axes for a given family name.
    pub fn font_variations(&self, family: &str) -> Vec<FontAxisInfo> {
        let mut inner = self.inner.lock().unwrap();
        let family_info = match inner.font_cx.collection.family_by_name(family) {
            Some(f) => f,
            None => return Vec::new(),
        };
        let font_info = match family_info.default_font() {
            Some(f) => f,
            None => return Vec::new(),
        };

        font_info
            .axes()
            .iter()
            .map(|axis| {
                let tag_bytes = axis.tag.to_be_bytes();
                let tag = String::from_utf8_lossy(&tag_bytes).to_string();
                FontAxisInfo {
                    tag,
                    min: axis.min,
                    max: axis.max,
                    default: axis.default,
                }
            })
            .collect()
    }

    /// Query font metadata for a given family name.
    pub fn font_metadata(&self, family: &str) -> Option<FontMetadata> {
        use parley::FontStyle;

        let mut inner = self.inner.lock().unwrap();
        let family_info = inner.font_cx.collection.family_by_name(family)?;
        let font_info = family_info.default_font()?;

        let style = match font_info.style() {
            FontStyle::Normal => "normal".to_string(),
            FontStyle::Italic => "italic".to_string(),
            FontStyle::Oblique(_) => "oblique".to_string(),
        };

        Some(FontMetadata {
            family: family_info.name().to_string(),
            style,
            weight: font_info.weight().value(),
            width: font_info.width().ratio(),
            is_variable: !font_info.axes().is_empty(),
        })
    }
}

pub const DEFAULT_FONT_FAMILY: &str = "Noto Sans";

pub struct TextPlugin;

impl Plugin for TextPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(TextContext::new());
    }
}
