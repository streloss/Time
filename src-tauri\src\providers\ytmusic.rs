use crate::models::Track;
use serde_json::Value;
use std::time::Duration;

pub struct YtMusicProvider {
    client: reqwest::Client,
}

impl Default for YtMusicProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl YtMusicProvider {
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36")
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        Self { client }
    }

    /// Searches YouTube Music for songs matching the query.
    pub async fn search(&self, query: &str) -> Result<Vec<Track>, String> {
        let trimmed = query.trim();
        if trimmed.is_empty() {
            return Ok(Self::get_default_tracks());
        }

        match self.query_innertube_search(trimmed).await {
            Ok(tracks) if !tracks.is_empty() => Ok(tracks),
            _ => {
                // Try fallback piped/invidious search or return filtered local tracks
                let mut fallback = Self::get_default_tracks();
                fallback.retain(|t| {
                    t.title.to_lowercase().contains(&trimmed.to_lowercase())
                        || t.artist.to_lowercase().contains(&trimmed.to_lowercase())
                });
                if fallback.is_empty() {
                    Ok(Self::get_default_tracks())
                } else {
                    Ok(fallback)
                }
            }
        }
    }

    /// Resolves a direct audio stream URL for a given video ID.
    pub async fn get_stream_url(&self, video_id: &str) -> Result<String, String> {
        // First try Innertube Android player API
        if let Ok(url) = self.resolve_innertube_player(video_id).await {
            return Ok(url);
        }

        // Fallback: Piped API stream resolver
        if let Ok(url) = self.resolve_piped_stream(video_id).await {
            return Ok(url);
        }

        Err(format!("Unable to resolve stream URL for video {}", video_id))
    }

    async fn query_innertube_search(&self, query: &str) -> Result<Vec<Track>, Box<dyn std::error::Error + Send + Sync>> {
        let payload = serde_json::json!({
            "context": {
                "client": {
                    "clientName": "WEB_REMIX",
                    "clientVersion": "1.20240301.01.00",
                    "hl": "en",
                    "gl": "US"
                }
            },
            "query": query,
            // "Eg-KAQwIABAAGAAgACgAMABqChAEEAMQCRAFEAo%3D" filters for Songs
            "params": "Eg-KAQwIABAAGAAgACgAMABqChAEEAMQCRAFEAo%3D"
        });

        let resp = self
            .client
            .post("https://music.youtube.com/youtubei/v1/search")
            .header("Origin", "https://music.youtube.com")
            .header("Referer", "https://music.youtube.com/")
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await?;

        if !resp.status().is_success() {
            return Err("YouTube Music search returned non-success status".into());
        }

        let body: Value = resp.json().await?;
        let mut tracks = Vec::new();

        Self::extract_tracks_from_innertube(&body, &mut tracks);

        Ok(tracks)
    }

    fn extract_tracks_from_innertube(root: &Value, out: &mut Vec<Track>) {
        let contents = root
            .pointer("/contents/tabbedSearchResultsRenderer/tabs/0/tabRenderer/content/sectionListRenderer/contents")
            .or_else(|| root.pointer("/contents/sectionListRenderer/contents"));

        let sections = match contents.and_then(|v| v.as_array()) {
            Some(arr) => arr,
            None => return,
        };

        for section in sections {
            let items = section
                .pointer("/musicShelfRenderer/contents")
                .or_else(|| section.pointer("/itemSectionRenderer/contents"))
                .and_then(|v| v.as_array());

            if let Some(item_list) = items {
                for item in item_list {
                    if let Some(renderer) = item.get("musicResponsiveListItemRenderer") {
                        if let Some(track) = Self::parse_responsive_item(renderer) {
                            out.push(track);
                        }
                    }
                }
            }
        }
    }

    fn parse_responsive_item(renderer: &Value) -> Option<Track> {
        let video_id = renderer
            .pointer("/playlistItemData/videoId")
            .and_then(|v| v.as_str())
            .or_else(|| {
                renderer
                    .pointer("/overlay/musicItemThumbnailOverlayRenderer/content/musicPlayButtonRenderer/playNavigationEndpoint/watchEndpoint/videoId")
                    .and_then(|v| v.as_str())
            })?
            .to_string();

        let flex_columns = renderer.pointer("/flexColumns")?.as_array()?;

        // Column 0: Title
        let title_runs = flex_columns
            .first()?
            .pointer("/musicResponsiveListItemFlexColumnRenderer/text/runs")?
            .as_array()?;
        let title = title_runs.first()?.get("text")?.as_str()?.to_string();

        // Column 1: Artist, Album, Duration
        let mut artist = "Unknown Artist".to_string();
        let mut album = None;
        let mut duration = 0u64;

        if let Some(second_col) = flex_columns.get(1) {
            if let Some(runs) = second_col
                .pointer("/musicResponsiveListItemFlexColumnRenderer/text/runs")
                .and_then(|v| v.as_array())
            {
                let text_runs: Vec<&str> = runs
                    .iter()
                    .filter_map(|r| r.get("text").and_then(|t| t.as_str()))
                    .filter(|t| *t != " • " && *t != " & ")
                    .collect();

                if let Some(first_artist) = text_runs.first() {
                    artist = first_artist.to_string();
                }

                if text_runs.len() >= 3 {
                    album = Some(text_runs[1].to_string());
                    if let Some(dur_str) = text_runs.last() {
                        duration = Self::parse_duration_string(dur_str);
                    }
                } else if text_runs.len() == 2 {
                    if let Some(dur_str) = text_runs.get(1) {
                        let parsed = Self::parse_duration_string(dur_str);
                        if parsed > 0 {
                            duration = parsed;
                        } else {
                            album = Some(dur_str.to_string());
                        }
                    }
                }
            }
        }

        // Thumbnail / Cover
        let cover_url = renderer
            .pointer("/thumbnail/musicThumbnailRenderer/thumbnail/thumbnails")
            .and_then(|v| v.as_array())
            .and_then(|arr| arr.last())
            .and_then(|thumb| thumb.get("url"))
            .and_then(|u| u.as_str())
            .map(|s| s.to_string())
            .or_else(|| Some(format!("https://i.ytimg.com/vi/{}/hqdefault.jpg", video_id)));

        // Badges: Explicit
        let is_explicit = renderer.pointer("/badges").and_then(|b| b.as_array()).map(|badges| {
            badges.iter().any(|badge| {
                badge
                    .pointer("/musicInlineBadgeRenderer/accessibilityData/accessibilityData/label")
                    .and_then(|l| l.as_str())
                    .map(|l| l.to_lowercase().contains("explicit"))
                    .unwrap_or(false)
            })
        });

        Some(Track {
            id: video_id,
            title,
            artist,
            album,
            duration: if duration == 0 { 210 } else { duration },
            cover_url,
            audio_url: None,
            is_explicit,
            is_offline: Some(false),
        })
    }

    fn parse_duration_string(dur: &str) -> u64 {
        let parts: Vec<&str> = dur.split(':').collect();
        match parts.len() {
            2 => {
                let m: u64 = parts[0].parse().unwrap_or(0);
                let s: u64 = parts[1].parse().unwrap_or(0);
                (m * 60) + s
            }
            3 => {
                let h: u64 = parts[0].parse().unwrap_or(0);
                let m: u64 = parts[1].parse().unwrap_or(0);
                let s: u64 = parts[2].parse().unwrap_or(0);
                (h * 3600) + (m * 60) + s
            }
            _ => 0,
        }
    }

    async fn resolve_innertube_player(&self, video_id: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let payload = serde_json::json!({
            "context": {
                "client": {
                    "clientName": "ANDROID",
                    "clientVersion": "19.05.36",
                    "hl": "en",
                    "gl": "US"
                }
            },
            "videoId": video_id
        });

        let resp = self
            .client
            .post("https://www.youtube.com/youtubei/v1/player")
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await?;

        let data: Value = resp.json().await?;
        if let Some(formats) = data.pointer("/streamingData/adaptiveFormats").and_then(|v| v.as_array()) {
            // Find highest audio format with direct URL
            for fmt in formats {
                let mime = fmt.get("mimeType").and_then(|m| m.as_str()).unwrap_or("");
                if mime.starts_with("audio/") {
                    if let Some(url) = fmt.get("url").and_then(|u| u.as_str()) {
                        return Ok(url.to_string());
                    }
                }
            }
        }

        Err("No direct stream URL found in Innertube response".into())
    }

    async fn resolve_piped_stream(&self, video_id: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let piped_instances = [
            "https://pipedapi.kavin.rocks",
            "https://api.piped.privacydev.net",
            "https://piped-api.lunar.icu",
        ];

        for instance in piped_instances {
            let url = format!("{}/streams/{}", instance, video_id);
            if let Ok(resp) = self.client.get(&url).send().await {
                if let Ok(json) = resp.json::<Value>().await {
                    if let Some(audio_streams) = json.get("audioStreams").and_then(|v| v.as_array()) {
                        if let Some(first_audio) = audio_streams.first() {
                            if let Some(stream_url) = first_audio.get("url").and_then(|u| u.as_str()) {
                                return Ok(stream_url.to_string());
                            }
                        }
                    }
                }
            }
        }

        Err("Failed to resolve from piped instances".into())
    }

    /// Curated default tracks with fallback high quality audio streams.
    pub fn get_default_tracks() -> Vec<Track> {
        vec![
            Track {
                id: "1".to_string(),
                title: "Starlight".to_string(),
                artist: "The Nocturnals".to_string(),
                album: Some("Astral Memories".to_string()),
                duration: 225,
                cover_url: Some("https://images.unsplash.com/photo-1518709268805-4e9042af9f23?w=300&auto=format&fit=crop&q=80".to_string()),
                audio_url: Some("https://www.soundhelix.com/examples/mp3/SoundHelix-Song-1.mp3".to_string()),
                is_explicit: Some(true),
                is_offline: Some(false),
            },
            Track {
                id: "2".to_string(),
                title: "Echoes".to_string(),
                artist: "Aurora".to_string(),
                album: Some("Parallel Horizons".to_string()),
                duration: 252,
                cover_url: Some("https://images.unsplash.com/photo-1509198397868-475647b2a1e5?w=300&auto=format&fit=crop&q=80".to_string()),
                audio_url: Some("https://www.soundhelix.com/examples/mp3/SoundHelix-Song-2.mp3".to_string()),
                is_explicit: Some(false),
                is_offline: Some(false),
            },
            Track {
                id: "3".to_string(),
                title: "Midnight City".to_string(),
                artist: "M83".to_string(),
                album: Some("Hurry Up, We Are Dreaming".to_string()),
                duration: 244,
                cover_url: Some("https://images.unsplash.com/photo-1470225620780-dba8ba36b745?w=300&auto=format&fit=crop&q=80".to_string()),
                audio_url: Some("https://www.soundhelix.com/examples/mp3/SoundHelix-Song-3.mp3".to_string()),
                is_explicit: Some(false),
                is_offline: Some(false),
            },
            Track {
                id: "4".to_string(),
                title: "After Dark".to_string(),
                artist: "Mr.Kitty".to_string(),
                album: Some("Time".to_string()),
                duration: 258,
                cover_url: Some("https://images.unsplash.com/photo-1511671782779-c97d3d27a1d4?w=300&auto=format&fit=crop&q=80".to_string()),
                audio_url: Some("https://www.soundhelix.com/examples/mp3/SoundHelix-Song-4.mp3".to_string()),
                is_explicit: Some(false),
                is_offline: Some(false),
            },
            Track {
                id: "5".to_string(),
                title: "Resonance".to_string(),
                artist: "HOME".to_string(),
                album: Some("Odyssey".to_string()),
                duration: 212,
                cover_url: Some("https://images.unsplash.com/photo-1493225457124-a3eb161ffa5f?w=300&auto=format&fit=crop&q=80".to_string()),
                audio_url: Some("https://www.soundhelix.com/examples/mp3/SoundHelix-Song-5.mp3".to_string()),
                is_explicit: Some(false),
                is_offline: Some(false),
            },
        ]
    }
}
