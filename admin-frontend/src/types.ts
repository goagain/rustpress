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
export interface SettingItem {
  key: string;
  value: any;
  label: string;
  description?: string;
  input_type: string;
}

export interface SettingsTab {
  id: string;
  label: string;
  description?: string;
  items: SettingItem[];
}

export interface AdminSettingsTabsResponse {
  tabs: SettingsTab[];
}

export interface AdminSettingsUpdateRequest {
  settings: Record<string, any>;
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
  status: string; // "enabled", "disabled", "pending_review"
  config: any;
  manifest: any;
  created_at: string;
  updated_at: string;
}

export interface AdminPluginUpdateRequest {
  enabled?: boolean;
  config?: any;
}

export interface PluginPermissionInfo {
  permission: string;
  is_granted: boolean;
  permission_type: 'required' | 'optional';
  description?: string;
}

export interface PluginPermissionsResponse {
  plugin_id: string;
  permissions: PluginPermissionInfo[];
}

export interface UpdatePluginPermissionsRequest {
  permissions: Record<string, boolean>;
}

export interface PluginInstallRequest {
  rpk_data: string;
  permission_grants: Record<string, boolean>;
}

export interface AdminPluginEnableResponse {
  plugin: AdminPluginListResponse;
  new_permissions: string[];
  requires_permission_review: boolean;
}

export interface AdminOpenAITestResponse {
  success: boolean;
  message: string;
}

// OpenAI types
export interface OpenAIApiKeyResponse {
  id: number;
  name: string;
  api_key_masked: string;
  endpoint: string | null;
  is_default: boolean;
  default_model: string | null;
  created_at: string;
  updated_at: string;
}

export interface CreateOpenAIApiKeyRequest {
  name: string;
  api_key: string;
  endpoint?: string;
  is_default?: boolean;
  default_model?: string;
}

export interface UpdateOpenAIApiKeyRequest {
  name?: string;
  api_key?: string;
  endpoint?: string;
  is_default?: boolean;
  default_model?: string;
}

export interface TestOpenAIApiKeyResponse {
  success: boolean;
  message: string;
  response_content?: string;
  model_used?: string;
  token_usage?: TestTokenUsage;
}

export interface TestTokenUsage {
  prompt_tokens: number;
  completion_tokens: number;
  total_tokens: number;
}

export interface OpenAIModel {
  id: string;
  object: string;
  created: number;
  owned_by: string;
}

export interface ListOpenAIModelsResponse {
  models: OpenAIModel[];
  default_model: string | null;
}

export interface SetDefaultModelRequest {
  model_id: string;
}
