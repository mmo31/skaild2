import React, { useEffect, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { Application, getApplications, createApplication } from '../services/api';
import { ApplicationForm } from '../components/ApplicationForm';

export const ApplicationsPage: React.FC = () => {
  const navigate = useNavigate();
  const [applications, setApplications] = useState<Application[]>([]);
  const [loading, setLoading] = useState(true);
  const [fetchError, setFetchError] = useState('');
  const [showForm, setShowForm] = useState(false);
  const [formLoading, setFormLoading] = useState(false);
  const [formError, setFormError] = useState('');

  useEffect(() => {
    getApplications()
      .then(setApplications)
      .catch(() => setFetchError('Failed to load applications'))
      .finally(() => setLoading(false));
  }, []);

  const handleCreate = async (data: { name: string; upstream_url: string; hostname: string }) => {
    setFormLoading(true);
    setFormError('');
    try {
      const app = await createApplication(data as any);
      setApplications((prev) => [...prev, app]);
      setShowForm(false);
    } catch (err: any) {
      setFormError(err.response?.data?.error || 'Failed to create application');
    } finally {
      setFormLoading(false);
    }
  };

  return (
    <div className="space-y-6">
      <header className="flex items-center justify-between">
        <h1 className="text-2xl font-semibold text-slate-900">Applications</h1>
        {!showForm && (
          <button
            onClick={() => setShowForm(true)}
            className="mc-button-primary px-4 py-2 text-sm"
          >
            Add Application
          </button>
        )}
      </header>

      {showForm && (
        <div className="mc-surface p-6">
          <h2 className="text-lg font-semibold text-slate-100 mb-4">New Application</h2>
          <ApplicationForm
            onSubmit={handleCreate as any}
            onCancel={() => { setShowForm(false); setFormError(''); }}
            loading={formLoading}
            error={formError}
          />
        </div>
      )}

      {loading && <p className="text-slate-400 text-sm">Loading…</p>}
      {fetchError && (
        <p className="text-red-500 text-sm">{fetchError}</p>
      )}

      {!loading && !fetchError && applications.length === 0 && !showForm && (
        <div className="mc-surface p-10 text-center space-y-3">
          <p className="text-slate-300">No applications yet.</p>
          <button
            onClick={() => setShowForm(true)}
            className="mc-button-primary px-4 py-2 text-sm"
          >
            Add your first one
          </button>
        </div>
      )}

      {applications.length > 0 && (
        <div className="mc-surface overflow-hidden">
          <table className="w-full text-sm text-left">
            <thead className="border-b border-slate-700">
              <tr className="text-slate-400 text-xs uppercase tracking-wide">
                <th className="px-4 py-3">Name</th>
                <th className="px-4 py-3">Hostname</th>
                <th className="px-4 py-3">Upstream URL</th>
                <th className="px-4 py-3">Status</th>
                <th className="px-4 py-3">Actions</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-slate-700/60">
              {applications.map((app) => (
                <tr key={app.id} className="hover:bg-slate-800/40 transition">
                  <td className="px-4 py-3 font-medium text-slate-100">{app.name}</td>
                  <td className="px-4 py-3 text-slate-300 font-mono text-xs">{app.hostname}</td>
                  <td className="px-4 py-3 text-slate-300 font-mono text-xs">{app.upstream_url}</td>
                  <td className="px-4 py-3">
                    <span className={`px-2 py-0.5 rounded-full text-xs font-medium ${
                      app.enabled
                        ? 'bg-green-500/20 text-green-400'
                        : 'bg-slate-500/20 text-slate-400'
                    }`}>
                      {app.enabled ? 'Enabled' : 'Disabled'}
                    </span>
                  </td>
                  <td className="px-4 py-3">
                    <button
                      onClick={() => navigate(`/applications/${app.id}`)}
                      className="text-mc-teal hover:text-mc-teal/80 text-xs font-medium transition"
                    >
                      Edit
                    </button>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
};
