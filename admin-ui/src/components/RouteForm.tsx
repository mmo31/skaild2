import React, { useState } from 'react';
import { CreateRouteInput, UpdateRouteInput } from '../services/api';

interface RouteFormInitial {
  host: string;
  path_prefix: string;
  access_mode: 'public' | 'login_required';
  enabled: boolean;
}

interface RouteFormProps {
  applicationHostname: string;
  initial?: RouteFormInitial;
  onSubmit: (data: CreateRouteInput | UpdateRouteInput) => Promise<void>;
  onCancel: () => void;
  loading?: boolean;
  error?: string;
}

export const RouteForm: React.FC<RouteFormProps> = ({
  applicationHostname,
  initial,
  onSubmit,
  onCancel,
  loading = false,
  error = '',
}) => {
  const isEdit = !!initial;
  const [host, setHost] = useState(initial?.host ?? applicationHostname);
  const [pathPrefix, setPathPrefix] = useState(initial?.path_prefix ?? '/');
  const [accessMode, setAccessMode] = useState<'public' | 'login_required'>(initial?.access_mode ?? 'login_required');
  const [enabled, setEnabled] = useState(initial?.enabled ?? true);
  const [fieldError, setFieldError] = useState('');

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setFieldError('');
    if (!host.trim()) {
      setFieldError('Host is required');
      return;
    }
    try {
      const data = isEdit
        ? { host: host.trim(), path_prefix: pathPrefix.trim() || '/', access_mode: accessMode, enabled }
        : { host: host.trim(), path_prefix: pathPrefix.trim() || '/', access_mode: accessMode };
      await onSubmit(data);
    } catch {
      // error display is managed by the parent
    }
  };

  return (
    <form onSubmit={handleSubmit} className="space-y-4">
      <div>
        <label className="block text-xs text-slate-400 uppercase tracking-wide mb-1">
          Host <span className="text-red-400">*</span>
        </label>
        <input
          type="text"
          value={host}
          onChange={(e) => setHost(e.target.value)}
          placeholder="app.yourdomain.com"
          className="w-full bg-slate-800/50 border border-slate-600 rounded-lg px-3 py-2 text-slate-100 text-sm placeholder-slate-500 focus:outline-none focus:ring-2 focus:ring-accent-aqua"
          required
        />
      </div>

      <div>
        <label className="block text-xs text-slate-400 uppercase tracking-wide mb-1">
          Path Prefix
        </label>
        <input
          type="text"
          value={pathPrefix}
          onChange={(e) => setPathPrefix(e.target.value)}
          placeholder="/"
          className="w-full bg-slate-800/50 border border-slate-600 rounded-lg px-3 py-2 text-slate-100 text-sm placeholder-slate-500 focus:outline-none focus:ring-2 focus:ring-accent-aqua"
        />
        <p className="text-xs text-slate-500 mt-1">Defaults to / if left empty</p>
      </div>

      <div>
        <label className="block text-xs text-slate-400 uppercase tracking-wide mb-2">
          Access Mode <span className="text-red-400">*</span>
        </label>
        <div className="space-y-2">
          <label className="flex items-center gap-2 cursor-pointer">
            <input
              type="radio"
              name="access_mode"
              value="login_required"
              checked={accessMode === 'login_required'}
              onChange={() => setAccessMode('login_required')}
              className="accent-accent-aqua"
            />
            <span className="text-slate-200 text-sm">
              Login required <span className="text-slate-400">(default)</span>
            </span>
          </label>
          <label className="flex items-center gap-2 cursor-pointer">
            <input
              type="radio"
              name="access_mode"
              value="public"
              checked={accessMode === 'public'}
              onChange={() => setAccessMode('public')}
              className="accent-accent-aqua"
            />
            <span className="text-slate-200 text-sm">
              Public <span className="text-slate-400">(no login required)</span>
            </span>
          </label>
        </div>
      </div>

      {isEdit && (
        <div>
          <label className="block text-xs text-slate-400 uppercase tracking-wide mb-2">
            Status
          </label>
          <label className="flex items-center gap-2 cursor-pointer">
            <input
              type="checkbox"
              checked={enabled}
              onChange={(e) => setEnabled(e.target.checked)}
              className="accent-accent-aqua w-4 h-4"
            />
            <span className="text-slate-200 text-sm">Enabled</span>
          </label>
        </div>
      )}

      {(fieldError || error) && (
        <p className="text-red-400 text-sm">{fieldError || error}</p>
      )}

      <div className="flex items-center gap-3 pt-2">
        <button
          type="submit"
          disabled={loading}
          className="mc-button-primary px-4 py-2 text-sm disabled:opacity-50"
        >
          {loading ? 'Saving…' : isEdit ? 'Update Route' : 'Save Route'}
        </button>
        <button
          type="button"
          onClick={onCancel}
          className="text-slate-400 hover:text-slate-200 text-sm transition"
        >
          Cancel
        </button>
      </div>
    </form>
  );
};
