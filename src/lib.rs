pub mod prelude;

use std::num::NonZero;

use bevy::app::{App, AppExit};
use bevy::prelude::*;
use bevy::render::RenderPlugin;

use processing_core::config::{Config, ConfigKey};
use processing_core::error;

fn create_app(config: Config) -> App {
    let mut app = App::new();

    app.insert_resource(config.clone());

    #[cfg(not(target_arch = "wasm32"))]
    let plugins = DefaultPlugins
        .build()
        .set(RenderPlugin {
            synchronous_pipeline_compilation: true,
            ..default()
        })
        .disable::<bevy::winit::WinitPlugin>()
        .disable::<bevy::log::LogPlugin>()
        .disable::<bevy::render::pipelined_rendering::PipelinedRenderingPlugin>()
        .set(WindowPlugin {
            primary_window: None,
            exit_condition: bevy::window::ExitCondition::DontExit,
            ..default()
        });

    #[cfg(target_arch = "wasm32")]
    let plugins = DefaultPlugins
        .build()
        .disable::<bevy::winit::WinitPlugin>()
        .disable::<bevy::log::LogPlugin>()
        .set(WindowPlugin {
            primary_window: None,
            exit_condition: bevy::window::ExitCondition::DontExit,
            ..default()
        });

    app.add_plugins(plugins);
    app.add_plugins(processing_midi::MidiPlugin);
    app.add_plugins(processing_input::InputPlugin);
    app.add_plugins(processing_render::ProcessingRenderPlugin);

    #[cfg(feature = "cuda")]
    app.add_plugins(processing_cuda::CudaPlugin);

    #[cfg(feature = "webcam")]
    app.add_plugins(processing_webcam::WebcamPlugin);

    app
}

/// Initialize the app, if not already initialized. Must be called from the main thread and cannot
/// be called concurrently from multiple threads.
#[cfg(not(target_arch = "wasm32"))]
pub fn init(config: Config) -> error::Result<()> {
    if processing_core::is_already_init()? {
        return Ok(());
    }
    setup_tracing(config.get(ConfigKey::LogLevel).map(|s| s.as_str()))?;

    let mut app = create_app(config);
    // contrary to what the following methods might imply, this is just finishing plugin setup
    // which normally happens in the app runner (i.e. in a "normal" bevy app), but since we don't
    // have one we need to do it manually here
    app.finish();
    app.cleanup();
    // also, we need to run the main schedule once to ensure all systems are initialized before we
    // return from init, to ensure any plugins that need to do setup in their first update can rely
    // on that
    app.update();
    processing_core::set_app(app);

    Ok(())
}

/// Initialize the app asynchronously
#[cfg(target_arch = "wasm32")]
pub async fn init(config: Config) -> error::Result<()> {
    use bevy::app::PluginsState;

    if processing_core::is_already_init()? {
        return Ok(());
    }
    setup_tracing(config.get(ConfigKey::LogLevel).map(|s| s.as_str()))?;

    let mut app = create_app(config);

    // we need to avoid blocking the main thread while waiting for plugins to initialize
    while app.plugins_state() == PluginsState::Adding {
        // yield to event loop
        wasm_bindgen_futures::JsFuture::from(js_sys::Promise::new(&mut |resolve, _| {
            web_sys::window()
                .unwrap()
                .set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, 0)
                .unwrap();
        }))
        .await
        .unwrap();
    }

    app.finish();
    app.cleanup();
    app.update();
    processing_core::set_app(app);

    Ok(())
}

pub fn exit(exit_code: u8) -> error::Result<()> {
    processing_core::app_mut(|app| {
        app.world_mut().write_message(match exit_code {
            0 => AppExit::Success,
            _ => AppExit::Error(NonZero::new(exit_code).unwrap()),
        });

        // one final update to process the exit message
        app.update();
        Ok(())
    })?;

    // we need to drop the app in a deterministic manner to ensure resources are cleaned up
    // otherwise we'll get wgpu graphics backend errors on exit
    drop(processing_core::take_app());

    Ok(())
}

#[cfg(feature = "cuda")]
pub use processing_cuda;

fn setup_tracing(log_level: Option<&str>) -> error::Result<()> {
    // TODO: figure out wasm compatible tracing subscriber
    #[cfg(not(target_arch = "wasm32"))]
    {
        use tracing_subscriber::EnvFilter;

        let filter = EnvFilter::try_new(log_level.unwrap_or("info"))
            .unwrap_or_else(|_| EnvFilter::new("info"));
        let subscriber = tracing_subscriber::FmtSubscriber::builder()
            .with_env_filter(filter)
            .finish();
        tracing::subscriber::set_global_default(subscriber)?;
    }
    Ok(())
}
