import React, { useState, useEffect, useRef } from 'react';
import { Track, LyricLine, NavTab } from './types';
import { NavigationRail } from './components/NavigationRail';
import { Header } from './components/Header';
import { WelcomeOnboarding } from './components/WelcomeOnboarding';
import { TrackList } from './components/TrackList';
import { LyricsPanel } from './components/LyricsPanel';
import { PlayerBar } from './components/PlayerBar';

const INITIAL_TRACKS: Track[] = [
  {
    id: '1',
    title: 'Starlight',
    artist: 'The Nocturnals',
    album: 'Astral Memories',
    duration: 225,
    isExplicit: true,
    isLiked: true,
    coverUrl: 'https://images.unsplash.com/photo-1518709268805-4e9042af9f23?w=300&auto=format&fit=crop&q=80',
  },
  {
    id: '2',
    title: 'Echoes',
    artist: 'Aurora',
    album: 'Parallel Horizons',
    duration: 252,
    isExplicit: false,
    isLiked: false,
    coverUrl: 'https://images.unsplash.com/photo-1509198397868-475647b2a1e5?w=300&auto=format&fit=crop&q=80',
  },
  {
    id: '3',
    title: 'Midnight City',
    artist: 'M83',
    album: 'Hurry Up, We Are Dreaming',
    duration: 244,
    isExplicit: false,
    isLiked: true,
    coverUrl: 'https://images.unsplash.com/photo-1470225620780-dba8ba36b745?w=300&auto=format&fit=crop&q=80',
  },
  {
    id: '4',
    title: 'After Dark',
    artist: 'Mr.Kitty',
    album: 'Time',
    duration: 258,
    isExplicit: false,
    isLiked: false,
    coverUrl: 'https://images.unsplash.com/photo-1511671782779-c97d3d27a1d4?w=300&auto=format&fit=crop&q=80',
  },
  {
    id: '5',
    title: 'Resonance',
    artist: 'HOME',
    album: 'Odyssey',
    duration: 212,
    isExplicit: false,
    isLiked: true,
    coverUrl: 'https://images.unsplash.com/photo-1493225457124-a3eb161ffa5f?w=300&auto=format&fit=crop&q=80',
  },
];

const SAMPLE_LYRICS: LyricLine[] = [
  { time: 0, text: '♪ (Instrumental Intro) ♪' },
  { time: 14, text: 'We danced into the night' },
  { time: 24, text: 'Beneath the glowing light' },
  { time: 38, text: 'Where dreams take flight' },
  { time: 52, text: 'And the music fades, but the feeling stays' },
  { time: 68, text: 'Deep down inside our veins' },
  { time: 84, text: 'And the stars ignite the flame' },
  { time: 104, text: 'Holding on to the golden rays' },
  { time: 124, text: 'Time stands still when the melody plays' },
  { time: 148, text: 'Forever echoes in the dark' },
  { time: 172, text: 'A silent spark within the heart' },
  { time: 200, text: '♪ (Outro) ♪' },
];

export const App: React.FC = () => {
  const [activeTab, setActiveTab] = useState<NavTab>('home');
  const [searchQuery, setSearchQuery] = useState('');
  const [tracks, setTracks] = useState<Track[]>(INITIAL_TRACKS);
  const [currentTrack, setCurrentTrack] = useState<Track | null>(INITIAL_TRACKS[0]);
  const [isPlaying, setIsPlaying] = useState(false);
  const [currentTime, setCurrentTime] = useState(28);
  const [duration, setDuration] = useState(INITIAL_TRACKS[0].duration);
  const [volume, setVolume] = useState(0.8);
  const [isMuted, setIsMuted] = useState(false);
  const [isShuffle, setIsShuffle] = useState(false);
  const [isRepeat, setIsRepeat] = useState(false);
  const [isLyricsOpen, setIsLyricsOpen] = useState(true);
  const [lyrics, setLyrics] = useState<LyricLine[]>(SAMPLE_LYRICS);

  const safeInvoke = async <T,>(cmd: string, args?: Record<string, unknown>): Promise<T | null> => {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      return await invoke<T>(cmd, args);
    } catch {
      return null;
    }
  };

  const handleSelectTrack = async (track: Track) => {
    setCurrentTrack(track);
    setDuration(track.duration);
    setCurrentTime(0);
    setIsPlaying(true);

    await safeInvoke('play_track', { track });

    const backendLyrics = await safeInvoke<LyricLine[]>('get_lyrics', {
      title: track.title,
      artist: track.artist,
      album: track.album,
      duration: track.duration,
    });
    if (backendLyrics && backendLyrics.length > 0) {
      setLyrics(backendLyrics);
    }
  };

  const handleTogglePlay = async () => {
    const next = !isPlaying;
    setIsPlaying(next);
    if (next) {
      await safeInvoke('resume_track');
    } else {
      await safeInvoke('pause_track');
    }
  };

  const handleNextTrack = () => {
    if (!currentTrack || tracks.length === 0) return;
    if (isShuffle && tracks.length > 1) {
      const remainingTracks = tracks.filter((t) => t.id !== currentTrack.id);
      const randomIndex = Math.floor(Math.random() * remainingTracks.length);
      handleSelectTrack(remainingTracks[randomIndex]);
      return;
    }
    const currentIndex = tracks.findIndex((t) => t.id === currentTrack.id);
    const nextIndex = (currentIndex + 1) % tracks.length;
    handleSelectTrack(tracks[nextIndex]);
  };

  const handlePreviousTrack = () => {
    if (!currentTrack || tracks.length === 0) return;
    const currentIndex = tracks.findIndex((t) => t.id === currentTrack.id);
    const prevIndex = (currentIndex - 1 + tracks.length) % tracks.length;
    handleSelectTrack(tracks[prevIndex]);
  };

  // Playback timer simulation & auto-advance
  useEffect(() => {
    if (isPlaying) {
      timerRef.current = window.setInterval(() => {
        setCurrentTime((prev) => {
          if (prev >= duration) {
            if (isRepeat) {
              return 0;
            }
            handleNextTrack();
            return 0;
          }
          return prev + 1;
        });
      }, 1000);
    } else if (timerRef.current) {
      clearInterval(timerRef.current);
    }

    return () => {
      if (timerRef.current) clearInterval(timerRef.current);
    };
  }, [isPlaying, duration, isRepeat, isShuffle, currentTrack, tracks]);

  // Global hotkeys (Space for play/pause, Arrows for seek)
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.target instanceof HTMLInputElement) return;

      if (e.code === 'Space') {
        e.preventDefault();
        setIsPlaying((prev) => !prev);
      } else if (e.code === 'ArrowRight') {
        e.preventDefault();
        setCurrentTime((prev) => Math.min(prev + 5, duration));
      } else if (e.code === 'ArrowLeft') {
        e.preventDefault();
        setCurrentTime((prev) => Math.max(prev - 5, 0));
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [duration]);

  const handleSeek = (seconds: number) => {
    setCurrentTime(seconds);
  };

  const handleDownloadTrack = (track: Track) => {
    setTracks((prev) =>
      prev.map((t) => (t.id === track.id ? { ...t, isOffline: !t.isOffline } : t))
    );
  };

  const handleSearchSubmit = async (query: string) => {
    if (!query.trim()) {
      setTracks(INITIAL_TRACKS);
      return;
    }
    const backendTracks = await safeInvoke<Track[]>('search_tracks', { query });
    if (backendTracks && backendTracks.length > 0) {
      setTracks(backendTracks);
    } else {
      const filtered = INITIAL_TRACKS.filter(
        (t) =>
          t.title.toLowerCase().includes(query.toLowerCase()) ||
          t.artist.toLowerCase().includes(query.toLowerCase())
      );
      setTracks(filtered);
    }
    setActiveTab('search');
  };

  // Tauri Window handlers (with safe fallback for web preview)
  const handleWindowClose = async () => {
    try {
      const { getCurrentWindow } = await import('@tauri-apps/api/window');
      await getCurrentWindow().hide();
    } catch {
      console.log('Window minimized to tray.');
    }
  };

  const handleWindowMinimize = async () => {
    try {
      const { getCurrentWindow } = await import('@tauri-apps/api/window');
      await getCurrentWindow().minimize();
    } catch {
      console.log('Window minimized.');
    }
  };

  const handleWindowMaximize = async () => {
    try {
      const { getCurrentWindow } = await import('@tauri-apps/api/window');
      await getCurrentWindow().toggleMaximize();
    } catch {
      console.log('Window maximize toggled.');
    }
  };

  return (
    <div className="flex flex-col h-screen w-screen bg-black text-white overflow-hidden select-none">
      {/* Top Header */}
      <Header
        searchQuery={searchQuery}
        onSearchChange={setSearchQuery}
        onSearchSubmit={handleSearchSubmit}
        onMinimize={handleWindowMinimize}
        onMaximize={handleWindowMaximize}
        onClose={handleWindowClose}
      />

      {/* Main Container: NavRail + Content + Lyrics Panel */}
      <div className="flex flex-1 overflow-hidden">
        <NavigationRail activeTab={activeTab} onTabChange={setActiveTab} />

        {/* Central Scrollable Content Area */}
        <main className="flex-1 overflow-y-auto px-8 py-6 flex flex-col gap-6">
          {activeTab === 'home' && (
            <>
              <WelcomeOnboarding
                onNavigate={setActiveTab}
                onOpenLyrics={() => setIsLyricsOpen(true)}
              />
              <TrackList
                title="Recently Played"
                tracks={tracks}
                currentTrackId={currentTrack?.id}
                isPlaying={isPlaying}
                onSelectTrack={handleSelectTrack}
                onTogglePlay={handleTogglePlay}
                onDownloadTrack={handleDownloadTrack}
              />
            </>
          )}

          {activeTab === 'search' && (
            <TrackList
              title={searchQuery ? `Results for "${searchQuery}"` : 'Discover Tracks'}
              tracks={tracks}
              currentTrackId={currentTrack?.id}
              isPlaying={isPlaying}
              onSelectTrack={handleSelectTrack}
              onTogglePlay={handleTogglePlay}
              onDownloadTrack={handleDownloadTrack}
            />
          )}

          {activeTab === 'library' && (
            <TrackList
              title="Your Library"
              tracks={tracks}
              currentTrackId={currentTrack?.id}
              isPlaying={isPlaying}
              onSelectTrack={handleSelectTrack}
              onTogglePlay={handleTogglePlay}
              onDownloadTrack={handleDownloadTrack}
            />
          )}

          {activeTab === 'downloads' && (
            <TrackList
              title="Downloaded Tracks (Offline)"
              tracks={tracks.filter((t) => t.isOffline)}
              currentTrackId={currentTrack?.id}
              isPlaying={isPlaying}
              onSelectTrack={handleSelectTrack}
              onTogglePlay={handleTogglePlay}
              onDownloadTrack={handleDownloadTrack}
            />
          )}

          {activeTab === 'settings' && (
            <section className="flex flex-col gap-4 max-w-xl">
              <h3 className="text-2xl font-bold tracking-tight text-white font-sans">
                Settings
              </h3>
              <div className="flex flex-col gap-3 mt-2">
                <div className="bg-[#151515] p-4 rounded-2xl border border-neutral-800 flex items-center justify-between">
                  <div>
                    <h5 className="font-semibold text-sm">Minimize to System Tray</h5>
                    <p className="text-xs text-neutral-400">Keep audio playing when window is closed</p>
                  </div>
                  <span className="text-xs px-2.5 py-1 rounded-full bg-neutral-800 text-white font-medium">
                    Enabled
                  </span>
                </div>

                <div className="bg-[#151515] p-4 rounded-2xl border border-neutral-800 flex items-center justify-between">
                  <div>
                    <h5 className="font-semibold text-sm">Hardware Acceleration</h5>
                    <p className="text-xs text-neutral-400">Smooth animations with background throttling</p>
                  </div>
                  <span className="text-xs px-2.5 py-1 rounded-full bg-neutral-800 text-white font-medium">
                    Enabled
                  </span>
                </div>

                <div className="bg-[#151515] p-4 rounded-2xl border border-neutral-800 flex items-center justify-between">
                  <div>
                    <h5 className="font-semibold text-sm">Discord Rich Presence</h5>
                    <p className="text-xs text-neutral-400">Show current track in your Discord status</p>
                  </div>
                  <span className="text-xs px-2.5 py-1 rounded-full bg-neutral-800 text-white font-medium">
                    Active
                  </span>
                </div>
              </div>
            </section>
          )}
        </main>

        {/* Right Lyrics Panel */}
        {isLyricsOpen && (
          <LyricsPanel
            track={currentTrack}
            lyrics={lyrics}
            currentTime={currentTime}
            onSeek={handleSeek}
            onClose={() => setIsLyricsOpen(false)}
          />
        )}
      </div>

      {/* Floating Bottom Player Bar */}
      <PlayerBar
        track={currentTrack}
        isPlaying={isPlaying}
        currentTime={currentTime}
        duration={duration}
        volume={volume}
        isMuted={isMuted}
        isShuffle={isShuffle}
        isRepeat={isRepeat}
        isLyricsOpen={isLyricsOpen}
        onTogglePlay={handleTogglePlay}
        onPrevious={handlePreviousTrack}
        onNext={handleNextTrack}
        onSeek={handleSeek}
        onVolumeChange={(v) => {
          setVolume(v);
          if (isMuted) setIsMuted(false);
          safeInvoke('set_volume', { volume: v });
        }}
        onToggleMute={() => setIsMuted((prev) => !prev)}
        onToggleShuffle={() => setIsShuffle((prev) => !prev)}
        onToggleRepeat={() => setIsRepeat((prev) => !prev)}
        onToggleLyrics={() => setIsLyricsOpen((prev) => !prev)}
      />
    </div>
  );
};
export default App;