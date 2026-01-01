import { useState, useEffect } from 'react';
import { api } from '../../services/api';
import type {
  AdminSettingsResponse,
  AdminUserListResponse,
  AdminPostListResponse,
  AdminPluginListResponse,
} from '../../types';

type TabType = 'settings' | 'users' | 'posts' | 'plugins';

export function AdminPanel() {
  const [activeTab, setActiveTab] = useState<TabType>('settings');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Settings state
  const [settings, setSettings] = useState<AdminSettingsResponse | null>(null);

  // Users state
  const [users, setUsers] = useState<AdminUserListResponse[]>([]);
  const [resetPasswordUserId, setResetPasswordUserId] = useState<number | null>(null);
  const [newPassword, setNewPassword] = useState('');

  // Posts state
  const [posts, setPosts] = useState<AdminPostListResponse[]>([]);

  // Plugins state
  const [plugins, setPlugins] = useState<AdminPluginListResponse[]>([]);

  // Load settings
  const loadSettings = async () => {
    try {
      setLoading(true);
      setError(null);
      const data = await api.getAdminSettings();
      setSettings(data);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load settings');
    } finally {
      setLoading(false);
    }
  };

  // Load users
  const loadUsers = async () => {
    try {
      setLoading(true);
      setError(null);
      const data = await api.getAdminUsers();
      setUsers(data);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load users');
    } finally {
      setLoading(false);
    }
  };

  // Load posts
  const loadPosts = async () => {
    try {
      setLoading(true);
      setError(null);
      const data = await api.getAdminPosts();
      setPosts(data);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load posts');
    } finally {
      setLoading(false);
    }
  };

  // Load plugins
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

  useEffect(() => {
    if (activeTab === 'settings') {
      loadSettings();
    } else if (activeTab === 'users') {
      loadUsers();
    } else if (activeTab === 'posts') {
      loadPosts();
    } else if (activeTab === 'plugins') {
      loadPlugins();
    }
  }, [activeTab]);

  const handleUpdateSettings = async () => {
    if (!settings) return;
    try {
      setLoading(true);
      setError(null);
      const updated = await api.updateAdminSettings({
        allow_external_registration: settings.allow_external_registration,
        maintenance_mode: settings.maintenance_mode,
      });
      setSettings(updated);
      alert('设置已更新');
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to update settings');
    } finally {
      setLoading(false);
    }
  };

  const handleBanUser = async (userId: number, banned: boolean) => {
    try {
      setLoading(true);
      setError(null);
      await api.banUser(userId, banned);
      await loadUsers();
      alert(`用户已${banned ? '封禁' : '解封'}`);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to update user ban status');
    } finally {
      setLoading(false);
    }
  };

  const handleResetPassword = async (userId: number) => {
    if (!newPassword || newPassword.length < 6) {
      alert('密码长度至少为6位');
      return;
    }
    try {
      setLoading(true);
      setError(null);
      await api.resetUserPassword(userId, newPassword);
      setResetPasswordUserId(null);
      setNewPassword('');
      alert('密码已重置');
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to reset password');
    } finally {
      setLoading(false);
    }
  };

  const handleDeletePost = async (postId: string) => {
    if (!confirm('确定要删除这篇帖子吗？')) return;
    try {
      setLoading(true);
      setError(null);
      await api.adminDeletePost(postId);
      await loadPosts();
      alert('帖子已删除');
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to delete post');
    } finally {
      setLoading(false);
    }
  };

  const handleTogglePlugin = async (pluginId: number, enabled: boolean) => {
    try {
      setLoading(true);
      setError(null);
      await api.updateAdminPlugin(pluginId, { enabled });
      await loadPlugins();
      alert(`插件已${enabled ? '启用' : '禁用'}`);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to update plugin');
    } finally {
      setLoading(false);
    }
  };

  const tabs = [
    { id: 'settings' as TabType, label: '设置' },
    { id: 'users' as TabType, label: '用户管理' },
    { id: 'posts' as TabType, label: '帖子管理' },
    { id: 'plugins' as TabType, label: '插件管理' },
  ];

  return (
    <div className="min-h-screen bg-gray-50">
      {/* Header */}
      <div className="bg-white border-b border-gray-200">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-4">
          <div className="flex justify-between items-center">
            <h1 className="text-2xl font-bold text-gray-900">管理员面板</h1>
            <button
              onClick={() => {
                api.clearTokens();
                window.location.reload();
              }}
              className="px-4 py-2 text-sm font-medium text-gray-700 bg-white border border-gray-300 rounded-md hover:bg-gray-50"
            >
              退出登录
            </button>
          </div>
        </div>
      </div>

      {/* Tabs */}
      <div className="bg-white border-b border-gray-200">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="flex space-x-1">
            {tabs.map((tab) => (
              <button
                key={tab.id}
                onClick={() => setActiveTab(tab.id)}
                className={`px-4 py-3 font-medium text-sm transition-colors ${
                  activeTab === tab.id
                    ? 'text-orange-600 border-b-2 border-orange-600'
                    : 'text-gray-600 hover:text-gray-900'
                }`}
              >
                {tab.label}
              </button>
            ))}
          </div>
        </div>
      </div>

      {/* Error message */}
      {error && (
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 mt-4">
          <div className="p-3 bg-red-50 border border-red-200 text-red-700 rounded">
            {error}
          </div>
        </div>
      )}

      {/* Content */}
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
        {loading && (
          <div className="text-center py-8 text-gray-500">加载中...</div>
        )}

        {/* Settings Tab */}
        {activeTab === 'settings' && settings && !loading && (
          <div className="bg-white rounded-lg shadow p-6 space-y-6">
            <div>
              <label className="flex items-center space-x-3">
                <input
                  type="checkbox"
                  checked={settings.allow_external_registration}
                  onChange={(e) =>
                    setSettings({ ...settings, allow_external_registration: e.target.checked })
                  }
                  className="w-5 h-5 text-orange-600 rounded"
                />
                <span className="text-gray-700">允许外部用户注册</span>
              </label>
            </div>
            <div>
              <label className="flex items-center space-x-3">
                <input
                  type="checkbox"
                  checked={settings.maintenance_mode}
                  onChange={(e) =>
                    setSettings({ ...settings, maintenance_mode: e.target.checked })
                  }
                  className="w-5 h-5 text-orange-600 rounded"
                />
                <span className="text-gray-700">维护模式</span>
              </label>
            </div>
            <button
              onClick={handleUpdateSettings}
              className="px-4 py-2 bg-orange-600 text-white rounded hover:bg-orange-700 transition-colors"
            >
              保存设置
            </button>
          </div>
        )}

        {/* Users Tab */}
        {activeTab === 'users' && !loading && (
          <div className="bg-white rounded-lg shadow overflow-hidden">
            <table className="min-w-full divide-y divide-gray-200">
              <thead className="bg-gray-50">
                <tr>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">ID</th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">用户名</th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">邮箱</th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">角色</th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">状态</th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">操作</th>
                </tr>
              </thead>
              <tbody className="bg-white divide-y divide-gray-200">
                {users.map((item) => (
                  <tr key={item.user.id}>
                    <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">{item.user.id}</td>
                    <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">{item.user.username}</td>
                    <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">{item.user.email}</td>
                    <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">{item.user.role}</td>
                    <td className="px-6 py-4 whitespace-nowrap text-sm">
                      {item.is_banned ? (
                        <span className="px-2 py-1 text-xs font-semibold rounded bg-red-100 text-red-800">已封禁</span>
                      ) : (
                        <span className="px-2 py-1 text-xs font-semibold rounded bg-green-100 text-green-800">正常</span>
                      )}
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap text-sm space-x-2">
                      <button
                        onClick={() => handleBanUser(item.user.id, !item.is_banned)}
                        className={`px-3 py-1 rounded text-xs ${
                          item.is_banned
                            ? 'bg-green-100 text-green-800 hover:bg-green-200'
                            : 'bg-red-100 text-red-800 hover:bg-red-200'
                        }`}
                      >
                        {item.is_banned ? '解封' : '封禁'}
                      </button>
                      {resetPasswordUserId === item.user.id ? (
                        <div className="inline-flex space-x-2">
                          <input
                            type="password"
                            value={newPassword}
                            onChange={(e) => setNewPassword(e.target.value)}
                            placeholder="新密码"
                            className="px-2 py-1 border rounded text-xs"
                          />
                          <button
                            onClick={() => handleResetPassword(item.user.id)}
                            className="px-3 py-1 bg-blue-100 text-blue-800 rounded text-xs hover:bg-blue-200"
                          >
                            确认
                          </button>
                          <button
                            onClick={() => {
                              setResetPasswordUserId(null);
                              setNewPassword('');
                            }}
                            className="px-3 py-1 bg-gray-100 text-gray-800 rounded text-xs hover:bg-gray-200"
                          >
                            取消
                          </button>
                        </div>
                      ) : (
                        <button
                          onClick={() => setResetPasswordUserId(item.user.id)}
                          className="px-3 py-1 bg-blue-100 text-blue-800 rounded text-xs hover:bg-blue-200"
                        >
                          重置密码
                        </button>
                      )}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}

        {/* Posts Tab */}
        {activeTab === 'posts' && !loading && (
          <div className="bg-white rounded-lg shadow overflow-hidden">
            <table className="min-w-full divide-y divide-gray-200">
              <thead className="bg-gray-50">
                <tr>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">ID</th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">标题</th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">分类</th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">创建时间</th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">操作</th>
                </tr>
              </thead>
              <tbody className="bg-white divide-y divide-gray-200">
                {posts.map((item) => (
                  <tr key={item.post.id}>
                    <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">{item.post.id}</td>
                    <td className="px-6 py-4 text-sm text-gray-900">{item.post.title}</td>
                    <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">{item.post.category}</td>
                    <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                      {new Date(item.post.created_at).toLocaleDateString()}
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap text-sm">
                      <button
                        onClick={() => handleDeletePost(item.post.id)}
                        className="px-3 py-1 bg-red-100 text-red-800 rounded text-xs hover:bg-red-200"
                      >
                        删除
                      </button>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}

        {/* Plugins Tab */}
        {activeTab === 'plugins' && !loading && (
          <div className="bg-white rounded-lg shadow overflow-hidden">
            <table className="min-w-full divide-y divide-gray-200">
              <thead className="bg-gray-50">
                <tr>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">名称</th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">描述</th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">版本</th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">状态</th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">操作</th>
                </tr>
              </thead>
              <tbody className="bg-white divide-y divide-gray-200">
                {plugins.map((plugin) => (
                  <tr key={plugin.id}>
                    <td className="px-6 py-4 whitespace-nowrap text-sm font-medium text-gray-900">{plugin.name}</td>
                    <td className="px-6 py-4 text-sm text-gray-900">{plugin.description || '-'}</td>
                    <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">{plugin.version}</td>
                    <td className="px-6 py-4 whitespace-nowrap text-sm">
                      {plugin.enabled ? (
                        <span className="px-2 py-1 text-xs font-semibold rounded bg-green-100 text-green-800">已启用</span>
                      ) : (
                        <span className="px-2 py-1 text-xs font-semibold rounded bg-gray-100 text-gray-800">已禁用</span>
                      )}
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap text-sm">
                      <button
                        onClick={() => handleTogglePlugin(plugin.id, !plugin.enabled)}
                        className={`px-3 py-1 rounded text-xs ${
                          plugin.enabled
                            ? 'bg-gray-100 text-gray-800 hover:bg-gray-200'
                            : 'bg-green-100 text-green-800 hover:bg-green-200'
                        }`}
                      >
                        {plugin.enabled ? '禁用' : '启用'}
                      </button>
                    </td>
                  </tr>
                ))}
                {plugins.length === 0 && (
                  <tr>
                    <td colSpan={5} className="px-6 py-4 text-center text-sm text-gray-500">
                      暂无插件
                    </td>
                  </tr>
                )}
              </tbody>
            </table>
          </div>
        )}
      </div>
    </div>
  );
}
