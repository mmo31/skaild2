import React, { useEffect, useState } from 'react';
import { useNavigate, useParams } from 'react-router-dom';
import { Application, getApplications, updateApplication } from '../services/api';
import { ApplicationForm } from '../components/ApplicationForm';

export const ApplicationDetailPage: React.FC = () => {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const [application, setApplication] = useState<Application | null>(null);
  const [loading, setLoading] = useState(true);
  const [fetchError, setFetchError] = useState('');
  const [editing, setEditing] = useState(false);
  const [formLoading, setFormLoading] = useState(false);
  const [formError, setFormError] = useState('');

  useEffect(() => {
    // Fetch by id from the list endpoint (no single GET added to api.ts per story scope)
    getApplications()
      .then((apps) => {
        const found = apps.find((a) => a.id === id);
        if (!found) setFetchError('Application not found');
        else setApplication(found);
      })
      .catch(() => setFetchError('Failed to load application'))
      .finally(() => setLoading(false));
  }, [id]);

  const handleUpdate = async (data: { name?: string; upstream_url?: string; hostname?: string }) => {
    if (!id) return;
    setFormLoading(true);
    setFormError('');
    try {
      const updated = await updateApplication(id, data);
      setApplication(updated);
      setEditing(false);
    } catch (err: any) {
      setFormError(err.response?.data?.error || 'Failed to update application');
    } finally {
      setFormLoading(false);
    }
  };

  if (loading) return <p className="text-slate-400 text-sm p-6">Loading…</p>;
  if (fetchError) return <p className="text-red-500 text-sm p-6">{fetchError}</p>;
  if (!application) return null;

  return (
    <div className="space-y-6 max-w-2xl">
      <header className="flex items-center gap-3">
        <button
          onClick={() => navigate('/applications')}
          className="text-slate-400 hover:text-slate-200 text-sm transition"
        >
          ← Applications
        </button>
        <h1 className="text-2xl font-semibold text-slate-900">{application.name}</h1>
      </header>

      <div className="mc-surface p-6 space-y-4">
        {editing ? (
          <>
            <h2 className="text-lg font-semibold text-slate-100">Edit Application</h2>
            <ApplicationForm
              initial={application}
              onSubmit={handleUpdate as any}
              onCancel={() => { setEditing(false); setFormError(''); }}
              loading={formLoading}
              error={formError}
            />
          </>
        ) : (
          <>
            <dl className="space-y-3 text-sm">
              <div>
                <dt className="text-slate-400 text-xs uppercase tracking-wide mb-0.5">Name</dt>
                <dd className="text-slate-100 font-medium">{application.name}</dd>
              </div>
              <div>
                <dt className="text-slate-400 text-xs uppercase tracking-wide mb-0.5">Hostname</dt>
                <dd className="text-slate-100 font-mono">{application.hostname}</dd>
              </div>
              <div>
                <dt className="text-slate-400 text-xs uppercase tracking-wide mb-0.5">Upstream URL</dt>
                <dd className="text-slate-100 font-mono">{application.upstream_url}</dd>
              </div>
              <div>
                <dt className="text-slate-400 text-xs uppercase tracking-wide mb-0.5">Status</dt>
                <dd>
                  <span className={`px-2 py-0.5 rounded-full text-xs font-medium ${
                    application.enabled
                      ? 'bg-green-500/20 text-green-400'
                      : 'bg-slate-500/20 text-slate-400'
                  }`}>
                    {application.enabled ? 'Enabled' : 'Disabled'}
                  </span>
                </dd>
              </div>
              <div>
                <dt className="text-slate-400 text-xs uppercase tracking-wide mb-0.5">Created</dt>
                <dd className="text-slate-400 text-xs">{new Date(application.created_at).toLocaleString()}</dd>
              </div>
            </dl>
            <div className="pt-2">
              <button
                onClick={() => setEditing(true)}
                className="mc-button-primary px-4 py-2 text-sm"
              >
                Edit
              </button>
            </div>
          </>
        )}
      </div>
    </div>
  );
};
