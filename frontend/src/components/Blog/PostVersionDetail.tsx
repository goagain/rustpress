import { useState, useEffect } from 'react';
import { Calendar, Tag, ArrowLeft, Clock, Archive } from 'lucide-react';
import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';
import rehypeRaw from 'rehype-raw';
import rehypeHighlight from 'rehype-highlight';
import { api } from '../../services/api';
import { normalizeImageUrl } from '../../utils/url';
import type { PostVersionResponse } from '../../types';
import 'highlight.js/styles/github.css';

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

      <div className="prose prose-slate prose-lg max-w-none 
        prose-headings:font-bold 
        prose-a:text-orange-600 prose-a:no-underline hover:prose-a:underline 
        prose-code:bg-slate-100 prose-code:px-1 prose-code:py-0.5 prose-code:rounded prose-code:text-sm 
        prose-code:before:content-none prose-code:after:content-none
        prose-pre:bg-slate-900 prose-pre:text-slate-100 prose-pre:border prose-pre:border-slate-700 prose-pre:rounded-lg
        prose-blockquote:border-l-orange-500 prose-blockquote:bg-slate-50
        prose-img:rounded-lg prose-img:shadow-md
        prose-table:border prose-table:border-slate-300 prose-table:w-full
        prose-th:bg-slate-100 prose-th:font-semibold prose-th:p-3 prose-th:text-left
        prose-td:border prose-td:border-slate-200 prose-td:p-3
        prose-ul:list-disc prose-ul:pl-6
        prose-ol:list-decimal prose-ol:pl-6
        prose-li:my-1
        prose-hr:border-slate-300
        prose-strong:font-bold prose-strong:text-slate-900
        prose-em:italic
        prose-del:line-through
        prose-mark:bg-yellow-200">
        <ReactMarkdown
          remarkPlugins={[remarkGfm]}
          rehypePlugins={[rehypeRaw, rehypeHighlight]}
          components={{
            // Handle images: convert server URLs to relative paths
            img: ({node, className, src, alt, ...props}: any) => {
              const normalizedSrc = normalizeImageUrl(src || '');
              return (
                <img
                  src={normalizedSrc}
                  alt={alt}
                  className={`${className || ''} rounded-lg shadow-md`}
                  {...props}
                />
              );
            },
            // Ensure proper rendering of all markdown elements
            ul: ({node, className, ...props}) => (
              <ul className={`${className || ''} list-disc pl-6`} {...props} />
            ),
            ol: ({node, className, ...props}) => (
              <ol className={`${className || ''} list-decimal pl-6`} {...props} />
            ),
            li: ({node, className, ...props}) => (
              <li className={`${className || ''} my-1`} {...props} />
            ),
            table: ({node, className, ...props}) => (
              <table className={`${className || ''} border border-slate-300 w-full`} {...props} />
            ),
            thead: ({node, className, ...props}) => (
              <thead className={`${className || ''} bg-slate-50`} {...props} />
            ),
            tbody: ({node, className, ...props}) => (
              <tbody className={className || ''} {...props} />
            ),
            tr: ({node, className, ...props}) => (
              <tr className={`${className || ''} border-b border-slate-200`} {...props} />
            ),
            th: ({node, className, ...props}) => (
              <th className={`${className || ''} bg-slate-100 font-semibold p-3 text-left border border-slate-300`} {...props} />
            ),
            td: ({node, className, ...props}) => (
              <td className={`${className || ''} p-3 border border-slate-200`} {...props} />
            ),
            input: ({node, className, ...props}: any) => {
              // Task list checkboxes
              if (props.type === 'checkbox') {
                return (
                  <input
                    type="checkbox"
                    className={`${className || ''} mr-2`}
                    disabled
                    {...props}
                  />
                );
              }
              return <input className={className || ''} {...props} />;
            },
          }}
        >
          {version.content}
        </ReactMarkdown>
      </div>
    </article>
  );
}
