import { useState, useEffect } from 'react';
import { Calendar, Tag, ArrowLeft, Clock, Archive } from 'lucide-react';
import { api } from '../../services/api';
import { MarkdownRenderer } from '../Markdown/MarkdownRenderer';
import type { PostVersionResponse } from '../../types';

interface PostVersionDetailProps {
  postId: string;
  versionId: string;
  onBack: () => void;
}

export function PostVersionDetail({ postId, versionId, onBack }: PostVersionDetailProps) {
  const [version, setVersion] = useState<PostVersionResponse | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState('');

  useEffect(() => {
    loadVersion();
  }, [postId, versionId]);

  const loadVersion = async () => {
    try {
      setLoading(true);
      const data = await api.getPostVersion(postId, versionId);
      setVersion(data);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load version');
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

  if (!version) {
    return (
      <div className="text-center py-12 text-slate-500">
        <p>Version not found</p>
      </div>
    );
  }

  return (
    <article className="bg-white rounded-xl border border-slate-200 p-8">
      {/* Historical version banner */}
      <div className="mb-6 p-4 bg-amber-50 border border-amber-200 rounded-lg">
        <div className="flex items-center gap-2 text-amber-800">
          <Archive size={20} />
          <span className="font-semibold">Historical Version</span>
        </div>
        <p className="text-sm text-amber-700 mt-1">
          You are viewing a historical version of this post. This is not the current version.
        </p>
      </div>

      <div className="flex items-center justify-between mb-6">
        {onBack && (
          <button
            onClick={onBack}
            className="inline-flex items-center gap-2 text-slate-600 hover:text-orange-600 transition-colors"
          >
            <ArrowLeft size={20} />
            Back to Versions
          </button>
        )}
      </div>

      <header className="mb-8">
        <div className="flex items-center gap-3 mb-4 flex-wrap">
          <span className="inline-flex items-center gap-1 px-3 py-1 bg-orange-100 text-orange-700 text-sm font-medium rounded-full">
            <Tag size={14} />
            {version.category}
          </span>
          <span className="inline-flex items-center gap-1 px-3 py-1 bg-slate-100 text-slate-700 text-sm font-medium rounded-full">
            <span className="font-semibold">Version {version.version_number}</span>
          </span>
          <span className="inline-flex items-center gap-1 text-sm text-slate-500">
            <Calendar size={14} />
            {new Date(version.created_at).toLocaleDateString('en-US', {
              year: 'numeric',
              month: 'long',
              day: 'numeric',
            })}
          </span>
          <span className="inline-flex items-center gap-1 text-sm text-slate-500">
            <Clock size={14} />
            {new Date(version.created_at).toLocaleTimeString('en-US', {
              hour: '2-digit',
              minute: '2-digit',
            })}
          </span>
        </div>
        {version.change_note && (
          <div className="mb-4 p-3 bg-slate-50 border border-slate-200 rounded-lg">
            <p className="text-sm text-slate-600">
              <span className="font-semibold">Change Note:</span> {version.change_note}
            </p>
          </div>
        )}
        <h1 className="text-4xl font-extrabold text-slate-900 leading-tight mb-4">
          {version.title}
        </h1>
      </header>

      <div className="markdown-content">
        <MarkdownRenderer content={version.content} />
      </div>
    </article>
  );
}
