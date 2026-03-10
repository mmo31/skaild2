import React, { createContext, useContext, useState, useEffect } from 'react';
import { getCurrentAdmin, login as apiLogin, logout as apiLogout, AdminResponse } from '../services/api';

interface AuthContextType {
  admin: AdminResponse | null;
  loading: boolean;
  login: (email: string, password: string) => Promise<void>;
  logout: () => Promise<void>;
  refreshAdmin: () => Promise<void>;
}

const AuthContext = createContext<AuthContextType | undefined>(undefined);

export const AuthProvider: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  const [admin, setAdmin] = useState<AdminResponse | null>(null);
  const [loading, setLoading] = useState(true);

  const refreshAdmin = async () => {
    try {
      const data = await getCurrentAdmin();
      setAdmin(data);
    } catch (error) {
      setAdmin(null);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    refreshAdmin();
  }, []);

  const login = async (email: string, password: string) => {
    const data = await apiLogin({ email, password });
    setAdmin(data);
  };

  const logout = async () => {
    await apiLogout();
    setAdmin(null);
  };

  return (
    <AuthContext.Provider value={{ admin, loading, login, logout, refreshAdmin }}>
      {children}
    </AuthContext.Provider>
  );
};

export const useAuth = () => {
  const context = useContext(AuthContext);
  if (context === undefined) {
    throw new Error('useAuth must be used within an AuthProvider');
  }
  return context;
};
