import axios from 'axios';

const API_BASE_URL = import.meta.env.VITE_API_URL || 'http://localhost:8080';

const api = axios.create({
  baseURL: API_BASE_URL,
  withCredentials: true, // Important for session cookies
  headers: {
    'Content-Type': 'application/json',
  },
});

export interface SetupStatusResponse {
  setup_complete: boolean;
}

export interface CreateAdminRequest {
  name: string;
  email: string;
  password: string;
}

export interface AdminResponse {
  id: string;
  name: string;
  email: string;
  created_at?: string;
}

export interface LoginRequest {
  email: string;
  password: string;
}

export interface LogoutResponse {
  success: boolean;
}

export interface ErrorResponse {
  error: string;
}

// Setup API
export const getSetupStatus = async (): Promise<SetupStatusResponse> => {
  const response = await api.get<SetupStatusResponse>('/api/setup/status');
  return response.data;
};

export const createFirstAdmin = async (data: CreateAdminRequest): Promise<AdminResponse> => {
  const response = await api.post<AdminResponse>('/api/setup/create-admin', data);
  return response.data;
};

// Auth API
export const login = async (data: LoginRequest): Promise<AdminResponse> => {
  const response = await api.post<AdminResponse>('/api/auth/login', data);
  return response.data;
};

export const logout = async (): Promise<LogoutResponse> => {
  const response = await api.post<LogoutResponse>('/api/auth/logout');
  return response.data;
};

export const getCurrentAdmin = async (): Promise<AdminResponse> => {
  const response = await api.get<AdminResponse>('/api/auth/me');
  return response.data;
};

export default api;
