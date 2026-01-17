use std::sync::Arc;

use super::database::DatabaseInstance;

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<DatabaseInstance>,
}