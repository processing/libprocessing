pub mod config;
pub mod error;

use std::cell::RefCell;
use std::sync::OnceLock;

use bevy::app::App;
use tracing::debug;

static IS_INIT: OnceLock<()> = OnceLock::new();

thread_local! {
    static APP: RefCell<Option<App>> = const { RefCell::new(None) };
}

pub fn app_mut<T>(cb: impl FnOnce(&mut App) -> error::Result<T>) -> error::Result<T> {
    // `try_with` rather than `with` so callers (especially `Drop`s running
    // during pyo3 module teardown) get a graceful error instead of a panic
    // when the thread-local has already been destroyed.
    let res = APP.try_with(|app_cell| {
        let mut app_borrow = app_cell
            .try_borrow_mut()
            .map_err(|_| error::ProcessingError::AppAccess)?;
        let app = app_borrow
            .as_mut()
            .ok_or(error::ProcessingError::AppAccess)?;
        cb(app)
    });
    match res {
        Ok(inner) => inner,
        Err(_) => Err(error::ProcessingError::AppAccess),
    }
}

pub fn is_already_init() -> error::Result<bool> {
    let is_init = IS_INIT.get().is_some();
    let thread_has_app = APP.with(|app_cell| app_cell.borrow().is_some());
    if is_init && !thread_has_app {
        return Err(error::ProcessingError::AppAccess);
    }
    if is_init && thread_has_app {
        debug!("App already initialized");
        return Ok(true);
    }
    Ok(false)
}

pub fn set_app(app: App) {
    APP.with(|app_cell| {
        IS_INIT.get_or_init(|| ());
        *app_cell.borrow_mut() = Some(app);
    });
}

pub fn take_app() -> Option<App> {
    APP.with(|app_cell| app_cell.borrow_mut().take())
}
