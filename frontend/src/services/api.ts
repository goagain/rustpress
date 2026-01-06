import type { 
  PostResponse, 
  LoginRequest, 
  LoginResponse, 
  User,
  PostVersionResponse,
  PostDraftResponse,
  SaveDraftRequest,
  UpdatePostRequest,
} from '../types';

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
    // Parse user info from JWT token
    const token = getAccessToken();
    if (!token) {
      throw new Error('Not authenticated');
    }

    try {
      const payload = JSON.parse(atob(token.split('.')[1]));
      // Note: This is a simplified version. In production, you might want to verify the token
      // and fetch full user data from the backend
      return {
        id: parseInt(payload.sub, 10),
        username: payload.username,
        email: '', // Not available in token
        role: payload.role as 'Root' | 'Admin' | 'User',
        created_at: '',
        updated_at: '',
      };
    } catch (error) {
      throw new Error('Failed to parse user information from token');
    }
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

  async createPost(post: { title: string; category?: string | null; description?: string | null; content: string; author_id: number }): Promise<PostResponse> {
    // Remove null/empty fields from the request body
    const requestBody = { ...post };
    if (requestBody.category === null || requestBody.category === '') {
      delete requestBody.category;
    }
    if (requestBody.description === null || requestBody.description === '') {
      delete requestBody.description;
    }

    const response = await authenticatedFetch(`${API_BASE_URL}/posts`, {
      method: 'POST',
      body: JSON.stringify(requestBody),
    });

    if (!response.ok) {
      const errorData = await response.json().catch(() => ({}));
      throw new Error(errorData.message || 'Failed to create post');
    }

    const data = await response.json();
    // Convert id from number to string
    return {
      ...data,
      id: String(data.id),
    };
  },

  async uploadImage(file: File): Promise<{ url: string; filename: string }> {
    const formData = new FormData();
    formData.append('image', file);

    const token = getAccessToken();
    const headers: HeadersInit = {};
    if (token) {
      headers['Authorization'] = `Bearer ${token}`;
    }

    const response = await fetch(`${API_BASE_URL}/upload/image`, {
      method: 'POST',
      body: formData,
      // Don't set Content-Type header, browser will set it with boundary for multipart/form-data
      headers,
    });

    if (!response.ok) {
      const errorData = await response.json().catch(() => ({}));
      throw new Error(errorData.message || 'Failed to upload image');
    }

    return response.json();
  },

  // Post version management
  async getPostVersions(postId: string): Promise<PostVersionResponse[]> {
    const response = await fetch(`${API_BASE_URL}/posts/${postId}/versions`);
    if (!response.ok) {
      throw new Error('Failed to get post versions');
    }
    const data = await response.json();
    return data.map((v: any) => ({
      ...v,
      id: String(v.id),
      post_id: String(v.post_id),
    }));
  },

  async getPostVersion(postId: string, versionId: string): Promise<PostVersionResponse> {
    const response = await fetch(`${API_BASE_URL}/posts/${postId}/versions/${versionId}`);
    if (!response.ok) {
      throw new Error('Failed to get post version');
    }
    const data = await response.json();
    return {
      ...data,
      id: String(data.id),
      post_id: String(data.post_id),
    };
  },

  async restorePostFromVersion(postId: string, versionId: string): Promise<PostResponse> {
    const response = await authenticatedFetch(`${API_BASE_URL}/posts/${postId}/versions/${versionId}/restore`, {
      method: 'POST',
    });

    if (!response.ok) {
      throw new Error('Failed to restore post from version');
    }

    const data = await response.json();
    return {
      ...data,
      id: String(data.id),
    };
  },

  // Post update
  async updatePost(postId: string, post: UpdatePostRequest): Promise<PostResponse> {
    const response = await authenticatedFetch(`${API_BASE_URL}/posts/${postId}`, {
      method: 'PUT',
      body: JSON.stringify(post),
    });

    if (!response.ok) {
      const errorData = await response.json().catch(() => ({}));
      throw new Error(errorData.message || 'Failed to update post');
    }

    const data = await response.json();
    return {
      ...data,
      id: String(data.id),
    };
  },

  // Draft management
  async saveDraft(draft: SaveDraftRequest): Promise<PostDraftResponse> {
    const response = await authenticatedFetch(`${API_BASE_URL}/drafts`, {
      method: 'POST',
      body: JSON.stringify(draft),
    });

    if (!response.ok) {
      const errorData = await response.json().catch(() => ({}));
      throw new Error(errorData.message || 'Failed to save draft');
    }

    const data = await response.json();
    return {
      ...data,
      id: String(data.id),
      post_id: data.post_id ? String(data.post_id) : null,
    };
  },

  async getDraft(postId?: string): Promise<PostDraftResponse | null> {
    const url = postId 
      ? `${API_BASE_URL}/drafts?post_id=${postId}`
      : `${API_BASE_URL}/drafts`;
    
    const response = await authenticatedFetch(url);
    
    if (response.status === 404) {
      return null;
    }

    if (!response.ok) {
      throw new Error('Failed to get draft');
    }

    const data = await response.json();
    return {
      ...data,
      id: String(data.id),
      post_id: data.post_id ? String(data.post_id) : null,
    };
  },

  async deleteDraft(postId?: string): Promise<void> {
    const url = postId 
      ? `${API_BASE_URL}/drafts?post_id=${postId}`
      : `${API_BASE_URL}/drafts`;
    
    const response = await authenticatedFetch(url, {
      method: 'DELETE',
    });

    if (!response.ok && response.status !== 404) {
      throw new Error('Failed to delete draft');
    }
  },
};

