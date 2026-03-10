import React, { useEffect, useState } from 'react';
import { useNavigate, useParams } from 'react-router-dom';
import {
  Application,
  Route,
  CreateRouteInput,
  getApplications,
  updateApplication,
  getRoutes,
  createRoute,
} from '../services/api';
import { ApplicationForm } from '../components/ApplicationForm';
import { RouteForm } from '../components/RouteForm';

export const ApplicationDetailPage: React.FC = () => {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const [application, setApplication] = useState<Application | null>(null);
  const [loading, setLoading] = useState(true);
  const [fetchError, setFetchError] = useState('');
  const [editing, setEditing] = useState(false);
  const [formLoading, setFormLoading] = useState(false);
  const [formError, setFormError] = useState('');

  const [routes, setRoutes] = useState<Route[]>([]);
  const [routesLoading, setRoutesLoading] = useState(true);
  const [routesError, setRoutesError] = useState('');
  const [addingRoute, setAddingRoute] = useState(false);
  const [addRouteLoading, setAddRouteLoading] = useState(false);
  const [addRouteError, setAddRouteError] = useState('');

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

  useEffect(() => {
    if (!id) return;
    getRoutes(id)
      .then(setRoutes)
      .catch(() => setRoutesError('Failed to load routes'))
      .finally(() => setRoutesLoading(false));
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

  const handleAddRoute = async (data: CreateRouteInput) => {
    if (!id) return;
    setAddRouteLoading(true);
    setAddRouteError('');
    try {
      const newRoute = await createRoute(id, data);
      setRoutes((prev) => [...prev, newRoute]);
      setAddingRoute(false);
    } catch (err: any) {
      setAddRouteError(err.response?.data?.error || 'Failed to create route');
      throw err;
    } finally {
      setAddRouteLoading(false);
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

      {/* Routes section */}
      <div className="mc-surface p-6 space-y-4">
        <div className="flex items-center justify-between">
          <h2 className="text-lg font-semibold text-slate-100">Routes</h2>
          {!addingRoute && (
            <button
              onClick={() => { setAddingRoute(true); setAddRouteError(''); }}
              className="mc-button-primary px-3 py-1.5 text-xs"
            >
              + Add Route
            </button>
          )}
        </div>

        {addingRoute && (
          <div className="border border-slate-600/50 rounded-lg p-4 bg-slate-800/30">
            <h3 className="text-sm font-medium text-slate-300 mb-3">New Route</h3>
            <RouteForm
              applicationHostname={application.hostname}
              onSubmit={handleAddRoute}
              onCancel={() => { setAddingRoute(false); setAddRouteError(''); }}
              loading={addRouteLoading}
              error={addRouteError}
            />
          </div>
        )}

        {routesLoading ? (
          <p className="text-slate-400 text-sm">Loading routes…</p>
        ) : routesError ? (
          <p className="text-red-400 text-sm">{routesError}</p>
        ) : routes.length === 0 ? (
          <p className="text-slate-500 text-sm">No routes yet. Add your first one.</p>
        ) : (
          <table className="w-full text-sm">
            <thead>
              <tr className="text-left text-xs text-slate-400 uppercase tracking-wide border-b border-slate-700">
                <th className="pb-2 pr-4">Host</th>
                <th className="pb-2 pr-4">Path Prefix</th>
                <th className="pb-2 pr-4">Access Mode</th>
                <th className="pb-2">Status</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-slate-700/50">
              {routes.map((route) => (
                <tr key={route.id}>
                  <td className="py-2.5 pr-4 font-mono text-slate-200">{route.host}</td>
                  <td className="py-2.5 pr-4 font-mono text-slate-300">{route.path_prefix}</td>
                  <td className="py-2.5 pr-4">
                    <span className={`px-2 py-0.5 rounded-full text-xs font-medium ${
                      route.access_mode === 'public'
                        ? 'bg-blue-500/20 text-blue-400'
                        : 'bg-yellow-500/20 text-yellow-400'
                    }`}>
                      {route.access_mode === 'public' ? 'Public' : 'Login required'}
                    </span>
                  </td>
                  <td className="py-2.5">
                    <span className={`px-2 py-0.5 rounded-full text-xs font-medium ${
                      route.enabled
                        ? 'bg-green-500/20 text-green-400'
                        : 'bg-slate-500/20 text-slate-400'
                    }`}>
                      {route.enabled ? 'Enabled' : 'Disabled'}
                    </span>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
      </div>
    </div>
  );
};
