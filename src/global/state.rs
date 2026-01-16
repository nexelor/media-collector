use std::sync::Arc;

use crate::global::database::DatabaseInstance;

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<DatabaseInstance>,
}