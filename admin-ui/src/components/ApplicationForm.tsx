import React, { useState } from 'react';
import { Application, CreateApplicationInput, UpdateApplicationInput } from '../services/api';

interface ApplicationFormProps {
  initial?: Application;
  onSubmit: (data: CreateApplicationInput | UpdateApplicationInput) => Promise<void>;
  onCancel: () => void;
  loading: boolean;
  error: string;
}

export const ApplicationForm: React.FC<ApplicationFormProps> = ({
  initial,
  onSubmit,
  onCancel,
  loading,
  error,
}) => {
  const [name, setName] = useState(initial?.name ?? '');
  const [upstreamUrl, setUpstreamUrl] = useState(initial?.upstream_url ?? '');
  const [hostname, setHostname] = useState(initial?.hostname ?? '');

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    await onSubmit({ name, upstream_url: upstreamUrl, hostname });
  };

  return (
    <form onSubmit={handleSubmit} className="space-y-4">
      {error && (
        <div className="bg-red-500/10 border border-red-500/50 text-red-600 px-4 py-3 rounded-lg text-sm">
          {error}
        </div>
      )}

      <div>
        <label htmlFor="app-name" className="block text-sm font-medium text-slate-300 mb-1">
          Name <span className="text-red-400">*</span>
        </label>
        <input
          id="app-name"
          type="text"
          required
          value={name}
          onChange={(e) => setName(e.target.value)}
          placeholder="My Home Assistant"
          className="w-full px-4 py-2 bg-slate-800/60 border border-slate-600 rounded-lg focus:outline-none focus:ring-2 focus:ring-mc-teal text-slate-100 placeholder-slate-500"
        />
      </div>

      <div>
        <label htmlFor="app-upstream" className="block text-sm font-medium text-slate-300 mb-1">
          Upstream URL <span className="text-red-400">*</span>
        </label>
        <input
          id="app-upstream"
          type="url"
          required
          value={upstreamUrl}
          onChange={(e) => setUpstreamUrl(e.target.value)}
          placeholder="http://192.168.1.10:8123"
          className="w-full px-4 py-2 bg-slate-800/60 border border-slate-600 rounded-lg focus:outline-none focus:ring-2 focus:ring-mc-teal text-slate-100 placeholder-slate-500"
        />
      </div>

      <div>
        <label htmlFor="app-hostname" className="block text-sm font-medium text-slate-300 mb-1">
          Hostname <span className="text-red-400">*</span>
        </label>
        <input
          id="app-hostname"
          type="text"
          required
          value={hostname}
          onChange={(e) => setHostname(e.target.value)}
          placeholder="homeassistant.yourdomain.com"
          className="w-full px-4 py-2 bg-slate-800/60 border border-slate-600 rounded-lg focus:outline-none focus:ring-2 focus:ring-mc-teal text-slate-100 placeholder-slate-500"
        />
      </div>

      <div className="flex gap-3 pt-2">
        <button
          type="submit"
          disabled={loading}
          className="mc-button-primary px-5 py-2 text-sm disabled:opacity-60"
        >
          {loading ? 'Saving…' : 'Save Application'}
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
