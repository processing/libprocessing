use thiserror::Error;

pub type Result<T> = std::result::Result<T, ProcessingError>;

#[derive(Error, Debug)]
pub enum ProcessingError {
    #[error("App was accessed from multiple threads")]
    AppAccess,
    #[error("Error initializing tracing: {0}")]
    Tracing(#[from] tracing::subscriber::SetGlobalDefaultError),
    #[error("Surface not found")]
    SurfaceNotFound,
    #[error("Handle error: {0}")]
    HandleError(#[from] raw_window_handle::HandleError),
    #[error("Invalid window handle provided")]
    InvalidWindowHandle,
    #[error("Image not found")]
    ImageNotFound,
    #[error("Unsupported texture format")]
    UnsupportedTextureFormat,
    #[error("Invalid argument: {0}")]
    InvalidArgument(String),
    #[error("Graphics not found")]
    GraphicsNotFound,
    #[error("Invalid entity")]
    InvalidEntity,
    #[error("Geometry not found")]
    GeometryNotFound,
    #[error("Layout not found")]
    LayoutNotFound,
    #[error("Transform not found")]
    TransformNotFound,
    #[error("Material not found")]
    MaterialNotFound,
    #[error("Unknown material property: {0}")]
    UnknownMaterialProperty(String),
}
