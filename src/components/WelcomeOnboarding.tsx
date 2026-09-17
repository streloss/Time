import React from 'react';
import { NavTab } from '../types';

interface WelcomeOnboardingProps {
  onNavigate: (tab: NavTab) => void;
  onOpenLyrics: () => void;
}

export const WelcomeOnboarding: React.FC<WelcomeOnboardingProps> = ({ onNavigate, onOpenLyrics }) => {
  const cards = [
    {
      id: 'search',
      title: 'Direct Search',
      badge: 'Fast Lookup',
      description: 'Type any song name, artist, or paste a music link to start playback instantly.',
      actionLabel: 'Search Now',
      icon: 'search',
      onClick: () => onNavigate('search'),
    },
    {
      id: 'offline',
      title: 'Offline Storage',
      badge: 'No Internet',
      description: 'Keep your favorite tracks saved locally to play anywhere without an internet connection.',
      actionLabel: 'Open Downloads',
      icon: 'download_for_offline',
      onClick: () => onNavigate('downloads'),
    },
    {
      id: 'lyrics',
      title: 'Synced Lyrics',
      badge: 'Line-by-line',
      description: 'Follow along with real-time synchronized karaoke lyrics fetched automatically.',
      actionLabel: 'Open Lyrics',
      icon: 'lyrics',
      onClick: onOpenLyrics,
    },
  ];

  return (
    <section className="flex flex-col gap-4 mb-6" aria-label="Quick start guide">
      <div>
        <h2 className="text-2xl md:text-3xl font-bold tracking-tight text-white font-sans">
          Welcome to Time
        </h2>
        <p className="text-sm text-neutral-400 mt-1 font-sans">
          Native audio playback. Ad-free, minimal footprint, and instant streaming.
        </p>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-3 gap-4 mt-1">
        {cards.map((card) => (
          <div
            key={card.id}
            className="group relative bg-[#141414] hover:bg-[#1A1A1A] border border-neutral-800 hover:border-neutral-700 rounded-3xl p-5 flex flex-col justify-between transition-all duration-200"
          >
            <div className="flex flex-col gap-3.5">
              <div className="flex items-center justify-between">
                <div className="w-10 h-10 rounded-2xl bg-neutral-900 border border-neutral-800 flex items-center justify-center text-white group-hover:bg-white group-hover:text-black transition-colors duration-200">
                  <span className="material-symbols-rounded text-xl" aria-hidden="true">
                    {card.icon}
                  </span>
                </div>
                <span className="text-[10px] font-mono uppercase tracking-wider text-neutral-400 px-2 py-0.5 rounded-full bg-neutral-900 border border-neutral-800">
                  {card.badge}
                </span>
              </div>

              <div>
                <h3 className="text-base font-semibold text-white tracking-tight">
                  {card.title}
                </h3>
                <p className="text-xs text-neutral-400 mt-1 leading-relaxed">
                  {card.description}
                </p>
              </div>
            </div>

            <div className="mt-5 pt-1">
              <button
                type="button"
                onClick={card.onClick}
                className="w-full py-2.5 px-4 rounded-full bg-white text-black font-semibold text-xs tracking-wide hover:bg-neutral-200 active:scale-[0.98] transition-all focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-white"
              >
                {card.actionLabel}
              </button>
            </div>
          </div>
        ))}
      </div>
    </section>
  );
};