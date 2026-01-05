import { useState, useEffect } from 'react';
import { api } from '../../services/api';
import type {
  AdminPluginListResponse,
} from '../../types';
import { PluginPermissionsModal } from './PluginPermissionsModal';

export function PluginsPage() {
  const [plugins, setPlugins] = useState<AdminPluginListResponse[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [permissionsModal, setPermissionsModal] = useState<{
    isOpen: boolean;
    pluginId: string;
    pluginName: string;
    pluginStatus: string;
  }>({
    isOpen: false,
    pluginId: '',
    pluginName: '',
    pluginStatus: '',
  });

  useEffect(() => {
    loadPlugins();
  }, []);

  const loadPlugins = async () => {
    try {
      setLoading(true);
      setError(null);
      const data = await api.getAdminPlugins();
      setPlugins(data);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load plugins');
    } finally {
      setLoading(false);
    }
  };

  const handleTogglePlugin = async (pluginId: number, enabled: boolean) => {
    try {
      setLoading(true);
      const result = await api.updateAdminPlugin(pluginId, { enabled });

      if (enabled && result.requires_permission_review) {
        // Plugin requires permission review
        const permissionsText = result.new_permissions.join('\n• ');
        const confirmEnable = confirm(
          `This plugin requires new permissions:\n\n• ${permissionsText}\n\nDo you want to enable the plugin anyway? You can manage permissions later.`
        );

        if (!confirmEnable) {
          // Revert the enable action
          await api.updateAdminPlugin(pluginId, { enabled: false });
          await loadPlugins();
          return;
        }
      }

      await loadPlugins();
    } catch (err) {
      alert(`Failed to ${enabled ? 'enable' : 'disable'} plugin: ${err instanceof Error ? err.message : 'Unknown error'}`);
    } finally {
      setLoading(false);
    }
  };

  const handleUninstallPlugin = async (pluginId: number, pluginName: string) => {
    if (!confirm(`Are you sure you want to uninstall "${pluginName}"? This action cannot be undone.`)) {
      return;
    }

    try {
      setLoading(true);
      await api.uninstallPlugin(pluginId);
      await loadPlugins();
    } catch (err) {
      alert(`Failed to uninstall plugin: ${err instanceof Error ? err.message : 'Unknown error'}`);
    } finally {
      setLoading(false);
    }
  };

  const handleManagePermissions = (plugin: AdminPluginListResponse) => {
    setPermissionsModal({
      isOpen: true,
      pluginId: plugin.name, // Use name as ID since that's what we use in backend
      pluginName: plugin.name,
      pluginStatus: plugin.status,
    });
  };

  const handlePermissionsUpdated = () => {
    loadPlugins();
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center py-8">
        <div className="text-gray-500">Loading plugins...</div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="bg-red-50 border border-red-200 rounded-md p-4">
        <div className="text-red-800">Error: {error}</div>
        <button
          onClick={loadPlugins}
          className="mt-2 px-3 py-1 bg-red-600 text-white rounded hover:bg-red-700"
        >
          Retry
        </button>
      </div>
    );
  }

  const handleUploadPlugin = () => {
    const input = document.createElement('input');
    input.type = 'file';
    input.accept = '.rpk';
    input.onchange = (e) => {
      const file = (e.target as HTMLInputElement).files?.[0];
      if (file) {
        handleFileUpload(file);
      }
    };
    input.click();
  };

  const handleFileUpload = async (file: File) => {
    try {
      setLoading(true);
      setError(null);

      const formData = new FormData();
      formData.append('plugin', file);

      await api.uploadPlugin(formData);
      await loadPlugins();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to upload plugin');
    } finally {
      setLoading(false);
    }
  };

  const handleBrowseMarketplace = () => {
    // TODO: Implement marketplace browsing
    alert('Marketplace feature coming soon!');
  };

  return (
    <div className="space-y-6">
      <div className="bg-white shadow-sm rounded-lg overflow-hidden">
        <div className="px-6 py-4 border-b border-gray-200">
          <div className="flex items-center justify-between">
            <h3 className="text-lg font-medium text-gray-900">Plugins Management</h3>
            <div className="flex space-x-3">
              <button
                onClick={handleBrowseMarketplace}
                className="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2"
              >
                Browse Marketplace
              </button>
              <button
                onClick={handleUploadPlugin}
                className="px-4 py-2 bg-green-600 text-white rounded-md hover:bg-green-700 focus:outline-none focus:ring-2 focus:ring-green-500 focus:ring-offset-2"
              >
                Upload Plugin
              </button>
            </div>
          </div>
        </div>

        <div className="overflow-x-auto">
          <table className="min-w-full divide-y divide-gray-200">
            <thead className="bg-gray-50">
              <tr>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">Name</th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">Version</th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">Description</th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">Status</th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">Permissions</th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">Actions</th>
              </tr>
            </thead>
            <tbody className="bg-white divide-y divide-gray-200">
              {plugins.map((plugin) => (
                <tr key={plugin.id}>
                  <td className="px-6 py-4 whitespace-nowrap text-sm font-medium text-gray-900">
                    {plugin.name}
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                    {plugin.version}
                  </td>
                  <td className="px-6 py-4 text-sm text-gray-500">
                    {plugin.description}
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                    {plugin.status === 'pending_review' ? (
                      <span className="inline-flex px-2 py-1 text-xs font-semibold rounded-full bg-yellow-100 text-yellow-800">
                        Pending Review
                      </span>
                    ) : plugin.enabled ? (
                      <span className="inline-flex px-2 py-1 text-xs font-semibold rounded-full bg-green-100 text-green-800">
                        Enabled
                      </span>
                    ) : (
                      <span className="inline-flex px-2 py-1 text-xs font-semibold rounded-full bg-gray-100 text-gray-800">
                        Disabled
                      </span>
                    )}
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap text-sm font-medium">
                    <button
                      onClick={() => handleManagePermissions(plugin)}
                      className="px-3 py-1 bg-blue-100 text-blue-800 hover:bg-blue-200 rounded text-xs"
                    >
                      Manage
                    </button>
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap text-sm font-medium space-x-2">
                    {plugin.status === 'pending_review' ? (
                      <button
                        onClick={() => handleManagePermissions(plugin)}
                        className="px-3 py-1 bg-yellow-100 text-yellow-800 hover:bg-yellow-200 rounded text-xs"
                      >
                        Review Permissions
                      </button>
                    ) : (
                      <>
                        <button
                          onClick={() => handleTogglePlugin(plugin.id, !plugin.enabled)}
                          className={`px-3 py-1 rounded text-xs ${
                            plugin.enabled
                              ? 'bg-red-100 text-red-800 hover:bg-red-200'
                              : 'bg-green-100 text-green-800 hover:bg-green-200'
                          }`}
                        >
                          {plugin.enabled ? 'Disable' : 'Enable'}
                        </button>
                        <button
                          onClick={() => handleUninstallPlugin(plugin.id, plugin.name)}
                          className="px-3 py-1 bg-red-600 text-white hover:bg-red-700 rounded text-xs"
                          disabled={loading}
                        >
                          Uninstall
                        </button>
                      </>
                    )}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </div>

      <PluginPermissionsModal
        pluginId={permissionsModal.pluginId}
        pluginName={permissionsModal.pluginName}
        pluginStatus={permissionsModal.pluginStatus || 'enabled'}
        isOpen={permissionsModal.isOpen}
        onClose={() => setPermissionsModal({ isOpen: false, pluginId: '', pluginName: '', pluginStatus: '' })}
        onPermissionsUpdated={handlePermissionsUpdated}
      />
    </div>
  );
}