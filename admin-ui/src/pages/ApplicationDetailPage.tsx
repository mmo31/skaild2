import React, { useEffect, useState } from 'react';
import { useNavigate, useParams } from 'react-router-dom';
import {
  Application,
  Route,
  CreateRouteInput,
  UpdateRouteInput,
  ConnectionTestResult,
  getApplications,
  updateApplication,
  getRoutes,
  createRoute,
  updateRoute,
  testRoute,
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
  const [testResults, setTestResults] = useState<Record<string, ConnectionTestResult | 'loading'>>({});
  const [editingRouteId, setEditingRouteId] = useState<string | null>(null);
  const [editRouteLoading, setEditRouteLoading] = useState(false);
  const [editRouteError, setEditRouteError] = useState('');

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

  const handleUpdateRoute = async (routeId: string, data: UpdateRouteInput) => {
    setEditRouteLoading(true);
    setEditRouteError('');
    try {
      const updated = await updateRoute(routeId, data);
      setRoutes((prev) => prev.map((r) => (r.id === routeId ? updated : r)));
      setEditingRouteId(null);
    } catch (err: any) {
      console.error('updateRoute error', err.response?.status, err.response?.data, err.message);
      setEditRouteError(
        err.response?.data?.error ||
        (err.response?.status ? `Server error ${err.response.status}` : err.message) ||
        'Failed to update route'
      );
    } finally {
      setEditRouteLoading(false);
    }
  };

  const handleTestRoute = async (routeId: string) => {
    setTestResults((prev) => ({ ...prev, [routeId]: 'loading' }));
    try {
      const result = await testRoute(routeId);
      setTestResults((prev) => ({ ...prev, [routeId]: result }));
    } catch {
      setTestResults((prev) => ({
        ...prev,
        [routeId]: {
          status: 'error',
          error_kind: 'error',
          error_message: 'Request failed',
        },
      }));
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
                <th className="pb-2 pr-4">Status</th>
                <th className="pb-2">Actions</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-slate-700/50">
              {routes.map((route) => {
                const testResult = testResults[route.id];
                const isEditingThis = editingRouteId === route.id;
                return (
                <React.Fragment key={route.id}>
                <tr className={isEditingThis ? 'opacity-50' : ''}>
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
                  <td className="py-2.5 pr-4">
                    <span className={`px-2 py-0.5 rounded-full text-xs font-medium ${
                      route.enabled
                        ? 'bg-green-500/20 text-green-400'
                        : 'bg-slate-500/20 text-slate-400'
                    }`}>
                      {route.enabled ? 'Enabled' : 'Disabled'}
                    </span>
                  </td>
                  <td className="py-2.5">
                    <div className="flex flex-col gap-1">
                      <div className="flex gap-1.5">
                        <button
                          onClick={() => handleTestRoute(route.id)}
                          disabled={testResult === 'loading' || isEditingThis}
                          className="mc-button-primary px-2 py-1 text-xs disabled:opacity-50"
                        >
                          {testResult === 'loading' ? 'Testing…' : 'Test'}
                        </button>
                        <button
                          onClick={() => {
                            setEditingRouteId(isEditingThis ? null : route.id);
                            setEditRouteError('');
                          }}
                          className={`px-2 py-1 text-xs rounded border transition ${
                            isEditingThis
                              ? 'border-slate-500 text-slate-300 bg-slate-700/50'
                              : 'border-slate-600 text-slate-300 hover:border-slate-400 hover:text-slate-100'
                          }`}
                        >
                          {isEditingThis ? 'Cancel' : 'Edit'}
                        </button>
                      </div>
                      {testResult && testResult !== 'loading' && (
                        <div className="flex flex-col gap-0.5">
                          {testResult.status === 'ok' ? (
                            <span className="px-2 py-0.5 rounded text-xs font-medium bg-green-500/20 text-green-400">
                              ✓ Reachable (HTTP {testResult.http_status}){testResult.latency_ms != null ? ` · ${testResult.latency_ms}ms` : ''}
                            </span>
                          ) : (
                            <span className="px-2 py-0.5 rounded text-xs font-medium bg-red-500/20 text-red-400">
                              ✗ {testResult.error_message}
                            </span>
                          )}
                          {testResult.auth_check && !testResult.auth_check.configured && (
                            <span className="px-2 py-0.5 rounded text-xs font-medium bg-yellow-500/20 text-yellow-400">
                              ⚠ No IdP configured
                            </span>
                          )}
                        </div>
                      )}
                    </div>
                  </td>
                </tr>
                {isEditingThis && (
                  <tr>
                    <td colSpan={5} className="pb-3 pt-1">
                      <div className="border border-slate-600/50 rounded-lg p-4 bg-slate-800/30">
                        <h3 className="text-sm font-medium text-slate-300 mb-3">Edit Route</h3>
                        <RouteForm
                          applicationHostname={application.hostname}
                          initial={{
                            host: route.host,
                            path_prefix: route.path_prefix,
                            access_mode: route.access_mode,
                            enabled: route.enabled,
                          }}
                          onSubmit={(data) => handleUpdateRoute(route.id, data as UpdateRouteInput)}
                          onCancel={() => { setEditingRouteId(null); setEditRouteError(''); }}
                          loading={editRouteLoading}
                          error={editRouteError}
                        />
                      </div>
                    </td>
                  </tr>
                )}
                </React.Fragment>
                );
              })}
            </tbody>
          </table>
        )}
      </div>
    </div>
  );
};
