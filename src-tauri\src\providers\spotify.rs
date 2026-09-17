use crate::models::Track;
use serde::Deserialize;
use serde_json::Value;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

#[derive(Clone)]
struct CachedToken {
    token: String,
    expires_at: Instant,
}

pub struct SpotifyProvider {
    client: reqwest::Client,
    cached_token: Arc<RwLock<Option<CachedToken>>>,
}

impl Default for SpotifyProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl SpotifyProvider {
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(8))
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36")
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        Self {
            client,
            cached_token: Arc::new(RwLock::new(None)),
        }
    }

    /// Obtains an anonymous web client access token from Spotify.
    async fn get_access_token(&self) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        {
            let read = self.cached_token.read().await;
            if let Some(cached) = &*read {
                if Instant::now() < cached.expires_at {
                    return Ok(cached.token.clone());
                }
            }
        }

        let resp = self
            .client
            .get("https://open.spotify.com/get_access_token?reason=transport&productType=web_player")
            .header("Referer", "https://open.spotify.com/")
            .send()
            .await?;

        if !resp.status().is_success() {
            return Err("Failed to obtain Spotify access token".into());
        }

        #[derive(Deserialize)]
        struct TokenResp {
            #[serde(rename = "accessToken")]
            access_token: String,
            #[serde(rename = "accessTokenExpirationTimestampMs")]
            expiration_ms: Option<u64>,
        }

        let body: TokenResp = resp.json().await?;
        let ttl_secs = match body.expiration_ms {
            Some(exp) => {
                let now_ms = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64;
                if exp > now_ms {
                    (exp - now_ms) / 1000
                } else {
                    1800
                }
            }
            None => 1800,
        };

        let token_val = body.access_token.clone();
        let mut write = self.cached_token.write().await;
        *write = Some(CachedToken {
            token: token_val.clone(),
            expires_at: Instant::now() + Duration::from_secs(ttl_secs.saturating_sub(60)),
        });

        Ok(token_val)
    }

    /// Searches Spotify tracks and returns them as `Vec<Track>`.
    pub async fn search(&self, query: &str) -> Result<Vec<Track>, String> {
        let trimmed = query.trim();
        if trimmed.is_empty() {
            return Ok(Vec::new());
        }

        let token = self.get_access_token().await.map_err(|e| e.to_string())?;

        let resp = self
            .client
            .get("https://api.spotify.com/v1/search")
            .bearer_auth(token)
            .query(&[("type", "track"), ("q", trimmed), ("limit", "15")])
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !resp.status().is_success() {
            return Err(format!("Spotify search failed with status {}", resp.status()));
        }

        let body: Value = resp.json().await.map_err(|e| e.to_string())?;
        let mut tracks = Vec::new();

        if let Some(items) = body.pointer("/tracks/items").and_then(|v| v.as_array()) {
            for item in items {
                if let Some(track) = Self::parse_spotify_track(item) {
                    tracks.push(track);
                }
            }
        }

        Ok(tracks)
    }

    fn parse_spotify_track(item: &Value) -> Option<Track> {
        let id = item.get("id")?.as_str()?.to_string();
        let title = item.get("name")?.as_str()?.to_string();

        let artists_list = item.get("artists")?.as_array()?;
        let artist_names: Vec<&str> = artists_list
            .iter()
            .filter_map(|a| a.get("name").and_then(|n| n.as_str()))
            .collect();
        let artist = if artist_names.is_empty() {
            "Unknown Artist".to_string()
        } else {
            artist_names.join(", ")
        };

        let album_val = item.get("album");
        let album = album_val.and_then(|a| a.get("name")).and_then(|n| n.as_str()).map(|s| s.to_string());

        let cover_url = album_val
            .and_then(|a| a.get("images"))
            .and_then(|imgs| imgs.as_array())
            .and_then(|arr| arr.first())
            .and_then(|img| img.get("url"))
            .and_then(|u| u.as_str())
            .map(|s| s.to_string());

        let duration_ms = item.get("duration_ms").and_then(|d| d.as_u64()).unwrap_or(0);
        let duration = duration_ms / 1000;

        let is_explicit = item.get("explicit").and_then(|e| e.as_bool());

        Some(Track {
            id,
            title,
            artist,
            album,
            duration,
            cover_url,
            audio_url: None,
            is_explicit,
            is_offline: Some(false),
        })
    }
}
