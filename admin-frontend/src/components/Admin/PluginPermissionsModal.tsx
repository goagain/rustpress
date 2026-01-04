import { useState, useEffect } from 'react';
import { api } from '../../services/api';
import type {
  PluginPermissionInfo,
} from '../../types';

interface PluginPermissionsModalProps {
  pluginId: string;
  pluginName: string;
  pluginStatus: string;
  isOpen: boolean;
  onClose: () => void;
  onPermissionsUpdated: () => void;
}

export function PluginPermissionsModal({
  pluginId,
  pluginName,
  pluginStatus,
  isOpen,
  onClose,
  onPermissionsUpdated,
}: PluginPermissionsModalProps) {
  const [permissions, setPermissions] = useState<PluginPermissionInfo[]>([]);
  const [loading, setLoading] = useState(false);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (isOpen && pluginId) {
      loadPermissions();
    }
  }, [isOpen, pluginId]);

  const loadPermissions = async () => {
    try {
      setLoading(true);
      setError(null);
      const data = await api.getPluginPermissions(pluginId);
      setPermissions(data.permissions);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load permissions');
    } finally {
      setLoading(false);
    }
  };

  const handlePermissionToggle = (permission: string, isGranted: boolean) => {
    setPermissions(prev =>
      prev.map(p =>
        p.permission === permission ? { ...p, is_granted: isGranted } : p
      )
    );
  };

  const handleSave = async () => {
    try {
      setSaving(true);
      setError(null);

      const updates: Record<string, boolean> = {};
      permissions.forEach(p => {
        if (pluginStatus === 'pending_review') {
          // For pending review, include all permissions that are being approved
          updates[p.permission] = p.is_granted;
        } else if (p.permission_type === 'optional') {
          // For normal management, only update optional permissions
          updates[p.permission] = p.is_granted;
        }
      });

      if (pluginStatus === 'pending_review') {
        // Use review API for pending plugins
        await api.reviewPluginPermissions(pluginId, updates);
      } else {
        // Use regular update API for normal permission management
        await api.updatePluginPermissions(pluginId, { permissions: updates });
      }

      onPermissionsUpdated();
      onClose();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to update permissions');
    } finally {
      setSaving(false);
    }
  };

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 bg-gray-600 bg-opacity-50 overflow-y-auto h-full w-full z-50">
      <div className="relative top-20 mx-auto p-5 border w-11/12 max-w-2xl shadow-lg rounded-md bg-white">
        <div className="mt-3">
          <div className="flex items-center justify-between mb-4">
            <h3 className="text-lg font-medium text-gray-900">
              {pluginStatus === 'pending_review' ? 'Review Permissions' : 'Manage Permissions'} for {pluginName}
            </h3>
            <button
              onClick={onClose}
              className="text-gray-400 hover:text-gray-600"
            >
              <span className="sr-only">Close</span>
              <svg className="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
              </svg>
            </button>
          </div>

          {loading ? (
            <div className="flex items-center justify-center py-8">
              <div className="text-gray-500">Loading permissions...</div>
            </div>
          ) : error ? (
            <div className="bg-red-50 border border-red-200 rounded-md p-4 mb-4">
              <div className="text-red-800">Error: {error}</div>
              <button
                onClick={loadPermissions}
                className="mt-2 px-3 py-1 bg-red-600 text-white rounded hover:bg-red-700"
              >
                Retry
              </button>
            </div>
          ) : pluginStatus === 'pending_review' ? (
            // Permission Review UI for pending plugins
            <div className="space-y-4">
              <div className="bg-yellow-50 border border-yellow-200 rounded-md p-4">
                <div className="flex">
                  <div className="flex-shrink-0">
                    <svg className="h-5 w-5 text-yellow-400" viewBox="0 0 20 20" fill="currentColor">
                      <path fillRule="evenodd" d="M8.257 3.099c.765-1.36 2.722-1.36 3.486 0l5.58 9.92c.75 1.334-.213 2.98-1.742 2.98H4.42c-1.53 0-2.493-1.646-1.743-2.98l5.58-9.92zM11 13a1 1 0 11-2 0 1 1 0 012 0zm-1-8a1 1 0 00-1 1v3a1 1 0 002 0V6a1 1 0 00-1-1z" clipRule="evenodd" />
                    </svg>
                  </div>
                  <div className="ml-3">
                    <h3 className="text-sm font-medium text-yellow-800">
                      Permission Review Required
                    </h3>
                    <div className="mt-2 text-sm text-yellow-700">
                      <p>This plugin has requested additional permissions in its update. Please review and approve the new permissions below.</p>
                    </div>
                  </div>
                </div>
              </div>

              <div className="space-y-3">
                {permissions.map((permission) => (
                  <div key={permission.permission} className="flex items-center justify-between p-4 border border-gray-200 rounded-md bg-yellow-50">
                    <div className="flex-1">
                      <div className="flex items-center">
                        <span className="font-medium text-gray-900">{permission.permission}</span>
                        {permission.permission_type === 'required' && (
                          <span className="ml-2 inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-red-100 text-red-800">
                            New Required
                          </span>
                        )}
                        {permission.permission_type === 'optional' && (
                          <span className="ml-2 inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-blue-100 text-blue-800">
                            New Optional
                          </span>
                        )}
                      </div>
                      {permission.description && (
                        <p className="mt-1 text-sm text-gray-600">{permission.description}</p>
                      )}
                    </div>

                    <div className="ml-4">
                      <label className="relative inline-flex items-center cursor-pointer">
                        <input
                          type="checkbox"
                          className="sr-only peer"
                          checked={permission.is_granted}
                          onChange={(e) => handlePermissionToggle(permission.permission, e.target.checked)}
                        />
                        <div className="w-11 h-6 bg-gray-200 peer-focus:outline-none peer-focus:ring-4 peer-focus:ring-yellow-300 rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-yellow-600"></div>
                      </label>
                    </div>
                  </div>
                ))}
              </div>
            </div>
          ) : (
            <div className="space-y-4">
              <div className="bg-blue-50 border border-blue-200 rounded-md p-3">
                <p className="text-sm text-blue-800">
                  <strong>Required permissions</strong> are automatically granted and cannot be disabled.
                  They are essential for the plugin to function.
                </p>
              </div>

              <div className="space-y-3">
                {permissions.map((permission) => (
                  <div key={permission.permission} className="flex items-center justify-between p-3 border border-gray-200 rounded-md">
                    <div className="flex-1">
                      <div className="flex items-center">
                        <span className="font-medium text-gray-900">{permission.permission}</span>
                        {permission.permission_type === 'required' && (
                          <span className="ml-2 inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-gray-100 text-gray-800">
                            Required
                          </span>
                        )}
                        {permission.permission_type === 'optional' && (
                          <span className="ml-2 inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-blue-100 text-blue-800">
                            Optional
                          </span>
                        )}
                      </div>
                      {permission.description && (
                        <p className="mt-1 text-sm text-gray-600">{permission.description}</p>
                      )}
                    </div>

                    <div className="ml-4">
                      {permission.permission_type === 'required' ? (
                        <span className="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-green-100 text-green-800">
                          ✓ Granted
                        </span>
                      ) : (
                        <label className="relative inline-flex items-center cursor-pointer">
                          <input
                            type="checkbox"
                            className="sr-only peer"
                            checked={permission.is_granted}
                            onChange={(e) => handlePermissionToggle(permission.permission, e.target.checked)}
                          />
                          <div className="w-11 h-6 bg-gray-200 peer-focus:outline-none peer-focus:ring-4 peer-focus:ring-blue-300 rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-blue-600"></div>
                        </label>
                      )}
                    </div>
                  </div>
                ))}
              </div>
            </div>
          )}

          <div className="flex justify-end space-x-3 mt-6">
            <button
              onClick={onClose}
              className="px-4 py-2 bg-gray-300 text-gray-700 rounded hover:bg-gray-400"
              disabled={saving}
            >
              Cancel
            </button>
            <button
              onClick={handleSave}
              disabled={saving}
              className="px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700 disabled:opacity-50"
            >
              {saving ? 'Saving...' : 'Save Permissions'}
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}