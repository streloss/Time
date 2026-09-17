import React from 'react';
import { Track } from '../types';

interface TrackListProps {
  title: string;
  tracks: Track[];
  currentTrackId?: string;
  isPlaying: boolean;
  onSelectTrack: (track: Track) => void;
  onTogglePlay: () => void;
  onDownloadTrack?: (track: Track) => void;
  onToggleLike?: (track: Track) => void;
}

export const TrackList: React.FC<TrackListProps> = ({
  title,
  tracks,
  currentTrackId,
  isPlaying,
  onSelectTrack,
  onTogglePlay,
  onDownloadTrack,
  onToggleLike,
}) => {
  const formatDuration = (seconds: number) => {
    const mins = Math.floor(seconds / 60);
    const secs = Math.floor(seconds % 60);
    return `${mins}:${secs < 10 ? '0' : ''}${secs}`;
  };

  return (
    <section className="flex flex-col gap-3" aria-label={title}>
      <div className="flex items-center justify-between">
        <h3 className="text-xl font-bold tracking-tight text-white font-sans">
          {title}
        </h3>
        <span className="text-xs text-neutral-500 font-mono">
          {tracks.length} {tracks.length === 1 ? 'track' : 'tracks'}
        </span>
      </div>

      {tracks.length === 0 ? (
        <div className="py-12 px-4 rounded-3xl bg-[#111111] border border-neutral-900 flex flex-col items-center justify-center text-center gap-2">
          <span className="material-symbols-rounded text-3xl text-neutral-600">
            queue_music
          </span>
          <p className="text-sm font-medium text-neutral-300">No tracks found</p>
          <p className="text-xs text-neutral-500 max-w-xs">
            Try searching for a different song or artist, or add music to this list.
          </p>
        </div>
      ) : (
        <div className="flex flex-col gap-1" role="list">
          {tracks.map((track, idx) => {
            const isCurrent = track.id === currentTrackId;

            return (
              <div
                key={track.id}
                role="button"
                tabIndex={0}
                aria-selected={isCurrent}
                onClick={() => {
                  if (isCurrent) {
                    onTogglePlay();
                  } else {
                    onSelectTrack(track);
                  }
                }}
                onKeyDown={(e) => {
                  if (e.key === 'Enter' || e.key === ' ') {
                    e.preventDefault();
                    if (isCurrent) {
                      onTogglePlay();
                    } else {
                      onSelectTrack(track);
                    }
                  }
                }}
                className={`group flex items-center justify-between px-3.5 py-2.5 rounded-2xl cursor-pointer transition-all duration-150 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-neutral-400 ${
                  isCurrent
                    ? 'bg-neutral-900 border border-neutral-800 text-white'
                    : 'hover:bg-neutral-900/70 border border-transparent text-neutral-300'
                }`}
              >
                {/* Left: Index / Play Indicator & Track Info */}
                <div className="flex items-center gap-3.5 flex-1 min-w-0 pr-4">
                  {/* Index / Play / Equalizer indicator */}
                  <div className="w-6 text-center text-xs text-neutral-500 group-hover:text-white flex items-center justify-center">
                    {isCurrent && isPlaying ? (
                      <div className="flex items-end justify-center gap-0.5 h-3.5 w-3.5" title="Playing">
                        <span className="w-1 bg-white rounded-full animate-bounce [animation-delay:-0.3s] h-3.5"></span>
                        <span className="w-1 bg-white rounded-full animate-bounce [animation-delay:-0.15s] h-2.5"></span>
                        <span className="w-1 bg-white rounded-full animate-bounce h-3"></span>
                      </div>
                    ) : (
                      <>
                        <span className="group-hover:hidden font-mono text-[11px]">{idx + 1}</span>
                        <span className="hidden group-hover:inline-block material-symbols-rounded text-base text-white">
                          {isCurrent && isPlaying ? 'pause' : 'play_arrow'}
                        </span>
                      </>
                    )}
                  </div>

                  {/* Album Cover Thumbnail */}
                  <div className="w-10 h-10 rounded-xl bg-neutral-900 flex-shrink-0 overflow-hidden relative border border-neutral-800">
                    {track.coverUrl ? (
                      <img
                        src={track.coverUrl}
                        alt={track.title}
                        className="w-full h-full object-cover"
                        loading="lazy"
                      />
                    ) : (
                      <div className="w-full h-full flex items-center justify-center bg-neutral-900 text-neutral-500">
                        <span className="material-symbols-rounded text-lg">music_note</span>
                      </div>
                    )}
                  </div>

                  {/* Title and Artist */}
                  <div className="flex flex-col truncate">
                    <span
                      className={`text-sm font-semibold truncate ${
                        isCurrent ? 'text-white' : 'text-neutral-200'
                      }`}
                    >
                      {track.title}
                    </span>
                    <span className="text-xs text-neutral-400 truncate">
                      {track.artist}
                    </span>
                  </div>
                </div>

                {/* Right: Explicit tag, Like button, Offline status, Duration */}
                <div className="flex items-center gap-2.5 text-xs text-neutral-400 flex-shrink-0">
                  {track.isExplicit && (
                    <span className="px-1.5 py-0.5 rounded bg-neutral-800 text-[10px] font-mono font-semibold text-neutral-400 border border-neutral-700/50">
                      E
                    </span>
                  )}

                  {/* Like Button */}
                  {onToggleLike && (
                    <button
                      type="button"
                      onClick={(e) => {
                        e.stopPropagation();
                        onToggleLike(track);
                      }}
                      title={track.isLiked ? 'Unlike' : 'Like'}
                      aria-label={track.isLiked ? 'Unlike track' : 'Like track'}
                      className={`p-1.5 rounded-full hover:bg-neutral-800 transition-all ${
                        track.isLiked
                          ? 'text-white opacity-100'
                          : 'text-neutral-500 hover:text-white opacity-0 group-hover:opacity-100'
                      }`}
                    >
                      <span className={`material-symbols-rounded text-lg ${track.isLiked ? 'filled text-white' : ''}`}>
                        favorite
                      </span>
                    </button>
                  )}

                  {/* Offline Download Button */}
                  {onDownloadTrack && (
                    <button
                      type="button"
                      onClick={(e) => {
                        e.stopPropagation();
                        onDownloadTrack(track);
                      }}
                      title={track.isOffline ? 'Downloaded (Offline)' : 'Download offline'}
                      aria-label={track.isOffline ? 'Remove download' : 'Download track offline'}
                      className={`p-1.5 rounded-full hover:bg-neutral-800 transition-all ${
                        track.isOffline
                          ? 'text-white opacity-100'
                          : 'text-neutral-500 hover:text-white opacity-0 group-hover:opacity-100'
                      }`}
                    >
                      <span className="material-symbols-rounded text-lg">
                        {track.isOffline ? 'download_done' : 'download'}
                      </span>
                    </button>
                  )}

                  <span className="w-10 text-right tabular-nums text-neutral-400 font-mono text-xs">
                    {formatDuration(track.duration)}
                  </span>
                </div>
              </div>
            );
          })}
        </div>
      )}
    </section>
  );
};