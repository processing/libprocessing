//! A Sketch asset represents a source file containing user code for a Processing sketch.

use bevy::{
    asset::{
        AssetLoader, AssetPath, LoadContext,
        io::{AssetSourceId, Reader},
    },
    prelude::*,
};
use std::path::Path;

/// Plugin that registers the Sketch asset type and its loader.
pub struct LivecodePlugin;

impl Plugin for LivecodePlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<Sketch>()
            .init_asset_loader::<SketchLoader>();

        app.add_systems(PreStartup, load_current_sketch)
            .add_systems(Update, sketch_update_handler);
    }
}

// TODO: A better name is possible
fn sketch_update_handler(mut events: MessageReader<AssetEvent<Sketch>>) {
    for event in events.read() {
        match event {
            AssetEvent::Added { id } => {
                info!("Added: {id}")
            }
            AssetEvent::Modified { id } => {
                info!("Modified: {id}")
            }
            AssetEvent::Removed { id } => {
                info!("Removed: {id}")
            }
            AssetEvent::Unused { id } => {
                info!("Unused: {id}")
            }
            AssetEvent::LoadedWithDependencies { id } => {
                info!("LoadedWithDependencies: {id}")
            }
        }
    }
}

fn load_current_sketch(mut commands: Commands, asset_server: Res<AssetServer>) {
    info!("DEBUG: calling load_current_sketch");
    let path = Path::new("rectangle.py");
    let source = AssetSourceId::from("sketch_directory");
    let asset_path = AssetPath::from_path(path).with_source(source);
    let sketch_handle: Handle<Sketch> = asset_server.load(asset_path);
    commands.spawn(SketchRoot(sketch_handle));
}

/// `SketchRoot` is what will be spawned and will contain a `Handle` to the `Sketch`
#[derive(Component)]
pub struct SketchRoot(pub Handle<Sketch>);

/// A sketch source file loaded as a Bevy asset.
///
/// The `Sketch` asset contains the raw source code as a string. It does not interpret
/// or execute the code — that responsibility belongs to language-specific crates.
#[derive(Asset, TypePath, Debug)]
pub struct Sketch {
    source: String,
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
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut source = String::new();

        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;

        if let Ok(utf8) = str::from_utf8(&bytes) {
            source = utf8.to_string();
        }

        info!(source);

        Ok(Sketch { source })
    }

    fn extensions(&self) -> &[&str] {
        &["py"]
    }
}
