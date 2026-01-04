import { useState, useEffect } from 'react';
import { Calendar, Tag, ArrowLeft, Edit, History } from 'lucide-react';
import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';
import rehypeRaw from 'rehype-raw';
import rehypeHighlight from 'rehype-highlight';
import { api, isAuthenticated } from '../../services/api';
import { normalizeImageUrl } from '../../utils/url';
import { PostVersions } from './PostVersions';
import type { PostResponse } from '../../types';
import 'highlight.js/styles/github.css';

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

interface PostDetailProps {
  postId: string;
  onBack?: () => void;
  onEdit?: (post: PostResponse) => void;
}

export function PostDetail({ postId, onBack, onEdit }: PostDetailProps) {
  const [post, setPost] = useState<PostResponse | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState('');
  const [isAuthor, setIsAuthor] = useState(false);
  const [showVersions, setShowVersions] = useState(false);

  useEffect(() => {
    loadPost();
    checkIsAuthor();
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

  const checkIsAuthor = async () => {
    if (!isAuthenticated()) {
      setIsAuthor(false);
      return;
    }

    try {
      const user = await api.getCurrentUser();
      if (post && user.id === post.author_id) {
        setIsAuthor(true);
      }
    } catch (err) {
      setIsAuthor(false);
    }
  };

  useEffect(() => {
    if (post) {
      checkIsAuthor();
    }
  }, [post]);

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

  const handleEdit = () => {
    if (post && onEdit) {
      onEdit(post);
    }
  };

  const handleShowVersions = () => {
    setShowVersions(true);
  };

  const handleVersionsClose = () => {
    setShowVersions(false);
  };

  const handleVersionRestored = () => {
    setShowVersions(false);
    loadPost(); // Reload the post
  };

  return (
    <article className="bg-white rounded-xl border border-slate-200 p-8">
      <div className="flex items-center justify-between mb-6">
        {onBack && (
          <button
            onClick={onBack}
            className="inline-flex items-center gap-2 text-slate-600 hover:text-orange-600 transition-colors"
          >
            <ArrowLeft size={20} />
            Back to List
          </button>
        )}
        {isAuthor && (
          <div className="flex items-center gap-2">
            <button
              onClick={handleShowVersions}
              className="inline-flex items-center gap-2 px-4 py-2 text-sm font-medium text-slate-700 bg-slate-100 hover:bg-slate-200 rounded-lg transition-colors"
            >
              <History size={16} />
              History
            </button>
            <button
              onClick={handleEdit}
              className="inline-flex items-center gap-2 px-4 py-2 text-sm font-medium text-white bg-orange-500 hover:bg-orange-600 rounded-lg transition-colors"
            >
              <Edit size={16} />
              Edit
            </button>
          </div>
        )}
      </div>

      <header className="mb-8">
        <div className="flex items-center gap-3 mb-4">
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
        <h1 className="text-4xl font-extrabold text-slate-900 leading-tight mb-4">
          {post.title}
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
          {post.content}
        </ReactMarkdown>
      </div>

      {/* Version history modal */}
      {showVersions && (
        <PostVersions
          postId={postId}
          onClose={handleVersionsClose}
          onRestore={handleVersionRestored}
        />
      )}
    </article>
  );
}

