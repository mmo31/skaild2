import React from 'react';
import { Navigate } from 'react-router-dom';
import { useAuth } from '../hooks/useAuth';

export const ProtectedRoute: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  const { admin, loading } = useAuth();

  if (loading) {
    return (
      <div className="min-h-screen flex items-center justify-center bg-mc-ocean">
        <div className="mc-surface p-8">
          <p className="text-slate-700">Loading...</p>
        </div>
      </div>
    );
  }

  if (!admin) {
    return <Navigate to="/login" replace />;
  }

  return <>{children}</>;
};
