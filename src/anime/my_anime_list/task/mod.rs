pub mod fetch_anime;
pub mod search_anime;
pub mod update_anime;
pub mod batch_fetch;
pub mod fetch_extended;

// Re-export task types
pub use fetch_anime::FetchAnimeTask;
pub use search_anime::SearchAnimeTask;
pub use update_anime::UpdateAnimeTask;
pub use batch_fetch::BatchFetchTask;
pub use fetch_extended::{
    FetchCharactersTask,
    FetchStaffTask,
    FetchEpisodesTask,
};