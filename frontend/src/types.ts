export interface Post {
  id: string;
  title: string;
  content: string;
  category: string;
  author_id: number;
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

export interface Post {
  id: string;
  title: string;
  content: string;
  category: string;
  author_id: number;
  created_at: string;
  updated_at: string;
}

export interface PageData {
  posts: Post[];
  current_post: Post | null;
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

export interface User {
  id: number;
  username: string;
  email: string;
  role: 'Root' | 'Admin' | 'User';
  created_at: string;
  updated_at: string;
}

export interface CreatePostRequest {
  title: string;
  category?: string;
  content: string;
  author_id: number;
}

export interface PostVersionResponse {
  id: string;
  post_id: string;
  title: string;
  content: string;
  category: string;
  version_number: number;
  created_at: string;
  created_by: number;
  change_note: string | null;
}

export interface PostDraftResponse {
  id: string;
  post_id: string | null;
  title: string;
  content: string;
  category: string;
  author_id: number;
  created_at: string;
  updated_at: string;
}

export interface SaveDraftRequest {
  post_id?: number | null;
  title: string;
  content: string;
  category: string;
}

export interface UpdatePostRequest {
  title?: string;
  content?: string;
  category?: string;
  create_version?: boolean;
  change_note?: string;
}

declare global {
  interface Window {
    __INITIAL_DATA__?: PageData;
  }
}

