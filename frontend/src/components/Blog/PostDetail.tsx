import { useState, useEffect } from 'react';
import { Calendar, Tag, ArrowLeft } from 'lucide-react';
import { api } from '../../services/api';
import type { PostResponse } from '../../types';

interface PostDetailProps {
  postId: string;
  onBack?: () => void;
}

export function PostDetail({ postId, onBack }: PostDetailProps) {
  const [post, setPost] = useState<PostResponse | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState('');

  useEffect(() => {
    loadPost();
  }, [postId]);

  const loadPost = async () => {
    try {
      setLoading(true);
      const data = await api.getPost(postId);
      setPost(data);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load post');
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

  if (!post) {
    return (
      <div className="text-center py-12 text-slate-500">
        <p>Post not found</p>
      </div>
    );
  }

  return (
    <article className="bg-white rounded-xl border border-slate-200 p-8">
      {onBack && (
        <button
          onClick={onBack}
          className="inline-flex items-center gap-2 text-slate-600 hover:text-orange-600 mb-6 transition-colors"
        >
          <ArrowLeft size={20} />
          Back to List
        </button>
      )}

      <header className="mb-8">
        <div className="flex items-center gap-3 mb-4">
          <span className="inline-flex items-center gap-1 px-3 py-1 bg-orange-100 text-orange-700 text-sm font-medium rounded-full">
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
        <h1 className="text-4xl font-extrabold text-slate-900 leading-tight mb-4">
          {post.title}
        </h1>
      </header>

      <div
        className="prose prose-slate prose-lg max-w-none"
        dangerouslySetInnerHTML={{ __html: post.content }}
      />
    </article>
  );
}

