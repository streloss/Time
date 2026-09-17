import React from 'react';
import { Track } from '../types';

interface PlayerBarProps {
  track: Track | null;
  isPlaying: boolean;
  currentTime: number;
  duration: number;
  volume: number;
  isMuted: boolean;
  isShuffle: boolean;
  isRepeat: boolean;
  isLyricsOpen: boolean;
  onTogglePlay: () => void;
  onPrevious: () => void;
  onNext: () => void;
  onSeek: (seconds: number) => void;
  onVolumeChange: (volume: number) => void;
  onToggleMute: () => void;
  onToggleShuffle: () => void;
  onToggleRepeat: () => void;
  onToggleLyrics: () => void;
  onToggleLike?: (track: Track) => void;
}

export const PlayerBar: React.FC<PlayerBarProps> = ({
  track,
  isPlaying,
  currentTime,
  duration,
  volume,
  isMuted,
  isShuffle,
  isRepeat,
  isLyricsOpen,
  onTogglePlay,
  onPrevious,
  onNext,
  onSeek,
  onVolumeChange,
  onToggleMute,
  onToggleShuffle,
  onToggleRepeat,
  onToggleLyrics,
  onToggleLike,
}) => {
  const formatTime = (seconds: number) => {
    if (isNaN(seconds) || seconds < 0) return '0:00';
    const mins = Math.floor(seconds / 60);
    const secs = Math.floor(seconds % 60);
    return `${mins}:${secs < 10 ? '0' : ''}${secs}`;
  };

  const progressPct = duration > 0 ? Math.min(100, Math.max(0, (currentTime / duration) * 100)) : 0;
  const currentVol = isMuted ? 0 : volume;
  const volumePct = Math.min(100, Math.max(0, currentVol * 100));

  return (
    <footer className="w-full px-4 pb-3 pt-1 flex-shrink-0 select-none z-40 bg-transparent">
      {/* M3 Floating Player Island */}
      <div className="w-full max-w-7xl mx-auto bg-[#131313]/95 backdrop-blur-xl border border-neutral-800/90 rounded-3xl px-5 py-2.5 flex items-center justify-between shadow-2xl">
        {/* 1. Left: Track Info, Album Art & Quick Like */}
        <div className="flex items-center gap-3.5 w-1/4 min-w-[220px]">
          <div className="w-12 h-12 rounded-2xl bg-neutral-900 flex-shrink-0 overflow-hidden relative border border-neutral-800 shadow-sm">
            {track?.coverUrl ? (
              <img
                src={track.coverUrl}
                alt={track.title}
                className="w-full h-full object-cover"
              />
            ) : (
              <div className="w-full h-full flex items-center justify-center text-neutral-600">
                <span className="material-symbols-rounded text-2xl">music_note</span>
              </div>
            )}
          </div>

          <div className="flex flex-col truncate min-w-0 pr-1">
            <span className="text-sm font-semibold text-white tracking-tight truncate">
              {track ? track.title : 'No track selected'}
            </span>
            <span className="text-xs text-neutral-400 truncate">
              {track ? track.artist : 'Select a song to play'}
            </span>
          </div>

          {track && onToggleLike && (
            <button
              type="button"
              onClick={() => onToggleLike(track)}
              title={track.isLiked ? 'Unlike' : 'Like'}
              aria-label={track.isLiked ? 'Unlike track' : 'Like track'}
              className="p-1.5 rounded-full hover:bg-neutral-800 text-neutral-400 hover:text-white transition-colors flex-shrink-0 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-white"
            >
              <span className={`material-symbols-rounded text-lg ${track.isLiked ? 'filled text-white' : ''}`}>
                favorite
              </span>
            </button>
          )}
        </div>

        {/* 2. Center: Playback Controls & Expressive Timeline Scrubber */}
        <div className="flex flex-col items-center gap-1 flex-1 max-w-xl px-4">
          {/* Buttons */}
          <div className="flex items-center gap-4">
            <button
              type="button"
              onClick={onToggleShuffle}
              title="Shuffle playback"
              aria-label="Toggle shuffle"
              className={`p-1.5 rounded-full transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-white ${
                isShuffle ? 'text-white' : 'text-neutral-500 hover:text-white'
              }`}
            >
              <span className="material-symbols-rounded text-xl">shuffle</span>
            </button>

            <button
              type="button"
              onClick={onPrevious}
              title="Previous track"
              aria-label="Previous track"
              className="p-1.5 rounded-full text-neutral-300 hover:text-white transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-white active:scale-95"
            >
              <span className="material-symbols-rounded text-2xl">skip_previous</span>
            </button>

            {/* Circular Play / Pause Button */}
            <button
              type="button"
              onClick={onTogglePlay}
              title={isPlaying ? 'Pause' : 'Play'}
              aria-label={isPlaying ? 'Pause' : 'Play'}
              className="w-12 h-12 rounded-full bg-white text-black flex items-center justify-center hover:scale-105 active:scale-95 transition-all shadow-lg hover:bg-neutral-200 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-white"
            >
              <span className="material-symbols-rounded filled text-2xl text-black">
                {isPlaying ? 'pause' : 'play_arrow'}
              </span>
            </button>

            <button
              type="button"
              onClick={onNext}
              title="Next track"
              aria-label="Next track"
              className="p-1.5 rounded-full text-neutral-300 hover:text-white transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-white active:scale-95"
            >
              <span className="material-symbols-rounded text-2xl">skip_next</span>
            </button>

            <button
              type="button"
              onClick={onToggleRepeat}
              title="Repeat playback"
              aria-label="Toggle repeat"
              className={`p-1.5 rounded-full transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-white ${
                isRepeat ? 'text-white' : 'text-neutral-500 hover:text-white'
              }`}
            >
              <span className="material-symbols-rounded text-xl">repeat</span>
            </button>
          </div>

          {/* Expressive Timeline Slider */}
          <div className="w-full flex items-center gap-3 text-xs tabular-nums text-neutral-400 font-mono">
            <span className="w-10 text-right">{formatTime(currentTime)}</span>
            <input
              type="range"
              min={0}
              max={duration || 100}
              value={currentTime}
              aria-label="Seek track position"
              aria-valuemin={0}
              aria-valuemax={duration || 100}
              aria-valuenow={currentTime}
              aria-valuetext={formatTime(currentTime)}
              onChange={(e) => onSeek(Number(e.target.value))}
              style={{
                background: `linear-gradient(to right, #FFFFFF ${progressPct}%, #2A2A2A ${progressPct}%)`,
              }}
              className="w-full cursor-pointer"
            />
            <span className="w-10 text-left">{formatTime(duration)}</span>
          </div>
        </div>

        {/* 3. Right: Volume & Lyrics Actions */}
        <div className="flex items-center justify-end gap-2.5 w-1/4 min-w-[220px]">
          {/* Lyrics Toggle Button */}
          <button
            type="button"
            onClick={onToggleLyrics}
            title="Toggle lyrics panel"
            aria-label="Toggle lyrics"
            className={`p-2 rounded-full transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-white ${
              isLyricsOpen ? 'bg-neutral-800 text-white shadow-sm' : 'text-neutral-400 hover:text-white'
            }`}
          >
            <span className="material-symbols-rounded text-xl">lyrics</span>
          </button>

          {/* Volume Mute Toggle */}
          <button
            type="button"
            onClick={onToggleMute}
            title={isMuted ? 'Unmute' : 'Mute'}
            aria-label={isMuted ? 'Unmute audio' : 'Mute audio'}
            className="p-2 rounded-full text-neutral-400 hover:text-white transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-white"
          >
            <span className="material-symbols-rounded text-xl">
              {isMuted || volume === 0
                ? 'volume_off'
                : volume < 0.5
                ? 'volume_down'
                : 'volume_up'}
            </span>
          </button>

          {/* Expressive Volume Slider */}
          <div className="w-24">
            <input
              type="range"
              min={0}
              max={1}
              step={0.01}
              value={currentVol}
              aria-label="Volume level"
              aria-valuemin={0}
              aria-valuemax={100}
              aria-valuenow={Math.round(volumePct)}
              onChange={(e) => onVolumeChange(Number(e.target.value))}
              style={{
                background: `linear-gradient(to right, #FFFFFF ${volumePct}%, #2A2A2A ${volumePct}%)`,
              }}
              className="w-full cursor-pointer"
            />
          </div>
        </div>
      </div>
    </footer>
  );
};