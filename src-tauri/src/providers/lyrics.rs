use regex::Regex;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;
use std::time::Duration;

static TIMESTAMP_REGEX: OnceLock<Regex> = OnceLock::new();

fn timestamp_regex() -> &'static Regex {
    TIMESTAMP_REGEX.get_or_init(|| {
        Regex::new(r"\[(\d{1,2}):(\d{2})(?:\.(\d{1,3}))?\]").expect("invalid timestamp regex")
    })
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LyricLine {
    pub time: f64,
    pub text: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LrclibResponse {
    pub plain_lyrics: Option<String>,
    pub synced_lyrics: Option<String>,
    pub instrumental: Option<bool>,
}

/// Parses raw LRC synced lyrics text containing `[mm:ss.xx]` timestamps
/// into an ordered sequence of `LyricLine` entries.
pub fn parse_lrc(lrc_text: &str) -> Vec<LyricLine> {
    let mut lines = Vec::new();

    for raw_line in lrc_text.lines() {
        let trimmed = raw_line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let mut timestamps = Vec::new();
        let mut last_end = 0;

        for caps in timestamp_regex().captures_iter(trimmed) {
            if let Some(m) = caps.get(0) {
                last_end = last_end.max(m.end());
            }
            let minutes: f64 = caps[1].parse().unwrap_or(0.0);
            let seconds: f64 = caps[2].parse().unwrap_or(0.0);
            let frac = caps.get(3).map_or(0.0, |m| {
                format!("0.{}", m.as_str()).parse::<f64>().unwrap_or(0.0)
            });
            let time = ((minutes * 60.0 + seconds + frac) * 1000.0).round() / 1000.0;
            timestamps.push(time);
        }

        if timestamps.is_empty() {
            continue;
        }

        let lyric_text = trimmed[last_end..].trim().to_string();

        for time in timestamps {
            lines.push(LyricLine {
                time,
                text: lyric_text.clone(),
            });
        }
    }

    lines.sort_by(|a, b| a.time.partial_cmp(&b.time).unwrap_or(std::cmp::Ordering::Equal));
    lines
}

fn parse_plain_lyrics(plain: &str) -> Vec<LyricLine> {
    plain
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .enumerate()
        .map(|(idx, text)| LyricLine {
            time: idx as f64 * 3.5,
            text,
        })
        .collect()
}

/// Queries https://lrclib.net/api/get with track_name, artist_name, and optional duration.
/// Parses and returns synchronized lyrics.
pub async fn get_lyrics(
    title: &str,
    artist: &str,
    duration: Option<f64>,
) -> Result<Vec<LyricLine>, String> {
    let title_clean = title.trim();
    let artist_clean = artist.trim();

    if title_clean.is_empty() {
        return Ok(Vec::new());
    }

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .user_agent("TimeMusicPlayer/0.1.0 (https://github.com/time-player/time)")
        .build()
        .unwrap_or_else(|_| reqwest::Client::new());

    // 1. Primary query to /api/get with track_name, artist_name, and optional duration
    let mut query: Vec<(&str, String)> = vec![
        ("track_name", title_clean.to_string()),
        ("artist_name", artist_clean.to_string()),
    ];

    if let Some(dur) = duration {
        if dur > 0.0 {
            query.push(("duration", format!("{:.0}", dur)));
        }
    }

    let resp_res = client
        .get("https://lrclib.net/api/get")
        .query(&query)
        .send()
        .await;

    if let Ok(resp) = resp_res {
        if resp.status().is_success() {
            if let Ok(data) = resp.json::<LrclibResponse>().await {
                if let Some(synced) = data.synced_lyrics {
                    let parsed = parse_lrc(&synced);
                    if !parsed.is_empty() {
                        return Ok(parsed);
                    }
                }
                if data.instrumental == Some(true) {
                    return Ok(vec![LyricLine {
                        time: 0.0,
                        text: "♪ (Instrumental) ♪".to_string(),
                    }]);
                }
                if let Some(plain) = data.plain_lyrics {
                    let parsed = parse_plain_lyrics(&plain);
                    if !parsed.is_empty() {
                        return Ok(parsed);
                    }
                }
            }
        }
    }

    // 2. Fallback: retry /api/get without duration in case local duration slightly diverges
    if duration.is_some() {
        let fallback_query = [
            ("track_name", title_clean),
            ("artist_name", artist_clean),
        ];

        if let Ok(resp) = client
            .get("https://lrclib.net/api/get")
            .query(&fallback_query)
            .send()
            .await
        {
            if resp.status().is_success() {
                if let Ok(data) = resp.json::<LrclibResponse>().await {
                    if let Some(synced) = data.synced_lyrics {
                        let parsed = parse_lrc(&synced);
                        if !parsed.is_empty() {
                            return Ok(parsed);
                        }
                    }
                    if data.instrumental == Some(true) {
                        return Ok(vec![LyricLine {
                            time: 0.0,
                            text: "♪ (Instrumental) ♪".to_string(),
                        }]);
                    }
                    if let Some(plain) = data.plain_lyrics {
                        let parsed = parse_plain_lyrics(&plain);
                        if !parsed.is_empty() {
                            return Ok(parsed);
                        }
                    }
                }
            }
        }
    }

    // 3. Fallback: fuzzy search endpoint
    let search_q = format!("{} {}", title_clean, artist_clean);
    if let Ok(resp) = client
        .get("https://lrclib.net/api/search")
        .query(&[("q", &search_q)])
        .send()
        .await
    {
        if resp.status() == StatusCode::OK {
            if let Ok(results) = resp.json::<Vec<LrclibResponse>>().await {
                for item in results {
                    if let Some(synced) = item.synced_lyrics {
                        let parsed = parse_lrc(&synced);
                        if !parsed.is_empty() {
                            return Ok(parsed);
                        }
                    }
                }
            }
        }
    }

    Ok(Vec::new())
}

pub struct LyricsProvider {
    client: reqwest::Client,
}

impl Default for LyricsProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl LyricsProvider {
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .user_agent("TimeMusicPlayer/0.1.0 (https://github.com/time-player/time)")
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        Self { client }
    }

    pub async fn get_lyrics(
        &self,
        title: &str,
        artist: &str,
        duration: Option<f64>,
    ) -> Result<Vec<LyricLine>, String> {
        get_lyrics(title, artist, duration).await
    }

    pub async fn fetch_lyrics(
        &self,
        title: &str,
        artist: &str,
        _album: Option<&str>,
        duration: Option<u64>,
    ) -> Result<Vec<LyricLine>, String> {
        get_lyrics(title, artist, duration.map(|d| d as f64)).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_lrc_standard() {
        let lrc = "\
[00:14.20]We danced into the night
[00:24.50]Beneath the glowing light
[00:38.00]Where dreams take flight";

        let parsed = parse_lrc(lrc);
        assert_eq!(parsed.len(), 3);
        assert!((parsed[0].time - 14.20).abs() < 0.001);
        assert_eq!(parsed[0].text, "We danced into the night");
        assert!((parsed[1].time - 24.50).abs() < 0.001);
        assert_eq!(parsed[1].text, "Beneath the glowing light");
        assert!((parsed[2].time - 38.00).abs() < 0.001);
        assert_eq!(parsed[2].text, "Where dreams take flight");
    }

    #[test]
    fn test_parse_lrc_milliseconds_and_empty_lines() {
        let lrc = "\
[ti:Sample Song]
[ar:Sample Artist]
[01:05.340] In the darkness
[01:10.50]
[01:15.05]Coming home";

        let parsed = parse_lrc(lrc);
        assert_eq!(parsed.len(), 3);
        assert!((parsed[0].time - 65.34).abs() < 0.001);
        assert_eq!(parsed[0].text, "In the darkness");
        assert!((parsed[1].time - 70.50).abs() < 0.001);
        assert_eq!(parsed[1].text, "");
        assert!((parsed[2].time - 75.05).abs() < 0.001);
        assert_eq!(parsed[2].text, "Coming home");
    }

    #[test]
    fn test_parse_lrc_multiple_timestamps() {
        let lrc = "[00:10.00][00:20.00] Echo echo";
        let parsed = parse_lrc(lrc);
        assert_eq!(parsed.len(), 2);
        assert!((parsed[0].time - 10.0).abs() < 0.001);
        assert_eq!(parsed[0].text, "Echo echo");
        assert!((parsed[1].time - 20.0).abs() < 0.001);
        assert_eq!(parsed[1].text, "Echo echo");
    }
}
