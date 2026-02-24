import React from 'react';

const navItems = [
  'Dashboard',
  'Routes',
  'Identity',
  'Certificates',
  'Settings',
];

function App() {
  return (
    <div className="min-h-screen bg-mc-ocean text-text-primary flex">
      <aside className="w-64 p-4 space-y-4 border-r border-slate-700/60 hidden md:block">
        <div className="text-xl font-semibold tracking-tight text-slate-900">skaild2</div>
        <nav className="space-y-2" aria-label="Main navigation">
          {navItems.map((item) => (
            <button
              key={item}
              className="w-full text-left px-3 py-2 rounded-mc-button bg-slate-800/80 hover:bg-slate-700/80 text-white hover:text-slate-300 transition font-medium"
            >
              {item}
            </button>
          ))}
        </nav>
      </aside>
      <main className="flex-1 p-6 space-y-6">
        <header className="flex items-center justify-between">
          <h1 className="text-2xl font-semibold text-slate-900">Admin Dashboard</h1>
          <div className="flex items-center gap-3">
            <button className="mc-button-secondary px-4 py-2 text-sm">Connect IdP</button>
            <button className="mc-button-primary px-4 py-2 text-sm">Add Route</button>
          </div>
        </header>
        <section className="grid gap-6 md:grid-cols-2 xl:grid-cols-3">
          <article className="mc-surface p-5 space-y-3">
            <h2 className="text-sm font-semibold text-slate-200 uppercase tracking-wide">
              Quick Status
            </h2>
            <p className="text-sm text-slate-300">Baseline project scaffold is ready for further configuration stories.</p>
          </article>
          <article className="mc-surface p-5 space-y-3">
            <h2 className="text-sm font-semibold text-slate-200 uppercase tracking-wide">
              Routes
            </h2>
            <p className="text-sm text-slate-300">Placeholder for route list and configuration.</p>
          </article>
          <article className="mc-surface p-5 space-y-3">
            <h2 className="text-sm font-semibold text-slate-200 uppercase tracking-wide">
              Identity
            </h2>
            <p className="text-sm text-slate-300">Placeholder for IdP connections and roles.</p>
          </article>
        </section>
      </main>
    </div>
  );
}

export default App;
