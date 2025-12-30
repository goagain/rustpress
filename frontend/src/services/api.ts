import type { PostResponse, LoginRequest, LoginResponse, User } from '../types';

const API_BASE_URL = '/api';

// Get stored token
const getAccessToken = (): string | null => {
  return localStorage.getItem('access_token');
};

const getRefreshToken = (): string | null => {
  return localStorage.getItem('refresh_token');
};

// Save tokens
export const saveTokens = (tokens: LoginResponse): void => {
  localStorage.setItem('access_token', tokens.access_token);
  localStorage.setItem('refresh_token', tokens.refresh_token);
};

// Clear tokens
export const clearTokens = (): void => {
  localStorage.removeItem('access_token');
  localStorage.removeItem('refresh_token');
};

// Check if authenticated
export const isAuthenticated = (): boolean => {
  return getAccessToken() !== null;
};

// Refresh token
const refreshAccessToken = async (): Promise<string | null> => {
  const refreshToken = getRefreshToken();
  if (!refreshToken) {
    return null;
  }

  try {
    const response = await fetch(`${API_BASE_URL}/auth/refresh`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({ refresh_token: refreshToken }),
    });

    if (response.ok) {
      const data = await response.json();
      localStorage.setItem('access_token', data.access_token);
      return data.access_token;
    }
  } catch (error) {
    console.error('Failed to refresh token:', error);
  }

  return null;
};

// Authenticated fetch
const authenticatedFetch = async (
  url: string,
  options: RequestInit = {}
): Promise<Response> => {
  const token = getAccessToken();
  
  const headers = {
    'Content-Type': 'application/json',
    ...(token && { Authorization: `Bearer ${token}` }),
    ...options.headers,
  };

  let response = await fetch(url, { ...options, headers });

  // If token expired, try to refresh
  if (response.status === 401 && token) {
    const newToken = await refreshAccessToken();
    if (newToken) {
      headers.Authorization = `Bearer ${newToken}`;
      response = await fetch(url, { ...options, headers });
    } else {
      clearTokens();
      window.location.href = '/login';
    }
  }

  return response;
};

// API methods
export const api = {
  // Authentication related
  async login(credentials: LoginRequest): Promise<LoginResponse> {
    const response = await fetch(`${API_BASE_URL}/auth/login`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(credentials),
    });

    if (!response.ok) {
      throw new Error('Login failed');
    }

    return response.json();
  },

  async getCurrentUser(): Promise<User> {
    const response = await authenticatedFetch(`${API_BASE_URL}/users/me`);
    if (!response.ok) {
      throw new Error('Failed to get user information');
    }
    return response.json();
  },

  // Post related
  async getPosts(): Promise<PostResponse[]> {
    const response = await fetch(`${API_BASE_URL}/posts`);
    if (!response.ok) {
      throw new Error('Failed to get post list');
    }
    const data = await response.json();
    // Convert id from number to string
    return data.map((post: any) => ({
      ...post,
      id: String(post.id),
    }));
  },

  async getPost(id: string | number): Promise<PostResponse> {
    // Backend expects i64, so convert if string
    const postId = typeof id === 'string' ? id : String(id);
    const response = await fetch(`${API_BASE_URL}/posts/${postId}`);
    if (!response.ok) {
      throw new Error('Failed to get post');
    }
    const data = await response.json();
    // Convert id from number to string
    return {
      ...data,
      id: String(data.id),
    };
  },
};

