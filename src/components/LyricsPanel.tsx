import React, { useEffect, useRef } from 'react';
import { LyricLine, Track } from '../types';

interface LyricsPanelProps {
  track: Track | null;
  lyrics: LyricLine[];
  currentTime: number;
  onSeek: (seconds: number) => void;
  onClose?: () => void;
}

export const LyricsPanel: React.FC<LyricsPanelProps> = ({
  track,
  lyrics,
  currentTime,
  onSeek,
  onClose,
}) => {
  const activeLineRef = useRef<HTMLDivElement | null>(null);
  const containerRef = useRef<HTMLDivElement | null>(null);

  // Find active line index
  let activeIndex = -1;
  for (let i = 0; i < lyrics.length; i++) {
    if (lyrics[i].time <= currentTime) {
      activeIndex = i;
    } else {
      break;
    }
  }

  // Format seconds to mm:ss
  const formatTime = (seconds: number) => {
    const mins = Math.floor(seconds / 60);
    const secs = Math.floor(seconds % 60);
    return `${mins}:${secs < 10 ? '0' : ''}${secs}`;
  };

  // Smooth autoscroll to active line
  useEffect(() => {
    if (activeLineRef.current && containerRef.current) {
      activeLineRef.current.scrollIntoView({
        behavior: 'smooth',
        block: 'center',
      });
    }
  }, [activeIndex]);

  if (!track) {
    return (
      <aside
        className="w-80 h-full bg-[#0E0E0E] border-l border-neutral-900 flex flex-col items-center justify-center p-6 text-center text-neutral-500 text-sm flex-shrink-0 select-none"
        aria-label="Lyrics panel"
      >
        <span className="material-symbols-rounded text-3xl text-neutral-600 mb-2">
          lyrics
        </span>
        <p className="text-neutral-400 font-medium">No track selected</p>
        <p className="text-xs text-neutral-500 mt-1">
          Play any track to view synchronized lyrics
        </p>
      </aside>
    );
  }

  return (
    <aside
      className="w-84 h-full bg-[#0E0E0E] border-l border-neutral-900 flex flex-col justify-between select-none flex-shrink-0"
      aria-label="Synchronized lyrics"
    >
      {/* Header */}
      <div className="p-4 border-b border-neutral-900 flex items-center justify-between gap-3">
        <div className="flex items-center gap-3 min-w-0">
          {track.coverUrl && (
            <img
              src={track.coverUrl}
              alt={track.title}
              className="w-9 h-9 rounded-lg object-cover border border-neutral-800 flex-shrink-0"
            />
          )}
          <div className="min-w-0">
            <h4 className="text-sm font-bold text-white tracking-tight truncate">
              {track.title}
            </h4>
            <p className="text-xs text-neutral-400 truncate">
              {track.artist}
            </p>
          </div>
        </div>
        {onClose && (
          <button
            type="button"
            onClick={onClose}
            aria-label="Close lyrics panel"
            className="p-1.5 rounded-full hover:bg-neutral-800 text-neutral-400 hover:text-white transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-neutral-400 flex-shrink-0"
          >
            <span className="material-symbols-rounded text-lg">close</span>
          </button>
        )}
      </div>

      {/* Synchronized Lines Container */}
      <div
        ref={containerRef}
        className="flex-1 overflow-y-auto px-6 py-6 flex flex-col gap-4 scroll-smooth"
      >
        {lyrics.length === 0 ? (
          <div className="flex-1 flex flex-col items-center justify-center text-neutral-500 gap-2 text-center">
            <span className="material-symbols-rounded text-3xl text-neutral-600">
              music_off
            </span>
            <span className="text-sm font-medium text-neutral-300">
              No synced lyrics available
            </span>
            <span className="text-xs text-neutral-500">
              Lyrics could not be found for this song
            </span>
          </div>
        ) : (
          lyrics.map((line, idx) => {
            const isActive = idx === activeIndex;
            const isPast = idx < activeIndex;

            return (
              <div
                key={idx}
                ref={isActive ? activeLineRef : null}
                role="button"
                tabIndex={0}
                onClick={() => onSeek(line.time)}
                onKeyDown={(e) => {
                  if (e.key === 'Enter' || e.key === ' ') {
                    e.preventDefault();
                    onSeek(line.time);
                  }
                }}
                className={`group flex items-center justify-between cursor-pointer py-1 transition-all duration-200 text-left font-sans focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-neutral-400 rounded-lg ${
                  isActive
                    ? 'text-white text-lg font-bold scale-[1.02] origin-left'
                    : isPast
                    ? 'text-neutral-400 hover:text-neutral-200 text-sm font-medium'
                    : 'text-neutral-600 hover:text-neutral-300 text-sm font-medium'
                }`}
              >
                <span className="flex-1 pr-2">{line.text || '♪'}</span>
                <span className="opacity-0 group-hover:opacity-100 font-mono text-[10px] text-neutral-400 bg-neutral-900 px-1.5 py-0.5 rounded border border-neutral-800 transition-opacity">
                  {formatTime(line.time)}
                </span>
              </div>
            );
          })
        )}
      </div>

      {/* Footer Info */}
      <div className="p-3 border-t border-neutral-900 text-center text-[11px] text-neutral-500 font-sans">
        Synced via LRCLIB • Click line to seek
      </div>
    </aside>
  );
};