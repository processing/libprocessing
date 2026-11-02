//! The sketch runner: opens a window and calls `setup` once and `draw` every frame.

use std::path::PathBuf;

use bevy::render::render_resource::TextureFormat;
use processing_core::config::{Config, ConfigKey};
use processing_core::error::Result;
use processing_glfw::GlfwContext;
use processing_render::{advance_frame_count, graphics_create, surface_set_title};

use crate::Graphics;

/// Window and app options, returned from [`Sketch::settings`].
#[derive(Debug, Clone)]
pub struct Settings {
    width: u32,
    height: u32,
    title: String,
    transparent: bool,
    assets: Option<PathBuf>,
    log_level: Option<String>,
}

impl Settings {
    pub fn size(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            title: "Processing".to_string(),
            transparent: false,
            assets: None,
            log_level: None,
        }
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    /// A window with a transparent background.
    pub fn transparent(mut self) -> Self {
        self.transparent = true;
        self
    }

    /// Where `Image::load` and `Font::load` look for files.
    pub fn assets(mut self, dir: impl Into<PathBuf>) -> Self {
        self.assets = Some(dir.into());
        self
    }

    /// A `tracing` filter such as `"warn"` or `"processing=debug"`.
    pub fn log_level(mut self, level: impl Into<String>) -> Self {
        self.log_level = Some(level.into());
        self
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self::size(100, 100)
    }
}

/// A Processing sketch. State lives on `Self`.
pub trait Sketch: Sized {
    fn settings() -> Settings {
        Settings::default()
    }

    /// Runs once, before the first `draw`, with the window ready to draw into.
    fn setup(g: &mut Graphics) -> Result<Self>;

    fn draw(&mut self, _g: &mut Graphics) {}
}

/// Runs `S` until its window closes.
pub fn run<S: Sketch>() -> Result<()> {
    let settings = S::settings();
    let mut window = GlfwContext::new(settings.width, settings.height, settings.transparent)?;

    let mut config = Config::new();
    if let Some(dir) = &settings.assets {
        config.set(ConfigKey::AssetRootPath, dir.to_string_lossy().into_owned());
    }
    if let Some(level) = &settings.log_level {
        config.set(ConfigKey::LogLevel, level.clone());
    }
    crate::init(config)?;

    let result = run_loop::<S>(&mut window, &settings);
    crate::exit(if result.is_ok() { 0 } else { 1 })?;
    result
}

fn run_loop<S: Sketch>(window: &mut GlfwContext, settings: &Settings) -> Result<()> {
    let surface = window.create_surface(settings.width, settings.height, settings.transparent)?;
    let entity = graphics_create(
        surface,
        settings.width,
        settings.height,
        TextureFormat::Rgba16Float,
    )?;
    surface_set_title(surface, settings.title.clone())?;
    let mut g = Graphics::new(entity, surface, settings.width, settings.height);

    if !window.poll_events() {
        return Ok(());
    }
    advance_frame_count()?;
    g.begin_frame();
    let mut sketch = S::setup(&mut g)?;
    sketch.draw(&mut g);
    g.end_frame();

    while !g.exit_requested() && window.poll_events() {
        advance_frame_count()?;
        g.begin_frame();
        sketch.draw(&mut g);
        g.end_frame();
    }

    // handles release engine resources, so drop them while the app is still alive
    drop(sketch);
    drop(g);
    Ok(())
}
