use crate::models::Track;
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink};
use std::io::Cursor;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;

pub enum AudioCommand {
    PlayBytes {
        bytes: Vec<u8>,
        track_id: String,
    },
    Pause,
    Resume,
    SetVolume(f32),
    Stop,
}

pub struct PlayerManager {
    cmd_sender: Sender<AudioCommand>,
    current_track: Arc<Mutex<Option<Track>>>,
    is_playing: Arc<AtomicBool>,
    volume: Arc<Mutex<f32>>,
    http_client: reqwest::Client,
}

impl PlayerManager {
    pub fn new() -> Self {
        let (tx, rx) = channel::<AudioCommand>();
        let current_track = Arc::new(Mutex::new(None));
        let is_playing = Arc::new(AtomicBool::new(false));
        let volume = Arc::new(Mutex::new(0.8f32));

        let current_track_clone = Arc::clone(&current_track);
        let is_playing_clone = Arc::clone(&is_playing);

        // Dedicated audio thread to preserve thread affinity for WASAPI/ALSA/CoreAudio
        thread::Builder::new()
            .name("time-audio-engine".to_string())
            .spawn(move || {
                run_audio_loop(rx, current_track_clone, is_playing_clone);
            })
            .expect("Failed to spawn audio worker thread");

        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        Self {
            cmd_sender: tx,
            current_track,
            is_playing,
            volume,
            http_client,
        }
    }

    /// Plays a track by fetching its audio content and sending to decoder.
    pub async fn play(&self, track: Track, audio_url: &str) -> Result<(), String> {
        // Fetch audio bytes
        let bytes = if audio_url.starts_with("http://") || audio_url.starts_with("https://") {
            let resp = self
                .http_client
                .get(audio_url)
                .send()
                .await
                .map_err(|e| format!("Network error fetching audio: {}", e))?;

            if !resp.status().is_success() {
                return Err(format!("HTTP error {} fetching audio", resp.status()));
            }

            resp.bytes()
                .await
                .map_err(|e| format!("Failed to read audio stream bytes: {}", e))?
                .to_vec()
        } else {
            // Local file path
            tokio::fs::read(audio_url)
                .await
                .map_err(|e| format!("Failed to read local file {}: {}", audio_url, e))?
        };

        if bytes.is_empty() {
            return Err("Audio stream buffer is empty".to_string());
        }

        {
            let mut cur = self.current_track.lock().unwrap();
            *cur = Some(track.clone());
        }

        let vol = *self.volume.lock().unwrap();

        self.cmd_sender
            .send(AudioCommand::PlayBytes {
                bytes,
                track_id: track.id,
            })
            .map_err(|e| format!("Failed to send audio command: {}", e))?;

        // Re-apply volume
        let _ = self.cmd_sender.send(AudioCommand::SetVolume(vol));

        self.is_playing.store(true, Ordering::SeqCst);
        Ok(())
    }

    pub fn pause(&self) -> Result<(), String> {
        self.cmd_sender
            .send(AudioCommand::Pause)
            .map_err(|e| format!("Audio engine error: {}", e))?;
        self.is_playing.store(false, Ordering::SeqCst);
        Ok(())
    }

    pub fn resume(&self) -> Result<(), String> {
        self.cmd_sender
            .send(AudioCommand::Resume)
            .map_err(|e| format!("Audio engine error: {}", e))?;
        self.is_playing.store(true, Ordering::SeqCst);
        Ok(())
    }

    pub fn set_volume(&self, volume: f32) -> Result<(), String> {
        let clamped = volume.clamp(0.0, 1.0);
        {
            let mut v = self.volume.lock().unwrap();
            *v = clamped;
        }

        self.cmd_sender
            .send(AudioCommand::SetVolume(clamped))
            .map_err(|e| format!("Audio engine error: {}", e))
    }

    pub fn stop(&self) -> Result<(), String> {
        self.cmd_sender
            .send(AudioCommand::Stop)
            .map_err(|e| format!("Audio engine error: {}", e))?;
        self.is_playing.store(false, Ordering::SeqCst);
        Ok(())
    }

    pub fn get_current_track(&self) -> Option<Track> {
        self.current_track.lock().unwrap().clone()
    }

    pub fn is_playing(&self) -> bool {
        self.is_playing.load(Ordering::SeqCst)
    }
}

fn run_audio_loop(
    rx: Receiver<AudioCommand>,
    _current_track: Arc<Mutex<Option<Track>>>,
    is_playing: Arc<AtomicBool>,
) {
    let stream_result = OutputStream::try_default();
    let (_stream, stream_handle): (Option<OutputStream>, Option<OutputStreamHandle>) = match stream_result {
        Ok((s, h)) => (Some(s), Some(h)),
        Err(err) => {
            eprintln!("[Time Audio Engine] Warning: Failed to open default audio output stream: {}", err);
            (None, None)
        }
    };

    let mut current_sink: Option<Sink> = stream_handle.as_ref().and_then(|h| Sink::try_new(h).ok());
    let mut current_volume: f32 = 0.8;

    while let Ok(cmd) = rx.recv() {
        match cmd {
            AudioCommand::PlayBytes { bytes, .. } => {
                // Terminate existing playback sink
                if let Some(sink) = current_sink.take() {
                    sink.stop();
                }

                if let Some(h) = &stream_handle {
                    match Sink::try_new(h) {
                        Ok(new_sink) => {
                            let cursor = Cursor::new(bytes);
                            match Decoder::new(cursor) {
                                Ok(source) => {
                                    new_sink.set_volume(current_volume);
                                    new_sink.append(source);
                                    new_sink.play();
                                    current_sink = Some(new_sink);
                                    is_playing.store(true, Ordering::SeqCst);
                                }
                                Err(err) => {
                                    eprintln!("[Time Audio Engine] Audio decoding failed: {}", err);
                                    is_playing.store(false, Ordering::SeqCst);
                                }
                            }
                        }
                        Err(err) => {
                            eprintln!("[Time Audio Engine] Failed to create new Sink: {}", err);
                            is_playing.store(false, Ordering::SeqCst);
                        }
                    }
                }
            }
            AudioCommand::Pause => {
                if let Some(sink) = &current_sink {
                    sink.pause();
                }
                is_playing.store(false, Ordering::SeqCst);
            }
            AudioCommand::Resume => {
                if let Some(sink) = &current_sink {
                    sink.play();
                }
                is_playing.store(true, Ordering::SeqCst);
            }
            AudioCommand::SetVolume(vol) => {
                current_volume = vol.clamp(0.0, 1.0);
                if let Some(sink) = &current_sink {
                    sink.set_volume(current_volume);
                }
            }
            AudioCommand::Stop => {
                if let Some(sink) = current_sink.take() {
                    sink.stop();
                }
                is_playing.store(false, Ordering::SeqCst);
            }
        }
    }
}
