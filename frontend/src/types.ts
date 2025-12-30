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

declare global {
  interface Window {
    __INITIAL_DATA__?: PageData;
  }
}

