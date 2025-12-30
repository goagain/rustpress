import { useState } from 'react';
import { Save, Eye, EyeOff, X, Loader2 } from 'lucide-react';
import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';
import rehypeRaw from 'rehype-raw';
import rehypeHighlight from 'rehype-highlight';
import { api } from '../../services/api';
import 'highlight.js/styles/github.css';

interface CreatePostProps {
  onSuccess?: () => void;
  onCancel?: () => void;
}

export function CreatePost({ onSuccess, onCancel }: CreatePostProps) {
  const [title, setTitle] = useState('');
  const [category, setCategory] = useState('');
  const [content, setContent] = useState('');
  const [showPreview, setShowPreview] = useState(false);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState('');

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    
    if (!title.trim() || !category.trim() || !content.trim()) {
      setError('Please fill in all fields');
      return;
    }

    setLoading(true);
    setError('');

    try {
      // Get user ID from JWT token
      const token = localStorage.getItem('access_token');
      if (!token) {
        setError('Not authenticated. Please login first.');
        return;
      }

      // Parse JWT token to get user ID
      const payload = JSON.parse(atob(token.split('.')[1]));
      const authorId = parseInt(payload.sub, 10);
      
      if (!authorId) {
        setError('Failed to get user information from token.');
        return;
      }
      
      await api.createPost({
        title: title.trim(),
        category: category.trim(),
        content: content.trim(),
        author_id: authorId,
      });

      // Reset form
      setTitle('');
      setCategory('');
      setContent('');
      setError('');

      // Call success callback
      onSuccess?.();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to create post');
    } finally {
      setLoading(false);
    }
  };

  const markdownComponents = {
    ul: ({node, className, ...props}: any) => (
      <ul className={`${className || ''} list-disc pl-6`} {...props} />
    ),
    ol: ({node, className, ...props}: any) => (
      <ol className={`${className || ''} list-decimal pl-6`} {...props} />
    ),
    li: ({node, className, ...props}: any) => (
      <li className={`${className || ''} my-1`} {...props} />
    ),
    table: ({node, className, ...props}: any) => (
      <table className={`${className || ''} border border-slate-300 w-full`} {...props} />
    ),
    thead: ({node, className, ...props}: any) => (
      <thead className={`${className || ''} bg-slate-50`} {...props} />
    ),
    tbody: ({node, className, ...props}: any) => (
      <tbody className={className || ''} {...props} />
    ),
    tr: ({node, className, ...props}: any) => (
      <tr className={`${className || ''} border-b border-slate-200`} {...props} />
    ),
    th: ({node, className, ...props}: any) => (
      <th className={`${className || ''} bg-slate-100 font-semibold p-3 text-left border border-slate-300`} {...props} />
    ),
    td: ({node, className, ...props}: any) => (
      <td className={`${className || ''} p-3 border border-slate-200`} {...props} />
    ),
    input: ({node, className, ...props}: any) => {
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
  };

  return (
    <div className="bg-white rounded-xl border border-slate-200 p-8">
      <div className="flex items-center justify-between mb-6">
        <h2 className="text-2xl font-bold text-slate-900">Create New Post</h2>
        {onCancel && (
          <button
            onClick={onCancel}
            className="p-2 text-slate-400 hover:text-slate-600 transition-colors"
            aria-label="Cancel"
          >
            <X size={20} />
          </button>
        )}
      </div>

      <form onSubmit={handleSubmit} className="space-y-6">
        {error && (
          <div className="bg-red-50 border border-red-200 text-red-700 px-4 py-3 rounded-lg text-sm">
            {error}
          </div>
        )}

        {/* Title */}
        <div>
          <label htmlFor="title" className="block text-sm font-medium text-slate-700 mb-2">
            Title *
          </label>
          <input
            id="title"
            type="text"
            value={title}
            onChange={(e) => setTitle(e.target.value)}
            required
            className="w-full px-4 py-2 border border-slate-300 rounded-lg focus:ring-2 focus:ring-orange-500 focus:border-orange-500 outline-none transition-colors"
            placeholder="Enter post title"
            disabled={loading}
          />
        </div>

        {/* Category */}
        <div>
          <label htmlFor="category" className="block text-sm font-medium text-slate-700 mb-2">
            Category *
          </label>
          <input
            id="category"
            type="text"
            value={category}
            onChange={(e) => setCategory(e.target.value)}
            required
            className="w-full px-4 py-2 border border-slate-300 rounded-lg focus:ring-2 focus:ring-orange-500 focus:border-orange-500 outline-none transition-colors"
            placeholder="e.g., Announcement, Tutorial, News"
            disabled={loading}
          />
        </div>

        {/* Content Editor and Preview */}
        <div>
          <div className="flex items-center justify-between mb-2">
            <label htmlFor="content" className="block text-sm font-medium text-slate-700">
              Content (Markdown) *
            </label>
            <button
              type="button"
              onClick={() => setShowPreview(!showPreview)}
              className="flex items-center gap-2 px-3 py-1.5 text-sm text-slate-600 hover:text-slate-900 hover:bg-slate-100 rounded-lg transition-colors"
            >
              {showPreview ? (
                <>
                  <EyeOff size={16} />
                  Hide Preview
                </>
              ) : (
                <>
                  <Eye size={16} />
                  Show Preview
                </>
              )}
            </button>
          </div>

          <div className={showPreview ? 'grid grid-cols-1 lg:grid-cols-2 gap-4' : ''}>
            {/* Editor */}
            <div className={showPreview ? '' : 'w-full'}>
              <textarea
                id="content"
                value={content}
                onChange={(e) => setContent(e.target.value)}
                required
                rows={showPreview ? 20 : 15}
                className="w-full px-4 py-3 border border-slate-300 rounded-lg focus:ring-2 focus:ring-orange-500 focus:border-orange-500 outline-none transition-colors font-mono text-sm resize-y"
                placeholder="Write your post content in Markdown format..."
                disabled={loading}
              />
              <p className="mt-2 text-xs text-slate-500">
                Supports GitHub Flavored Markdown: lists, tables, code blocks, etc.
              </p>
            </div>

            {/* Preview */}
            {showPreview && (
              <div className="border border-slate-300 rounded-lg p-4 bg-slate-50 overflow-y-auto max-h-[600px]">
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
                  {content ? (
                    <ReactMarkdown
                      remarkPlugins={[remarkGfm]}
                      rehypePlugins={[rehypeRaw, rehypeHighlight]}
                      components={markdownComponents}
                    >
                      {content}
                    </ReactMarkdown>
                  ) : (
                    <p className="text-slate-400 italic">Preview will appear here...</p>
                  )}
                </div>
              </div>
            )}
          </div>
        </div>

        {/* Submit Button */}
        <div className="flex items-center gap-4 pt-4 border-t border-slate-200">
          <button
            type="submit"
            disabled={loading || !title.trim() || !category.trim() || !content.trim()}
            className="flex items-center gap-2 px-6 py-2.5 bg-orange-500 hover:bg-orange-600 text-white font-semibold rounded-lg transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
          >
            {loading ? (
              <>
                <Loader2 className="animate-spin" size={18} />
                Publishing...
              </>
            ) : (
              <>
                <Save size={18} />
                Publish Post
              </>
            )}
          </button>
          {onCancel && (
            <button
              type="button"
              onClick={onCancel}
              disabled={loading}
              className="px-6 py-2.5 border border-slate-300 text-slate-700 font-semibold rounded-lg hover:bg-slate-50 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
            >
              Cancel
            </button>
          )}
        </div>
      </form>
    </div>
  );
}

