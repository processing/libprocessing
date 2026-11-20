use std::path::PathBuf;

use bevy::render::render_resource::TextureFormat;
use processing_core::config::{Config, ConfigKey};
use processing_core::error::Result;
use processing_glfw::GlfwContext;
use processing_render::{advance_frame_count, graphics_create, surface_set_title};

use crate::Graphics;

#[derive(Debug, Clone)]
pub struct Settings {
    width: u32,
    height: u32,
    title: String,
    transparent: bool,
    headless: bool,
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
            headless: false,
            assets: None,
            log_level: None,
        }
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    pub fn transparent(mut self) -> Self {
        self.transparent = true;
        self
    }

    /// No window; runs until [`Graphics::exit`].
    pub fn headless(mut self) -> Self {
        self.headless = true;
        self
    }

    /// The asset directory.
    pub fn assets(mut self, dir: impl Into<PathBuf>) -> Self {
        self.assets = Some(dir.into());
        self
    }

    /// A `tracing` filter, e.g. `"warn"`.
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

pub trait Sketch: Sized {
    fn settings() -> Settings {
        Settings::default()
    }

    fn setup(g: &mut Graphics) -> Result<Self>;

    fn draw(&mut self, _g: &mut Graphics) {}
}

pub fn run<S: Sketch>() -> Result<()> {
    let settings = S::settings();
    let mut window = if settings.headless {
        None
    } else {
        Some(GlfwContext::new(
            settings.width,
            settings.height,
            settings.transparent,
        )?)
    };

    let mut config = Config::new();
    if let Some(dir) = &settings.assets {
        config.set(ConfigKey::AssetRootPath, dir.to_string_lossy().into_owned());
    }
    if let Some(level) = &settings.log_level {
        config.set(ConfigKey::LogLevel, level.clone());
    }
    crate::init(config)?;

    let result = match &mut window {
        Some(window) => {
            open_window(window, &settings).and_then(|g| run_frames::<S>(g, || window.poll_events()))
        }
        None => Graphics::create(settings.width, settings.height)
            .and_then(|g| run_frames::<S>(g, || true)),
    };
    crate::exit(if result.is_ok() { 0 } else { 1 })?;
    result
}

fn open_window(window: &mut GlfwContext, settings: &Settings) -> Result<Graphics> {
    let surface = window.create_surface(settings.width, settings.height, settings.transparent)?;
    let entity = graphics_create(
        surface,
        settings.width,
        settings.height,
        TextureFormat::Rgba16Float,
    )?;
    surface_set_title(surface, settings.title.clone())?;
    Ok(Graphics::new(
        entity,
        surface,
        settings.width,
        settings.height,
    ))
}

fn run_frames<S: Sketch>(mut g: Graphics, mut poll: impl FnMut() -> bool) -> Result<()> {
    if !poll() {
        return Ok(());
    }
    advance_frame_count()?;
    g.begin_frame();
    let mut sketch = S::setup(&mut g)?;
    sketch.draw(&mut g);
    g.end_frame();

    while !g.exit_requested() && poll() {
        advance_frame_count()?;
        g.begin_frame();
        sketch.draw(&mut g);
        g.end_frame();
    }

    // handles must drop while the app is alive
    drop(sketch);
    drop(g);
    Ok(())
}
