use discord_rich_presence::{activity, DiscordIpc, DiscordIpcClient};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

const DISCORD_APP_ID: &str = "1219273921029374002";

pub struct DiscordManager {
    client: Mutex<Option<DiscordIpcClient>>,
}

impl Default for DiscordManager {
    fn default() -> Self {
        Self::new()
    }
}

impl DiscordManager {
    pub fn new() -> Self {
        Self {
            client: Mutex::new(None),
        }
    }

    /// Connects to Discord IPC if not currently connected.
    fn ensure_connected(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut client_guard = self.client.lock().unwrap();
        if client_guard.is_some() {
            return Ok(());
        }

        match DiscordIpcClient::new(DISCORD_APP_ID) {
            Ok(mut new_client) => {
                if let Err(err) = new_client.connect() {
                    return Err(format!("Discord IPC connect failed: {}", err).into());
                }
                *client_guard = Some(new_client);
                Ok(())
            }
            Err(err) => Err(format!("Failed to initialize Discord client: {}", err).into()),
        }
    }

    /// Sets or updates the Discord Rich Presence activity.
    pub fn set_presence(
        &self,
        title: &str,
        artist: &str,
        album: Option<&str>,
        _duration: Option<u64>,
        is_playing: bool,
    ) -> Result<(), String> {
        let _ = self.ensure_connected();

        let mut client_guard = self.client.lock().unwrap();
        let client = match client_guard.as_mut() {
            Some(c) => c,
            None => {
                // Discord is not currently running; silently succeed without crashing player
                return Ok(());
            }
        };

        let state_str = if let Some(alb) = album {
            if !alb.is_empty() && alb != title {
                format!("{} • {}", artist, alb)
            } else {
                artist.to_string()
            }
        } else {
            artist.to_string()
        };

        let mut act = activity::Activity::new()
            .details(title)
            .state(&state_str)
            .assets(
                activity::Assets::new()
                    .large_image("icon")
                    .large_text("Time - Material You Music")
                    .small_image(if is_playing { "play" } else { "pause" })
                    .small_text(if is_playing { "Playing" } else { "Paused" }),
            );

        if is_playing {
            let start_time = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs() as i64;

            act = act.timestamps(activity::Timestamps::new().start(start_time));
        }

        if let Err(err) = client.set_activity(act) {
            // Disconnect on pipe error so reconnect can be attempted next time
            eprintln!("[Discord RPC] Failed to set activity: {}. Resetting client.", err);
            *client_guard = None;
        }

        Ok(())
    }

    /// Clears any active Discord presence.
    pub fn clear_presence(&self) -> Result<(), String> {
        let mut client_guard = self.client.lock().unwrap();
        if let Some(client) = client_guard.as_mut() {
            let _ = client.clear_activity();
        }
        Ok(())
    }
}
