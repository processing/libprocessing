//! A Sketch asset represents a source file containing user code for a Processing sketch.

use bevy::{
    asset::{
        AssetLoader, AssetPath, LoadContext,
        io::{AssetSourceId, Reader},
    },
    prelude::*,
};
use std::path::{Path, PathBuf};

/// Plugin that registers the Sketch asset type and its loader.
pub struct LivecodePlugin;

impl Plugin for LivecodePlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<Sketch>()
            .init_asset_loader::<SketchLoader>()
            .add_systems(Startup, load_current_sketch);
        // .add_systems(Update, test_system);
        let render_app = app.sub_app_mut(bevy::render::RenderApp);
        render_app.add_systems(ExtractSchedule, test_system);
    }
}

fn test_system() {
    info!("DEBUG: calling test_system");
    assert!(false);
}

fn load_current_sketch(asset_server: Res<AssetServer>) {
    info!("DEBUG: calling load_current_sketch");
    let path = Path::new("rectangle.py");
    let source = AssetSourceId::from("sketch_directory");
    let asset_path = AssetPath::from_path(path).with_source(source);
    let _h: Handle<Sketch> = asset_server.load(asset_path);
}

/// A sketch source file loaded as a Bevy asset.
///
/// The `Sketch` asset contains the raw source code as a string. It does not interpret
/// or execute the code — that responsibility belongs to language-specific crates.
#[derive(Asset, TypePath, Debug)]
pub struct Sketch {
    /// The source code contents of the sketch file.
    pub source: String,

    /// The original file path.
    pub path: PathBuf,
}

/// Loads sketch files from disk.
///
/// Currently supports `.py` files, but the loader is designed to be extended
/// for other languages in the future.
#[derive(Default)]
pub struct SketchLoader;

impl AssetLoader for SketchLoader {
    type Asset = Sketch;
    type Settings = ();
    type Error = std::io::Error;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut source = String::new();

        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;

        if let Ok(utf8) = str::from_utf8(&bytes) {
            source = utf8.to_string();
        }

        let asset_path = load_context.path();
        let path: PathBuf = asset_path.path().to_path_buf();

        Ok(Sketch { source, path })
    }

    fn extensions(&self) -> &[&str] {
        &["py"]
    }
}
