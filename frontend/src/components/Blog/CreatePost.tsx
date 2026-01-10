import { useState, useRef, useEffect, useCallback } from 'react';
import { Save, Eye, EyeOff, X, Loader2, Image as ImageIcon, Cloud } from 'lucide-react';
import { api, isAuthenticated } from '../../services/api';
import { normalizeImageUrl } from '../../utils/url';
import { MarkdownRenderer } from '../Markdown/MarkdownRenderer';
import type { PostResponse } from '../../types';

interface CreatePostProps {
  postId?: string;
  initialPost?: PostResponse;
  onSuccess?: () => void;
  onCancel?: () => void;
}

export function CreatePost({ postId, initialPost, onSuccess, onCancel }: CreatePostProps) {
  const isEditMode = !!postId;
  const [title, setTitle] = useState(initialPost?.title || '');
  const [category, setCategory] = useState(initialPost?.category || '');
  const [description, setDescription] = useState(initialPost?.description || '');
  const [content, setContent] = useState(initialPost?.content || '');
  const [showPreview, setShowPreview] = useState(false);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState('');
  const [uploadingImage, setUploadingImage] = useState(false);
  const [savingDraft, setSavingDraft] = useState(false);
  const [draftSaved, setDraftSaved] = useState(false);
  const [lastSavedContent, setLastSavedContent] = useState('');
  const fileInputRef = useRef<HTMLInputElement>(null);
  const textareaRef = useRef<HTMLTextAreaElement>(null);
  const saveDraftTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const lastContentRef = useRef(content);
  const charCountSinceLastSaveRef = useRef(0);

  // Load draft on mount
  useEffect(() => {
    const loadDraft = async () => {
      if (!isAuthenticated()) return;
      
      try {
        const draft = await api.getDraft(postId || undefined);
        if (draft && (!initialPost || new Date(draft.updated_at) > new Date(initialPost.updated_at))) {
          setTitle(draft.title);
          setCategory(draft.category);
          setDescription(''); // Draft doesn't have description
          setContent(draft.content);
          const draftContent = draft.title + draft.category + draft.content;
          setLastSavedContent(draftContent);
          lastContentRef.current = draftContent;
        } else if (initialPost) {
          // Initialize with post content
          const postContent = initialPost.title + initialPost.category + initialPost.content;
          setLastSavedContent(postContent);
          lastContentRef.current = postContent;
        }
      } catch (err) {
        // Draft not found or error, ignore
        if (initialPost) {
          // Initialize with post content
          const postContent = initialPost.title + initialPost.category + initialPost.content;
          setLastSavedContent(postContent);
          lastContentRef.current = postContent;
        }
      }
    };

    loadDraft();
  }, [postId, initialPost]);

  // Auto-save draft
  const saveDraft = useCallback(async () => {
    if (!isAuthenticated()) return;
    if (!title.trim() && !content.trim()) return; // Don't save empty drafts
    
    const currentContent = title + category + description + content;
    if (currentContent === lastSavedContent) return; // No changes

    try {
      setSavingDraft(true);
      await api.saveDraft({
        post_id: postId ? parseInt(postId, 10) : null,
        title: title.trim() || 'Untitled',
        category: category.trim() || '',
        description: description.trim() || null,
        content: content.trim() || '',
      });
      setLastSavedContent(currentContent);
      charCountSinceLastSaveRef.current = 0;
      setDraftSaved(true);
      setTimeout(() => setDraftSaved(false), 2000);
    } catch (err) {
      console.error('Failed to save draft:', err);
    } finally {
      setSavingDraft(false);
    }
  }, [title, category, content, postId, lastSavedContent]);

  // Auto-save logic: every 30 seconds or after 50 characters
  useEffect(() => {
    const currentContent = title + category + description + content;
    const charDiff = Math.abs(currentContent.length - lastContentRef.current.length);
    
    // Update character count
    if (charDiff > 0) {
      charCountSinceLastSaveRef.current += charDiff;
      
      // Save if 50+ characters changed
      if (charCountSinceLastSaveRef.current >= 50) {
        saveDraft();
        charCountSinceLastSaveRef.current = 0; // Reset counter after save
      }
    }
    
    lastContentRef.current = currentContent;

    // Clear existing timeout
    if (saveDraftTimeoutRef.current) {
      clearTimeout(saveDraftTimeoutRef.current);
    }

    // Set timeout for 30 seconds
    saveDraftTimeoutRef.current = setTimeout(() => {
      saveDraft();
    }, 30000);

    return () => {
      if (saveDraftTimeoutRef.current) {
        clearTimeout(saveDraftTimeoutRef.current);
      }
    };
  }, [title, category, content, saveDraft]);

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      if (saveDraftTimeoutRef.current) {
        clearTimeout(saveDraftTimeoutRef.current);
      }
    };
  }, []);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    
    if (!title.trim() || !content.trim()) {
      setError('Please fill in title and content');
      return;
    }

    setLoading(true);
    setError('');

    try {
      if (isEditMode && postId) {
        // Update existing post - always create version before update
        await api.updatePost(postId, {
          title: title.trim(),
          category: category.trim(),
          description: description.trim() || undefined,
          content: content.trim(),
          create_version: true, // Always create version when editing
          change_note: undefined, // Can be enhanced later to allow user to add change notes
        });
        
        // Delete draft after successful update
        try {
          await api.deleteDraft(postId);
        } catch (err) {
          // Ignore draft deletion errors
        }
      } else {
        // Create new post
        const token = localStorage.getItem('access_token');
        if (!token) {
          setError('Not authenticated. Please login first.');
          return;
        }

        const payload = JSON.parse(atob(token.split('.')[1]));
        const authorId = parseInt(payload.sub, 10);
        
        if (!authorId) {
          setError('Failed to get user information from token.');
          return;
        }
        
        await api.createPost({
          title: title.trim(),
          category: category.trim() || null,
          description: description.trim() || null,
          content: content.trim(),
          author_id: authorId,
        });

        // Delete draft after successful creation
        try {
          await api.deleteDraft();
        } catch (err) {
          // Ignore draft deletion errors
        }
      }

      // Reset form
      setTitle('');
      setCategory('');
      setDescription('');
      setContent('');
      setError('');

      // Call success callback
      onSuccess?.();
    } catch (err) {
      setError(err instanceof Error ? err.message : `Failed to ${isEditMode ? 'update' : 'create'} post`);
    } finally {
      setLoading(false);
    }
  };


  // Handle image upload
  const handleImageUpload = async (file: File) => {
    if (!file.type.startsWith('image/')) {
      setError('Please select an image file');
      return;
    }

    setUploadingImage(true);
    setError('');

    try {
      const result = await api.uploadImage(file);
      
      // Normalize URL to relative path if from same server
      const normalizedUrl = normalizeImageUrl(result.url);
      
      // Insert image markdown at cursor position
      const textarea = textareaRef.current;
      if (textarea) {
        const start = textarea.selectionStart;
        const end = textarea.selectionEnd;
        const imageMarkdown = `![${file.name}](${normalizedUrl})`;
        const newContent = 
          content.substring(0, start) + 
          imageMarkdown + 
          content.substring(end);
        setContent(newContent);
        
        // Set cursor position after inserted markdown
        setTimeout(() => {
          textarea.focus();
          const newPosition = start + imageMarkdown.length;
          textarea.setSelectionRange(newPosition, newPosition);
        }, 0);
      } else {
        // If no cursor position, append to end
        setContent(content + `\n![${file.name}](${normalizedUrl})\n`);
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to upload image');
    } finally {
      setUploadingImage(false);
    }
  };

  // Handle file input change
  const handleFileInputChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (file) {
      handleImageUpload(file);
    }
    // Reset input
    if (fileInputRef.current) {
      fileInputRef.current.value = '';
    }
  };

  // Handle paste event
  const handlePaste = async (e: React.ClipboardEvent<HTMLTextAreaElement>) => {
    const items = e.clipboardData.items;
    
    for (let i = 0; i < items.length; i++) {
      const item = items[i];
      
      // Check if pasted item is an image
      if (item.type.startsWith('image/')) {
        e.preventDefault();
        
        const file = item.getAsFile();
        if (file) {
          await handleImageUpload(file);
        }
        break;
      }
    }
  };

  // Handle drag and drop
  const handleDrop = async (e: React.DragEvent<HTMLTextAreaElement>) => {
    e.preventDefault();
    
    const files = e.dataTransfer.files;
    if (files.length > 0) {
      const file = files[0];
      if (file.type.startsWith('image/')) {
        await handleImageUpload(file);
      }
    }
  };

  const handleDragOver = (e: React.DragEvent<HTMLTextAreaElement>) => {
    e.preventDefault();
  };

  return (
    <div className="bg-white rounded-xl border border-slate-200 p-8">
      <div className="flex items-center justify-between mb-6">
        <div className="flex items-center gap-3">
          <h2 className="text-2xl font-bold text-slate-900">
            {isEditMode ? 'Edit Post' : 'Create New Post'}
          </h2>
          {savingDraft && (
            <span className="inline-flex items-center gap-1 text-sm text-slate-500">
              <Loader2 className="animate-spin" size={14} />
              Saving draft...
            </span>
          )}
          {draftSaved && !savingDraft && (
            <span className="inline-flex items-center gap-1 text-sm text-green-600">
              <Cloud size={14} />
              Draft saved
            </span>
          )}
        </div>
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
            Category
          </label>
          <input
            id="category"
            type="text"
            value={category}
            onChange={(e) => setCategory(e.target.value)}
            className="w-full px-4 py-2 border border-slate-300 rounded-lg focus:ring-2 focus:ring-orange-500 focus:border-orange-500 outline-none transition-colors"
            placeholder="e.g., Announcement, Tutorial, News (optional)"
            disabled={loading}
          />
        </div>

        {/* Description */}
        <div>
          <label htmlFor="description" className="block text-sm font-medium text-slate-700 mb-2">
            Description
          </label>
          <textarea
            id="description"
            value={description}
            onChange={(e) => setDescription(e.target.value)}
            rows={3}
            className="w-full px-4 py-2 border border-slate-300 rounded-lg focus:ring-2 focus:ring-orange-500 focus:border-orange-500 outline-none transition-colors resize-y"
            placeholder="Brief description of the post (optional)"
            disabled={loading}
          />
          <p className="mt-2 text-xs text-slate-500">
            A short summary that will be displayed in the sidebar when viewing the post
          </p>
        </div>

        {/* Content Editor and Preview */}
        <div>
          <div className="flex items-center justify-between mb-2">
            <label htmlFor="content" className="block text-sm font-medium text-slate-700">
              Content (Markdown) *
            </label>
            <div className="flex items-center gap-2">
              {/* Image upload button */}
              <input
                ref={fileInputRef}
                type="file"
                accept="image/*"
                onChange={handleFileInputChange}
                className="hidden"
                disabled={uploadingImage || loading}
              />
              <button
                type="button"
                onClick={() => fileInputRef.current?.click()}
                disabled={uploadingImage || loading}
                className="flex items-center gap-2 px-3 py-1.5 text-sm text-slate-600 hover:text-slate-900 hover:bg-slate-100 rounded-lg transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
                title="Upload image"
              >
                {uploadingImage ? (
                  <>
                    <Loader2 className="animate-spin" size={16} />
                    Uploading...
                  </>
                ) : (
                  <>
                    <ImageIcon size={16} />
                    Upload Image
                  </>
                )}
              </button>
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
          </div>

          <div className={showPreview ? 'grid grid-cols-1 lg:grid-cols-2 gap-4' : ''}>
            {/* Editor */}
            <div className={showPreview ? '' : 'w-full'}>
              <textarea
                ref={textareaRef}
                id="content"
                value={content}
                onChange={(e) => setContent(e.target.value)}
                onPaste={handlePaste}
                onDrop={handleDrop}
                onDragOver={handleDragOver}
                required
                rows={showPreview ? 20 : 15}
                className="w-full px-4 py-3 border border-slate-300 rounded-lg focus:ring-2 focus:ring-orange-500 focus:border-orange-500 outline-none transition-colors font-mono text-sm resize-y"
                placeholder="Write your post content in Markdown format... You can paste images directly here!"
                disabled={loading || uploadingImage}
              />
              <p className="mt-2 text-xs text-slate-500">
                Supports GitHub Flavored Markdown: lists, tables, code blocks, etc.
              </p>
            </div>

            {/* Preview */}
            {showPreview && (
              <div className="border border-slate-300 rounded-lg p-4 bg-white overflow-y-auto max-h-[600px]">
                <div className="markdown-content">
                  {content ? (
                    <MarkdownRenderer content={content} />
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
            disabled={loading || !title.trim() || !content.trim()}
            className="flex items-center gap-2 px-6 py-2.5 bg-orange-500 hover:bg-orange-600 text-white font-semibold rounded-lg transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
          >
            {loading ? (
              <>
                <Loader2 className="animate-spin" size={18} />
                {isEditMode ? 'Updating...' : 'Publishing...'}
              </>
            ) : (
              <>
                <Save size={18} />
                {isEditMode ? 'Update Post' : 'Publish Post'}
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

