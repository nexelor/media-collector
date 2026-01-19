pub mod fetch_anime;
pub mod search_anime;
pub mod update_anime;
pub mod batch_fetch;
pub mod fetch_extended;
pub mod fetch_pictures_for_anime;

// Re-export task types
pub use fetch_anime::FetchAnimeTask;
pub use search_anime::SearchAnimeTask;
pub use update_anime::UpdateAnimeTask;
pub use batch_fetch::BatchFetchTask;
pub use fetch_extended::{
    FetchCharactersTask,
    FetchStaffTask,
    FetchEpisodesTask,
    FetchVideosTask,        // NEW
    FetchStatisticsTask,    // NEW
    FetchMoreInfoTask,      // NEW
    FetchRecommendationsTask, // NEW
    FetchPicturesTask,      // NEW
};
pub use fetch_pictures_for_anime::FetchAnimePicturesTask;