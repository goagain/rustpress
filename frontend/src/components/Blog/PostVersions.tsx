import { useState, useEffect } from 'react';
import { X, Clock, RotateCcw, Eye } from 'lucide-react';
import { api } from '../../services/api';
import { PostVersionDetail } from './PostVersionDetail';
import type { PostVersionResponse } from '../../types';

interface PostVersionsProps {
  postId: string;
  onClose: () => void;
  onRestore?: () => void;
  onViewVersion?: (versionId: string) => void;
}

export function PostVersions({ postId, onClose, onRestore, onViewVersion }: PostVersionsProps) {
  const [versions, setVersions] = useState<PostVersionResponse[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState('');
  const [restoring, setRestoring] = useState<string | null>(null);
  const [viewingVersionId, setViewingVersionId] = useState<string | null>(null);

  useEffect(() => {
    loadVersions();
  }, [postId]);

  const loadVersions = async () => {
    try {
      setLoading(true);
      const data = await api.getPostVersions(postId);
      setVersions(data);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load versions');
    } finally {
      setLoading(false);
    }
  };

  const handleRestore = async (versionId: string) => {
    if (!confirm('Are you sure you want to restore this version? This will create a new version of the current state.')) {
      return;
    }

    try {
      setRestoring(versionId);
      await api.restorePostFromVersion(postId, versionId);
      onRestore?.();
      onClose();
    } catch (err) {
      alert(err instanceof Error ? err.message : 'Failed to restore version');
    } finally {
      setRestoring(null);
    }
  };

  const handleViewVersion = (versionId: string) => {
    if (onViewVersion) {
      onViewVersion(versionId);
    } else {
      setViewingVersionId(versionId);
    }
  };

  const handleBackToVersions = () => {
    setViewingVersionId(null);
  };

  // If viewing a specific version, show the detail view
  if (viewingVersionId) {
    return (
      <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50 p-4 overflow-y-auto">
        <div className="bg-white rounded-xl shadow-xl max-w-5xl w-full my-8">
          <PostVersionDetail
            postId={postId}
            versionId={viewingVersionId}
            onBack={handleBackToVersions}
          />
        </div>
      </div>
    );
  }

  if (loading) {
    return (
      <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
        <div className="bg-white rounded-xl p-8 max-w-2xl w-full mx-4">
          <div className="text-center text-slate-400">Loading versions...</div>
        </div>
      </div>
    );
  }

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50 p-4">
      <div className="bg-white rounded-xl shadow-xl max-w-4xl w-full max-h-[90vh] flex flex-col">
        <div className="flex items-center justify-between p-6 border-b border-slate-200">
          <h2 className="text-2xl font-bold text-slate-900">Version History</h2>
          <button
            onClick={onClose}
            className="p-2 text-slate-400 hover:text-slate-600 transition-colors"
          >
            <X size={24} />
          </button>
        </div>

        <div className="flex-1 overflow-y-auto p-6">
          {error && (
            <div className="bg-red-50 border border-red-200 text-red-700 px-4 py-3 rounded-lg mb-4">
              {error}
            </div>
          )}

          {versions.length === 0 ? (
            <div className="text-center py-12 text-slate-500">
              <p>No version history available</p>
            </div>
          ) : (
            <div className="space-y-4">
              {versions.map((version) => (
                <div
                  key={version.id}
                  className="border border-slate-200 rounded-lg p-4 hover:shadow-md transition-shadow"
                >
                  <div className="flex items-start justify-between gap-4">
                    <div 
                      className="flex-1 cursor-pointer"
                      onClick={() => handleViewVersion(version.id)}
                    >
                      <div className="flex items-center gap-3 mb-2">
                        <span className="inline-flex items-center gap-1 px-2 py-1 bg-slate-100 text-slate-700 text-xs font-medium rounded">
                          Version {version.version_number}
                        </span>
                        <span className="inline-flex items-center gap-1 text-xs text-slate-500">
                          <Clock size={12} />
                          {new Date(version.created_at).toLocaleString()}
                        </span>
                        {version.change_note && (
                          <span className="text-xs text-slate-600 italic">
                            "{version.change_note}"
                          </span>
                        )}
                      </div>
                      <h3 className="font-semibold text-slate-900 mb-1 hover:text-orange-600 transition-colors">
                        {version.title}
                      </h3>
                      <p className="text-sm text-slate-600 line-clamp-2">
                        {version.content.substring(0, 200)}
                        {version.content.length > 200 && '...'}
                      </p>
                    </div>
                    <div className="flex items-center gap-2">
                      <button
                        onClick={() => handleViewVersion(version.id)}
                        className="inline-flex items-center gap-2 px-3 py-2 text-sm font-medium text-slate-700 bg-slate-100 hover:bg-slate-200 rounded-lg transition-colors"
                      >
                        <Eye size={16} />
                        View
                      </button>
                      <button
                        onClick={() => handleRestore(version.id)}
                        disabled={restoring === version.id}
                        className="inline-flex items-center gap-2 px-3 py-2 text-sm font-medium text-white bg-orange-500 hover:bg-orange-600 rounded-lg transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
                      >
                        {restoring === version.id ? (
                          <>Restoring...</>
                        ) : (
                          <>
                            <RotateCcw size={16} />
                            Restore
                          </>
                        )}
                      </button>
                    </div>
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
