// Admin frontend types

export interface User {
  id: number;
  username: string;
  email: string;
  role: 'Root' | 'Admin' | 'User';
  created_at: string;
  updated_at: string;
}

export interface PostResponse {
  id: string;
  title: string;
  content: string;
  category: string;
  author_id: number;
  created_at: string;
  updated_at: string;
}

export interface LoginRequest {
  username: string;
  password: string;
}

export interface LoginResponse {
  access_token: string;
  refresh_token: string;
  token_type: string;
  expires_in: number;
}

// Admin types
export interface AdminSettingsResponse {
  allow_external_registration: boolean;
  maintenance_mode: boolean;
}

export interface AdminSettingsUpdateRequest {
  allow_external_registration?: boolean;
  maintenance_mode?: boolean;
}

export interface AdminUserListResponse {
  user: User;
  is_banned: boolean;
}

export interface AdminPostListResponse {
  post: PostResponse;
}

export interface AdminBanUserRequest {
  banned: boolean;
}

export interface AdminResetPasswordRequest {
  new_password: string;
}

export interface AdminResetPasswordResponse {
  success: boolean;
  message: string;
}

export interface AdminPluginListResponse {
  id: number;
  name: string;
  description: string | null;
  version: string;
  enabled: boolean;
  config: any;
  created_at: string;
  updated_at: string;
}

export interface AdminPluginUpdateRequest {
  enabled?: boolean;
  config?: any;
}
