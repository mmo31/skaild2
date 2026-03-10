import React from 'react';
import { useNavigate, useLocation } from 'react-router-dom';
import { useAuth } from '../hooks/useAuth';
import { ApplicationsPage } from './ApplicationsPage';
import { ApplicationDetailPage } from './ApplicationDetailPage';

interface DashboardPageProps {
  activePage?: string;
}

const navItems = [
  { label: 'Dashboard', path: '/dashboard' },
  { label: 'Applications', path: '/applications' },
  { label: 'Routes', path: '/routes' },
  { label: 'Identity', path: '/identity' },
  { label: 'Certificates', path: '/certificates' },
  { label: 'Settings', path: '/settings' },
];

export const DashboardPage: React.FC<DashboardPageProps> = ({ activePage }) => {
  const navigate = useNavigate();
  const location = useLocation();
  const { admin, logout } = useAuth();

  const isApplicationDetail = location.pathname.startsWith('/applications/') &&
    location.pathname !== '/applications';

  const renderContent = () => {
    if (isApplicationDetail) return <ApplicationDetailPage />;
    if (activePage === 'applications') return <ApplicationsPage />;
    return (
      <>
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
            <p className="text-sm text-slate-300">
              Authentication system is now active. You can now configure routes and identity providers.
            </p>
          </article>
          <article className="mc-surface p-5 space-y-3">
            <h2 className="text-sm font-semibold text-slate-200 uppercase tracking-wide">
              Routes
            </h2>
            <p className="text-sm text-slate-300">Configure proxied routes to your applications.</p>
          </article>
          <article className="mc-surface p-5 space-y-3">
            <h2 className="text-sm font-semibold text-slate-200 uppercase tracking-wide">
              Identity
            </h2>
            <p className="text-sm text-slate-300">Connect identity providers for SSO authentication.</p>
          </article>
        </section>
      </>
    );
  };

  return (
    <div className="min-h-screen bg-mc-ocean text-text-primary flex">
      {/* Sidebar */}
      <aside className="w-64 p-4 space-y-4 border-r border-slate-700/60 hidden md:block">
        <div className="text-xl font-semibold tracking-tight text-slate-900">skaild2</div>
        <nav className="space-y-2" aria-label="Main navigation">
          {navItems.map((item) => {
            const isActive = location.pathname === item.path ||
              (item.path === '/applications' && location.pathname.startsWith('/applications'));
            return (
              <button
                key={item.label}
                onClick={() => navigate(item.path)}
                className={`w-full text-left px-3 py-2 rounded-mc-button transition font-medium ${
                  isActive
                    ? 'bg-mc-teal/20 text-mc-teal'
                    : 'bg-slate-800/80 hover:bg-slate-700/80 text-white hover:text-slate-300'
                }`}
              >
                {item.label}
              </button>
            );
          })}
        </nav>

        {/* User info and logout */}
        <div className="pt-4 border-t border-slate-700/60">
          <div className="text-sm text-slate-600 mb-2">Logged in as:</div>
          <div className="text-sm font-medium text-slate-800 mb-1">{admin?.name}</div>
          <div className="text-xs text-slate-600 truncate mb-3">{admin?.email}</div>
          <button
            onClick={logout}
            className="w-full text-left px-3 py-2 rounded-mc-button bg-red-500/10 hover:bg-red-500/20 text-red-600 transition font-medium text-sm"
          >
            Logout
          </button>
        </div>
      </aside>

      {/* Main content */}
      <main className="flex-1 p-6 space-y-6">
        {renderContent()}
      </main>
    </div>
  );
};
