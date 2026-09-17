pub mod audio;
pub mod discord;
pub mod models;
pub mod providers;
pub mod tray;

use audio::AudioState;
use discord::DiscordManager;
use models::{LyricLine, Track};
use providers::ProviderManager;
use std::sync::Arc;
use tauri::Manager;

#[tauri::command]
async fn search_tracks(
    query: String,
    providers: tauri::State<'_, Arc<ProviderManager>>,
) -> Result<Vec<Track>, String> {
    providers.search_tracks(&query).await
}

#[tauri::command]
async fn get_lyrics(
    title: String,
    artist: String,
    album: Option<String>,
    duration: Option<u64>,
    providers: tauri::State<'_, Arc<ProviderManager>>,
) -> Result<Vec<LyricLine>, String> {
    providers
        .get_lyrics(&title, &artist, album.as_deref(), duration)
        .await
}

#[tauri::command]
async fn play_track(
    track: Track,
    audio_state: tauri::State<'_, AudioState>,
    providers: tauri::State<'_, Arc<ProviderManager>>,
    discord: tauri::State<'_, Arc<DiscordManager>>,
) -> Result<(), String> {
    let audio_url = providers.resolve_audio_url(&track).await?;

    audio_state.0.play(track.clone(), &audio_url).await?;

    let _ = discord.set_presence(
        &track.title,
        &track.artist,
        track.album.as_deref(),
        Some(track.duration),
        true,
    );

    Ok(())
}

#[tauri::command]
async fn pause_track(
    audio_state: tauri::State<'_, AudioState>,
    discord: tauri::State<'_, Arc<DiscordManager>>,
) -> Result<(), String> {
    audio_state.0.pause()?;

    if let Some(track) = audio_state.0.get_current_track() {
        let _ = discord.set_presence(
            &track.title,
            &track.artist,
            track.album.as_deref(),
            Some(track.duration),
            false,
        );
    }

    Ok(())
}

#[tauri::command]
async fn resume_track(
    audio_state: tauri::State<'_, AudioState>,
    discord: tauri::State<'_, Arc<DiscordManager>>,
) -> Result<(), String> {
    audio_state.0.resume()?;

    if let Some(track) = audio_state.0.get_current_track() {
        let _ = discord.set_presence(
            &track.title,
            &track.artist,
            track.album.as_deref(),
            Some(track.duration),
            true,
        );
    }

    Ok(())
}

#[tauri::command]
async fn set_volume(
    volume: f32,
    audio_state: tauri::State<'_, AudioState>,
) -> Result<(), String> {
    audio_state.0.set_volume(volume)
}

#[tauri::command]
async fn set_discord_presence(
    title: String,
    artist: String,
    album: Option<String>,
    duration: Option<u64>,
    is_playing: Option<bool>,
    discord: tauri::State<'_, Arc<DiscordManager>>,
) -> Result<(), String> {
    discord.set_presence(
        &title,
        &artist,
        album.as_deref(),
        duration,
        is_playing.unwrap_or(true),
    )
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(AudioState::new())
        .manage(Arc::new(ProviderManager::new()))
        .manage(Arc::new(DiscordManager::new()))
        .setup(|app| {
            // Setup system tray integration
            crate::tray::setup_tray(app.handle())?;

            // Close-to-tray handling: prevent window destruction on close request and hide instead
            if let Some(window) = app.get_webview_window("main") {
                let window_clone = window.clone();
                window.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        let _ = window_clone.hide();
                    }
                });
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            search_tracks,
            get_lyrics,
            play_track,
            pause_track,
            resume_track,
            set_volume,
            set_discord_presence
        ])
        .run(tauri::generate_context!())
        .expect("error while running Time application");
}
