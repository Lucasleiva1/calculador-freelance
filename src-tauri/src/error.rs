use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("{0}")]
    Validation(String),
    #[error("La información cambió en otra operación. Recargá e intentá nuevamente.")]
    RevisionConflict,
    #[error("No se encontró el registro solicitado.")]
    NotFound,
    #[error("Error de persistencia local: {0}")]
    Database(#[from] sqlx::Error),
    #[error("No se pudo actualizar el esquema local: {0}")]
    Migration(#[from] sqlx::migrate::MigrateError),
    #[error("Error de datos: {0}")]
    Json(#[from] serde_json::Error),
    #[error("No se pudo preparar el directorio de datos: {0}")]
    Io(#[from] std::io::Error),
}

pub type AppResult<T> = Result<T, AppError>;

pub fn command_error(error: AppError) -> String {
    error.to_string()
}
