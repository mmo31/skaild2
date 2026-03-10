import React, { useEffect, useState } from 'react';
import { Navigate } from 'react-router-dom';
import { getSetupStatus } from '../services/api';

export const RootRedirect: React.FC = () => {
  const [setupComplete, setSetupComplete] = useState<boolean | null>(null);

  useEffect(() => {
    const checkSetup = async () => {
      try {
        const status = await getSetupStatus();
        setSetupComplete(status.setup_complete);
      } catch (error) {
        console.error('Failed to check setup status:', error);
        setSetupComplete(true); // Default to login if check fails
      }
    };

    checkSetup();
  }, []);

  if (setupComplete === null) {
    return (
      <div className="min-h-screen flex items-center justify-center bg-mc-ocean">
        <div className="mc-surface p-8">
          <p className="text-slate-700">Loading...</p>
        </div>
      </div>
    );
  }

  return setupComplete ? <Navigate to="/login" replace /> : <Navigate to="/setup" replace />;
};
