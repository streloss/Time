pub mod lyrics;
pub mod matcher;
pub mod spotify;
pub mod ytmusic;

use crate::models::{LyricLine, Track};
use std::sync::Arc;

pub struct ProviderManager {
    pub ytmusic: Arc<ytmusic::YtMusicProvider>,
    pub spotify: Arc<spotify::SpotifyProvider>,
    pub lyrics: Arc<lyrics::LyricsProvider>,
}

impl Default for ProviderManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ProviderManager {
    pub fn new() -> Self {
        Self {
            ytmusic: Arc::new(ytmusic::YtMusicProvider::new()),
            spotify: Arc::new(spotify::SpotifyProvider::new()),
            lyrics: Arc::new(lyrics::LyricsProvider::new()),
        }
    }

    /// Unified search across YouTube Music and Spotify with smart deduplication.
    pub async fn search_tracks(&self, query: &str) -> Result<Vec<Track>, String> {
        let q = query.trim();
        if q.is_empty() {
            return Ok(ytmusic::YtMusicProvider::get_default_tracks());
        }

        // Concurrently search YouTube Music and Spotify
        let (yt_res, sp_res) = tokio::join!(
            self.ytmusic.search(q),
            self.spotify.search(q)
        );

        let yt_tracks = yt_res.unwrap_or_default();
        let sp_tracks = sp_res.unwrap_or_default();

        if yt_tracks.is_empty() && sp_tracks.is_empty() {
            // Fallback filtered default tracks
            let mut def = ytmusic::YtMusicProvider::get_default_tracks();
            def.retain(|t| {
                t.title.to_lowercase().contains(&q.to_lowercase())
                    || t.artist.to_lowercase().contains(&q.to_lowercase())
            });
            return Ok(if def.is_empty() {
                ytmusic::YtMusicProvider::get_default_tracks()
            } else {
                def
            });
        }

        let mut unified: Vec<Track> = Vec::new();

        // Add YouTube tracks first (as they have playable IDs)
        for mut yt in yt_tracks {
            // Enrich with Spotify album art or metadata if matched
            if let Some(sp) = matcher::find_best_match(&yt, &sp_tracks, 0.7) {
                if yt.cover_url.is_none() || yt.cover_url.as_deref().unwrap_or("").contains("hqdefault") {
                    if let Some(sp_cover) = &sp.cover_url {
                        yt.cover_url = Some(sp_cover.clone());
                    }
                }
                if yt.album.is_none() {
                    yt.album = sp.album.clone();
                }
            }
            unified.push(yt);
        }

        // For any Spotify tracks not yet represented in YouTube Music, add them
        for sp in sp_tracks {
            let already_exists = unified.iter().any(|u| matcher::is_match(u, &sp));
            if !already_exists {
                unified.push(sp);
            }
        }

        Ok(unified)
    }

    /// Fetches synchronized or plain lyrics for a track.
    pub async fn get_lyrics(
        &self,
        title: &str,
        artist: &str,
        album: Option<&str>,
        duration: Option<u64>,
    ) -> Result<Vec<LyricLine>, String> {
        self.lyrics.fetch_lyrics(title, artist, album, duration).await
    }

    /// Resolves an audio URL or stream for a track.
    pub async fn resolve_audio_url(&self, track: &Track) -> Result<String, String> {
        if let Some(existing_url) = &track.audio_url {
            if !existing_url.is_empty() {
                return Ok(existing_url.clone());
            }
        }

        // Try using track id as YouTube videoId
        if !track.id.is_empty() && track.id.len() >= 10 {
            if let Ok(url) = self.ytmusic.get_stream_url(&track.id).await {
                return Ok(url);
            }
        }

        // Search YouTube Music specifically for this track to get a valid stream
        let query = format!("{} {}", track.title, track.artist);
        let results = self.ytmusic.search(&query).await?;
        if let Some(best) = matcher::find_best_match(track, &results, 0.50).or_else(|| results.first()) {
            if let Ok(url) = self.ytmusic.get_stream_url(&best.id).await {
                return Ok(url);
            }
        }

        // Fallback default sample audio stream if streaming resolution cannot contact YouTube
        Ok("https://www.soundhelix.com/examples/mp3/SoundHelix-Song-1.mp3".to_string())
    }
}
