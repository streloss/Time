import React from 'react';

interface HeaderProps {
  searchQuery: string;
  onSearchChange: (q: string) => void;
  onSearchSubmit: (q: string) => void;
  onMinimize?: () => void;
  onMaximize?: () => void;
  onClose?: () => void;
}

export const Header: React.FC<HeaderProps> = ({
  searchQuery,
  onSearchChange,
  onSearchSubmit,
  onMinimize,
  onMaximize,
  onClose,
}) => {
  return (
    <header
      data-tauri-drag-region
      className="h-16 px-6 flex items-center justify-between border-b border-neutral-900 bg-black select-none z-30 flex-shrink-0"
    >
      {/* Search Input Bar */}
      <div className="flex-1 max-w-md" data-tauri-drag-region="false">
        <form
          onSubmit={(e) => {
            e.preventDefault();
            onSearchSubmit(searchQuery);
          }}
          className="relative flex items-center"
        >
          <span
            className="material-symbols-rounded absolute left-3.5 text-neutral-500 text-xl pointer-events-none"
            aria-hidden="true"
          >
            search
          </span>
          <input
            type="text"
            aria-label="Search tracks, artists, or paste music link"
            placeholder="Search tracks, artists, or paste link..."
            value={searchQuery}
            onChange={(e) => onSearchChange(e.target.value)}
            className="w-full bg-[#161616] text-sm text-white placeholder-neutral-500 pl-11 pr-20 py-2 rounded-full border border-neutral-800 focus:outline-none focus:border-neutral-500 focus:bg-[#1E1E1E] focus-visible:ring-2 focus-visible:ring-neutral-400 transition-all font-sans"
          />

          {/* Shortcut hint or Clear button */}
          {searchQuery ? (
            <button
              type="button"
              onClick={() => onSearchChange('')}
              aria-label="Clear search"
              className="absolute right-3 text-neutral-400 hover:text-white p-1 rounded-full hover:bg-neutral-800 transition-colors"
            >
              <span className="material-symbols-rounded text-base">close</span>
            </button>
          ) : (
            <div className="absolute right-3.5 flex items-center pointer-events-none">
              <kbd className="px-1.5 py-0.5 text-[10px] font-mono text-neutral-500 bg-neutral-900 border border-neutral-800 rounded">
                Ctrl K
              </kbd>
            </div>
          )}
        </form>
      </div>

      {/* Window Controls (Native Desktop Look) */}
      <div className="flex items-center gap-1.5 ml-4" data-tauri-drag-region="false">
        <button
          type="button"
          onClick={onMinimize}
          title="Minimize"
          aria-label="Minimize window"
          className="w-8 h-8 rounded-lg flex items-center justify-center text-neutral-400 hover:text-white hover:bg-neutral-800/80 active:scale-95 transition-all focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-neutral-400"
        >
          <span className="material-symbols-rounded text-base">remove</span>
        </button>
        <button
          type="button"
          onClick={onMaximize}
          title="Maximize"
          aria-label="Maximize window"
          className="w-8 h-8 rounded-lg flex items-center justify-center text-neutral-400 hover:text-white hover:bg-neutral-800/80 active:scale-95 transition-all focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-neutral-400"
        >
          <span className="material-symbols-rounded text-sm">crop_square</span>
        </button>
        <button
          type="button"
          onClick={onClose}
          title="Close to Tray"
          aria-label="Close to system tray"
          className="w-8 h-8 rounded-lg flex items-center justify-center text-neutral-400 hover:text-white hover:bg-red-600 active:scale-95 transition-all focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-red-400"
        >
          <span className="material-symbols-rounded text-base">close</span>
        </button>
      </div>
    </header>
  );
};