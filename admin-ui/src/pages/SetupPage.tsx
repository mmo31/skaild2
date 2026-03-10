import React, { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { createFirstAdmin } from '../services/api';

export const SetupPage: React.FC = () => {
  const navigate = useNavigate();
  const [name, setName] = useState('');
  const [email, setEmail] = useState('');
  const [password, setPassword] = useState('');
  const [confirmPassword, setConfirmPassword] = useState('');
  const [error, setError] = useState('');
  const [loading, setLoading] = useState(false);
  const [success, setSuccess] = useState(false);

  const getPasswordStrength = (pwd: string): { strength: string; color: string; percentage: number } => {
    let score = 0;
    if (pwd.length >= 12) score++;
    if (pwd.length >= 16) score++;
    if (/[a-z]/.test(pwd)) score++;
    if (/[A-Z]/.test(pwd)) score++;
    if (/[0-9]/.test(pwd)) score++;
    if (/[^a-zA-Z0-9]/.test(pwd)) score++;

    if (score <= 2) return { strength: 'Weak', color: 'bg-red-500', percentage: 33 };
    if (score <= 4) return { strength: 'Medium', color: 'bg-yellow-500', percentage: 66 };
    return { strength: 'Strong', color: 'bg-green-500', percentage: 100 };
  };

  const passwordStrength = getPasswordStrength(password);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError('');

    if (password !== confirmPassword) {
      setError('Passwords do not match');
      return;
    }

    setLoading(true);
    try {
      await createFirstAdmin({ name, email, password });
      setSuccess(true);
      setTimeout(() => {
        navigate('/login');
      }, 2000);
    } catch (err: any) {
      setError(err.response?.data?.error || 'Failed to create admin account');
    } finally {
      setLoading(false);
    }
  };

  if (success) {
    return (
      <div className="min-h-screen flex items-center justify-center bg-cover bg-center" 
           style={{ backgroundImage: 'url(/skaild-background.png)' }}>
        <div className="mc-surface p-8 max-w-md w-full mx-4 space-y-4">
          <div className="text-center">
            <div className="text-4xl mb-4">✅</div>
            <h2 className="text-2xl font-semibold text-slate-900 mb-2">Admin Account Created!</h2>
            <p className="text-slate-600">Redirecting to login...</p>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="min-h-screen flex items-center justify-center bg-cover bg-center" 
         style={{ backgroundImage: 'url(/skaild-background.png)' }}>
      <div className="mc-surface p-8 max-w-md w-full mx-4">
        <div className="text-center mb-6">
          <h1 className="text-3xl font-bold text-slate-900 mb-2">Welcome to skaild2</h1>
          <p className="text-slate-600">Create your admin account to get started</p>
        </div>

        <form onSubmit={handleSubmit} className="space-y-4">
          {error && (
            <div className="bg-red-500/10 border border-red-500/50 text-red-600 px-4 py-3 rounded-lg">
              {error}
            </div>
          )}

          <div>
            <label htmlFor="name" className="block text-sm font-medium text-slate-700 mb-1">
              Name
            </label>
            <input
              id="name"
              type="text"
              required
              value={name}
              onChange={(e) => setName(e.target.value)}
              className="w-full px-4 py-2 bg-white/60 backdrop-blur-sm border border-slate-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-mc-teal focus:border-transparent text-slate-900"
              placeholder="Your name"
            />
          </div>

          <div>
            <label htmlFor="email" className="block text-sm font-medium text-slate-700 mb-1">
              Email
            </label>
            <input
              id="email"
              type="email"
              required
              value={email}
              onChange={(e) => setEmail(e.target.value)}
              className="w-full px-4 py-2 bg-white/60 backdrop-blur-sm border border-slate-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-mc-teal focus:border-transparent text-slate-900"
              placeholder="admin@example.com"
            />
          </div>

          <div>
            <label htmlFor="password" className="block text-sm font-medium text-slate-700 mb-1">
              Password
            </label>
            <input
              id="password"
              type="password"
              required
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              className="w-full px-4 py-2 bg-white/60 backdrop-blur-sm border border-slate-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-mc-teal focus:border-transparent text-slate-900"
              placeholder="••••••••••••"
            />
            {password && (
              <div className="mt-2">
                <div className="flex items-center justify-between text-xs text-slate-600 mb-1">
                  <span>Password strength:</span>
                  <span className={`font-medium ${
                    passwordStrength.strength === 'Weak' ? 'text-red-600' :
                    passwordStrength.strength === 'Medium' ? 'text-yellow-600' :
                    'text-green-600'
                  }`}>
                    {passwordStrength.strength}
                  </span>
                </div>
                <div className="h-2 bg-slate-200 rounded-full overflow-hidden">
                  <div 
                    className={`h-full transition-all ${passwordStrength.color}`}
                    style={{ width: `${passwordStrength.percentage}%` }}
                  />
                </div>
              </div>
            )}
            <div className="mt-2 text-xs text-slate-600 space-y-1">
              <p>Password must contain:</p>
              <ul className="list-disc list-inside space-y-0.5 ml-2">
                <li className={password.length >= 12 ? 'text-green-600' : ''}>At least 12 characters</li>
                <li className={/[A-Z]/.test(password) ? 'text-green-600' : ''}>One uppercase letter</li>
                <li className={/[a-z]/.test(password) ? 'text-green-600' : ''}>One lowercase letter</li>
                <li className={/[0-9]/.test(password) ? 'text-green-600' : ''}>One number</li>
                <li className={/[^a-zA-Z0-9]/.test(password) ? 'text-green-600' : ''}>One special character</li>
              </ul>
            </div>
          </div>

          <div>
            <label htmlFor="confirmPassword" className="block text-sm font-medium text-slate-700 mb-1">
              Confirm Password
            </label>
            <input
              id="confirmPassword"
              type="password"
              required
              value={confirmPassword}
              onChange={(e) => setConfirmPassword(e.target.value)}
              className="w-full px-4 py-2 bg-white/60 backdrop-blur-sm border border-slate-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-mc-teal focus:border-transparent text-slate-900"
              placeholder="••••••••••••"
            />
            {confirmPassword && password !== confirmPassword && (
              <p className="mt-1 text-xs text-red-600">Passwords do not match</p>
            )}
          </div>

          <button
            type="submit"
            disabled={loading || password !== confirmPassword}
            className="w-full mc-button-primary py-3 font-medium disabled:opacity-50 disabled:cursor-not-allowed"
          >
            {loading ? 'Creating Account...' : 'Create Admin Account'}
          </button>
        </form>
      </div>
    </div>
  );
};
