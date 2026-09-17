import React from 'react';
import { NavTab } from '../types';

interface NavigationRailProps {
  activeTab: NavTab;
  onTabChange: (tab: NavTab) => void;
}

interface NavItem {
  id: NavTab;
  label: string;
  icon: string;
}

export const NavigationRail: React.FC<NavigationRailProps> = ({ activeTab, onTabChange }) => {
  const navItems: NavItem[] = [
    { id: 'home', label: 'Home', icon: 'home' },
    { id: 'search', label: 'Search', icon: 'search' },
    { id: 'library', label: 'Library', icon: 'library_music' },
    { id: 'downloads', label: 'Downloads', icon: 'download' },
    { id: 'settings', label: 'Settings', icon: 'settings' },
  ];

  return (
    <aside
      className="w-64 h-full bg-black flex flex-col justify-between p-4 border-r border-neutral-900 select-none flex-shrink-0"
      aria-label="Sidebar navigation"
    >
      <div className="flex flex-col gap-6">
        {/* Brand Header */}
        <div className="flex items-center gap-3 px-3 pt-2">
          <div className="w-8 h-8 rounded-full bg-white text-black flex items-center justify-center font-bold text-sm shadow-sm">
            <span className="material-symbols-rounded text-lg text-black filled">schedule</span>
          </div>
          <div className="flex flex-col">
            <h1 className="text-xl font-bold tracking-tight text-white font-sans leading-none">
              Time
            </h1>
            <span className="text-[10px] text-neutral-500 font-mono tracking-wider uppercase mt-0.5">
              Material You
            </span>
          </div>
        </div>

        {/* Navigation Items (Pixel Material You Pill Style) */}
        <nav className="flex flex-col gap-1.5" aria-label="Main navigation">
          {navItems.map((item) => {
            const isActive = activeTab === item.id;
            return (
              <button
                key={item.id}
                type="button"
                onClick={() => onTabChange(item.id)}
                aria-current={isActive ? 'page' : undefined}
                className={`group flex items-center gap-3.5 px-4 py-3 rounded-full text-sm font-medium transition-all duration-150 active:scale-[0.98] focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-white ${
                  isActive
                    ? 'bg-white text-black font-semibold shadow-sm'
                    : 'text-neutral-400 hover:text-white hover:bg-neutral-900/80'
                }`}
              >
                <span
                  className={`material-symbols-rounded text-[22px] transition-transform group-hover:scale-105 ${
                    isActive ? 'filled text-black' : 'text-neutral-400 group-hover:text-white'
                  }`}
                >
                  {item.icon}
                </span>
                <span className="tracking-tight">{item.label}</span>
              </button>
            );
          })}
        </nav>
      </div>

      {/* Footer / System Status */}
      <div className="px-4 py-3 bg-neutral-950 rounded-2xl border border-neutral-900">
        <div className="flex items-center gap-2 text-xs text-neutral-300">
          <span className="w-2 h-2 rounded-full bg-emerald-400 animate-pulse" aria-hidden="true"></span>
          <span className="font-medium">Tray Mode Ready</span>
        </div>
        <div className="text-[11px] text-neutral-500 mt-1 font-mono">
          Memory: ~38 MB • CPU: 0.2%
        </div>
      </div>
    </aside>
  );
};