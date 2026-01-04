import { useState, useEffect } from 'react';
import { Calendar, Tag, ArrowRight } from 'lucide-react';
import { api } from '../../services/api';
import { markdownToPlainText } from '../../utils/markdown';
import type { PostResponse } from '../../types';

// Generate color for category labels (GitHub-style)
const getCategoryColor = (category: string): { bg: string; text: string } => {
  // Simple hash function to generate consistent colors
  let hash = 0;
  for (let i = 0; i < category.length; i++) {
    hash = category.charCodeAt(i) + ((hash << 5) - hash);
  }

  // Predefined color palette (GitHub-style)
  const colors = [
    { bg: 'bg-blue-100', text: 'text-blue-700' },
    { bg: 'bg-green-100', text: 'text-green-700' },
    { bg: 'bg-yellow-100', text: 'text-yellow-700' },
    { bg: 'bg-red-100', text: 'text-red-700' },
    { bg: 'bg-purple-100', text: 'text-purple-700' },
    { bg: 'bg-pink-100', text: 'text-pink-700' },
    { bg: 'bg-indigo-100', text: 'text-indigo-700' },
    { bg: 'bg-gray-100', text: 'text-gray-700' },
    { bg: 'bg-emerald-100', text: 'text-emerald-700' },
    { bg: 'bg-cyan-100', text: 'text-cyan-700' },
    { bg: 'bg-orange-100', text: 'text-orange-700' },
    { bg: 'bg-teal-100', text: 'text-teal-700' },
  ];

  // Use absolute value and modulo to get consistent color
  const index = Math.abs(hash) % colors.length;
  return colors[index];
};

interface PostListProps {
  onPostSelect?: (post: PostResponse) => void;
}

export function PostList({ onPostSelect }: PostListProps) {
  const [posts, setPosts] = useState<PostResponse[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState('');

  useEffect(() => {
    loadPosts();
  }, []);

  const loadPosts = async () => {
    try {
      setLoading(true);
      const data = await api.getPosts();
      setPosts(data);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load posts');
    } finally {
      setLoading(false);
    }
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center py-12">
        <div className="text-slate-400">Loading...</div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="bg-red-50 border border-red-200 text-red-700 px-4 py-3 rounded-lg">
        {error}
      </div>
    );
  }

  if (posts.length === 0) {
    return (
      <div className="text-center py-12 text-slate-500">
        <p>No posts yet</p>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {posts.map((post) => (
        <article
          key={post.id}
          className="bg-white rounded-xl border border-slate-200 p-6 hover:shadow-lg transition-shadow cursor-pointer"
          onClick={() => onPostSelect?.(post)}
        >
          <div className="flex items-start justify-between gap-4">
            <div className="flex-1 min-w-0">
              <div className="flex items-center gap-3 mb-3">
                <span className={`inline-flex items-center gap-1 px-3 py-1 text-sm font-medium rounded-full ${getCategoryColor(post.category).bg} ${getCategoryColor(post.category).text}`}>
                  <Tag size={14} />
                  {post.category}
                </span>
                <span className="inline-flex items-center gap-1 text-sm text-slate-500">
                  <Calendar size={14} />
                  {new Date(post.created_at).toLocaleDateString('en-US', {
                    year: 'numeric',
                    month: 'long',
                    day: 'numeric',
                  })}
                </span>
              </div>
              <h2 className="text-2xl font-bold text-slate-900 mb-2 hover:text-orange-600 transition-colors">
                {post.title}
              </h2>
              <p className="text-slate-600 line-clamp-2 mb-4">
                {markdownToPlainText(post.content).substring(0, 150)}
                {markdownToPlainText(post.content).length > 150 && '...'}
              </p>
              <div className="flex items-center gap-2 text-sm text-orange-600 font-medium">
                Read more
                <ArrowRight size={16} />
              </div>
            </div>
          </div>
        </article>
      ))}
    </div>
  );
}

