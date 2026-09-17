export interface Track {
  id: string;
  title: string;
  artist: string;
  album?: string;
  duration: number; // in seconds
  coverUrl?: string;
  audioUrl?: string;
  isExplicit?: boolean;
  isOffline?: boolean;
  isLiked?: boolean;
}

export interface LyricLine {
  time: number; // seconds
  text: string;
}

export type NavTab = 'home' | 'search' | 'library' | 'downloads' | 'settings';

export interface PlayerState {
  currentTrack: Track | null;
  isPlaying: boolean;
  currentTime: number;
  duration: number;
  volume: number;
  isMuted: boolean;
  isShuffle: boolean;
  isRepeat: boolean;
  queue: Track[];
  history: Track[];
}