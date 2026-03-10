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

// Applications API

export interface Application {
  id: string;
  name: string;
  upstream_url: string;
  hostname: string;
  enabled: boolean;
  created_at: string;
  updated_at: string;
}

export interface CreateApplicationInput {
  name: string;
  upstream_url: string;
  hostname: string;
}

export interface UpdateApplicationInput {
  name?: string;
  upstream_url?: string;
  hostname?: string;
}

export const getApplications = async (): Promise<Application[]> => {
  const response = await api.get<Application[]>('/api/applications');
  return response.data;
};

export const createApplication = async (data: CreateApplicationInput): Promise<Application> => {
  const response = await api.post<Application>('/api/applications', data);
  return response.data;
};

export const updateApplication = async (id: string, data: UpdateApplicationInput): Promise<Application> => {
  const response = await api.put<Application>(`/api/applications/${id}`, data);
  return response.data;
};

// Routes API

export interface Route {
  id: string;
  application_id: string;
  host: string;
  path_prefix: string;
  access_mode: 'public' | 'login_required';
  enabled: boolean;
  created_at: string;
  updated_at: string;
}

export interface CreateRouteInput {
  host: string;
  path_prefix?: string;
  access_mode: 'public' | 'login_required';
}

export interface UpdateRouteInput {
  host?: string;
  path_prefix?: string;
  access_mode?: 'public' | 'login_required';
  enabled?: boolean;
}

export const getRoutes = async (appId: string): Promise<Route[]> => {
  const response = await api.get<Route[]>(`/api/applications/${appId}/routes`);
  return response.data;
};

export const createRoute = async (appId: string, data: CreateRouteInput): Promise<Route> => {
  const response = await api.post<Route>(`/api/applications/${appId}/routes`, data);
  return response.data;
};

export const updateRoute = async (id: string, data: UpdateRouteInput): Promise<Route> => {
  const response = await api.put<Route>(`/api/routes/${id}`, data);
  return response.data;
};

export default api;
