import type {
  LoginRequest,
  LoginResponse,
  AdminSettingsTabsResponse,
  AdminSettingsUpdateRequest,
  AdminUserListResponse,
  AdminPostListResponse,
  AdminResetPasswordResponse,
  AdminPluginListResponse,
  AdminPluginUpdateRequest,
  PluginPermissionsResponse,
  UpdatePluginPermissionsRequest,
  PluginInstallRequest,
  OpenAIApiKeyResponse,
  CreateOpenAIApiKeyRequest,
  UpdateOpenAIApiKeyRequest,
  TestOpenAIApiKeyResponse,
  ListOpenAIModelsResponse,
  SetDefaultModelRequest,
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

  const headers: HeadersInit = {
    'Content-Type': 'application/json',
    ...(token && { Authorization: `Bearer ${token}` }),
    ...options.headers,
  };

  let response = await fetch(url, { ...options, headers });

  // If token expired, try to refresh
  if (response.status === 401 && token) {
    const newToken = await refreshAccessToken();
    if (newToken) {
      const newHeaders: HeadersInit = {
        ...headers,
        Authorization: `Bearer ${newToken}`,
      };
      response = await fetch(url, { ...options, headers: newHeaders });
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

  async getCurrentUser() {
    // Parse user info from JWT token
    const token = getAccessToken();
    if (!token) {
      throw new Error('Not authenticated');
    }

    try {
      const payload = JSON.parse(atob(token.split('.')[1]));
      return {
        id: parseInt(payload.sub, 10),
        username: payload.username,
        email: '',
        role: payload.role as 'Root' | 'Admin' | 'User',
        created_at: '',
        updated_at: '',
      };
    } catch (error) {
      throw new Error('Failed to parse user information from token');
    }
  },

  // Admin API methods
  async getAdminSettingsTabs(): Promise<AdminSettingsTabsResponse> {
    const response = await authenticatedFetch(`${API_BASE_URL}/admin/settings/tabs`);
    if (!response.ok) {
      throw new Error('Failed to get settings tabs');
    }
    return response.json();
  },

  async updateAdminSettings(
    settings: AdminSettingsUpdateRequest
  ): Promise<AdminSettingsTabsResponse> {
    const response = await authenticatedFetch(`${API_BASE_URL}/admin/settings`, {
      method: 'PUT',
      body: JSON.stringify(settings),
    });
    if (!response.ok) {
      throw new Error('Failed to update settings');
    }
    return response.json();
  },


  async getAdminUsers(): Promise<AdminUserListResponse[]> {
    const response = await authenticatedFetch(`${API_BASE_URL}/admin/users`);
    if (!response.ok) {
      throw new Error('Failed to get users');
    }
    return response.json();
  },

  async banUser(userId: number, banned: boolean): Promise<void> {
    const response = await authenticatedFetch(
      `${API_BASE_URL}/admin/users/${userId}/ban`,
      {
        method: 'POST',
        body: JSON.stringify({ banned }),
      }
    );
    if (!response.ok) {
      throw new Error('Failed to update user ban status');
    }
  },

  async resetUserPassword(
    userId: number,
    newPassword: string
  ): Promise<AdminResetPasswordResponse> {
    const response = await authenticatedFetch(
      `${API_BASE_URL}/admin/users/${userId}/reset-password`,
      {
        method: 'POST',
        body: JSON.stringify({ new_password: newPassword }),
      }
    );
    if (!response.ok) {
      throw new Error('Failed to reset password');
    }
    return response.json();
  },

  async getAdminPosts(): Promise<AdminPostListResponse[]> {
    const response = await authenticatedFetch(`${API_BASE_URL}/admin/posts`);
    if (!response.ok) {
      throw new Error('Failed to get posts');
    }
    const data = await response.json();
    return data.map((item: any) => ({
      ...item,
      post: {
        ...item.post,
        id: String(item.post.id),
      },
    }));
  },

  async adminDeletePost(postId: string): Promise<void> {
    const response = await authenticatedFetch(
      `${API_BASE_URL}/admin/posts/${postId}`,
      {
        method: 'DELETE',
      }
    );
    if (!response.ok) {
      throw new Error('Failed to delete post');
    }
  },

  async getAdminPlugins(): Promise<AdminPluginListResponse[]> {
    const response = await authenticatedFetch(`${API_BASE_URL}/admin/plugins`);
    if (!response.ok) {
      throw new Error('Failed to get plugins');
    }
    return response.json();
  },

  async updateAdminPlugin(
    pluginId: number,
    update: AdminPluginUpdateRequest
  ): Promise<AdminPluginEnableResponse> {
    const response = await authenticatedFetch(
      `${API_BASE_URL}/admin/plugins/${pluginId}`,
      {
        method: 'PUT',
        body: JSON.stringify(update),
      }
    );
    if (!response.ok) {
      throw new Error('Failed to update plugin');
    }
    return response.json();
  },

  async uploadPlugin(formData: FormData): Promise<void> {
    const token = getAccessToken();

    const response = await fetch(`${API_BASE_URL}/admin/plugins/upload`, {
      method: 'POST',
      headers: {
        ...(token && { Authorization: `Bearer ${token}` }),
      },
      body: formData,
    });

    if (!response.ok) {
      const errorText = await response.text();
      throw new Error(`Failed to upload plugin: ${errorText}`);
    }
  },

  async uninstallPlugin(pluginId: number): Promise<void> {
    const response = await authenticatedFetch(
      `${API_BASE_URL}/admin/plugins/${pluginId}`,
      {
        method: 'DELETE',
      }
    );

    if (!response.ok) {
      const errorText = await response.text();
      throw new Error(`Failed to uninstall plugin: ${errorText}`);
    }
  },

  async installPlugin(request: PluginInstallRequest): Promise<void> {
    const response = await authenticatedFetch(`${API_BASE_URL}/admin/plugins`, {
      method: 'POST',
      body: JSON.stringify(request),
    });
    if (!response.ok) {
      throw new Error('Failed to install plugin');
    }
  },

  async getPluginPermissions(pluginId: string): Promise<PluginPermissionsResponse> {
    const response = await authenticatedFetch(`${API_BASE_URL}/admin/plugins/${pluginId}/permissions`);
    if (!response.ok) {
      throw new Error('Failed to get plugin permissions');
    }
    return response.json();
  },

  async updatePluginPermissions(
    pluginId: string,
    request: UpdatePluginPermissionsRequest
  ): Promise<void> {
    const response = await authenticatedFetch(
      `${API_BASE_URL}/admin/plugins/${pluginId}/permissions`,
      {
        method: 'PUT',
        body: JSON.stringify(request),
      }
    );
    if (!response.ok) {
      throw new Error('Failed to update plugin permissions');
    }
  },

  async reviewPluginPermissions(
    pluginId: string,
    approvedPermissions: Record<string, boolean>
  ): Promise<void> {
    const response = await authenticatedFetch(
      `${API_BASE_URL}/admin/plugins/${pluginId}/review-permissions`,
      {
        method: 'POST',
        body: JSON.stringify({ approved_permissions: approvedPermissions }),
      }
    );
    if (!response.ok) {
      throw new Error('Failed to review plugin permissions');
    }
  },

  // OpenAI API methods
  async getOpenAIApiKeys(): Promise<OpenAIApiKeyResponse[]> {
    const response = await authenticatedFetch(`${API_BASE_URL}/admin/openai/keys`);
    if (!response.ok) {
      throw new Error('Failed to get OpenAI API keys');
    }
    return response.json();
  },

  async createOpenAIApiKey(
    request: CreateOpenAIApiKeyRequest
  ): Promise<OpenAIApiKeyResponse> {
    const response = await authenticatedFetch(`${API_BASE_URL}/admin/openai/keys`, {
      method: 'POST',
      body: JSON.stringify(request),
    });
    if (!response.ok) {
      throw new Error('Failed to create OpenAI API key');
    }
    return response.json();
  },

  async updateOpenAIApiKey(
    keyId: number,
    request: UpdateOpenAIApiKeyRequest
  ): Promise<OpenAIApiKeyResponse> {
    const response = await authenticatedFetch(
      `${API_BASE_URL}/admin/openai/keys/${keyId}`,
      {
        method: 'PUT',
        body: JSON.stringify(request),
      }
    );
    if (!response.ok) {
      throw new Error('Failed to update OpenAI API key');
    }
    return response.json();
  },

  async deleteOpenAIApiKey(keyId: number): Promise<void> {
    const response = await authenticatedFetch(
      `${API_BASE_URL}/admin/openai/keys/${keyId}`,
      {
        method: 'DELETE',
      }
    );
    if (!response.ok) {
      throw new Error('Failed to delete OpenAI API key');
    }
  },

  async testOpenAIApiKey(keyId: number): Promise<TestOpenAIApiKeyResponse> {
    const response = await authenticatedFetch(
      `${API_BASE_URL}/admin/openai/keys/${keyId}/test`,
      {
        method: 'POST',
      }
    );
    if (!response.ok) {
      throw new Error('Failed to test OpenAI API key');
    }
    return response.json();
  },

  async listOpenAIModels(keyId: number): Promise<ListOpenAIModelsResponse> {
    const response = await authenticatedFetch(
      `${API_BASE_URL}/admin/openai/keys/${keyId}/models`
    );
    if (!response.ok) {
      throw new Error('Failed to list OpenAI models');
    }
    return response.json();
  },

  async setDefaultModel(
    keyId: number,
    request: SetDefaultModelRequest
  ): Promise<OpenAIApiKeyResponse> {
    const response = await authenticatedFetch(
      `${API_BASE_URL}/admin/openai/keys/${keyId}/models`,
      {
        method: 'POST',
        body: JSON.stringify(request),
      }
    );
    if (!response.ok) {
      throw new Error('Failed to set default model');
    }
    return response.json();
  },
};
