import { Link, useLocation } from 'react-router-dom';
import { clearTokens } from '../../services/api';

interface AdminPanelProps {
  children: React.ReactNode;
}

export function AdminPanel({ children }: AdminPanelProps) {
  const location = useLocation();

  const tabs = [
    { id: 'settings', label: 'Settings', path: '/settings/general' },
    { id: 'users', label: 'Users', path: '/users' },
    { id: 'posts', label: 'Posts', path: '/posts' },
    { id: 'plugins', label: 'Plugins', path: '/plugins' },
  ];

  const settingsSubTabs = [
    { id: 'general', label: 'General', path: 'settings/general' },
    { id: 'openai', label: 'OpenAI', path: 'settings/openai' },
  ];

  const getActiveTab = () => {
    const path = location.pathname;
    if (path.includes('/settings')) return 'settings';
    if (path.includes('/users')) return 'users';
    if (path.includes('/posts')) return 'posts';
    if (path.includes('/plugins')) return 'plugins';
    return 'settings';
  };

  const getActiveSubTab = () => {
    const path = location.pathname;
    if (path.includes('/settings/openai')) return 'openai';
    return 'general';
  };

  const activeTab = getActiveTab();
  const activeSubTab = getActiveSubTab();
  const isInSettings = activeTab === 'settings';

  return (
    <div className="min-h-screen bg-gray-50">
      {/* Header */}
      <div className="bg-white border-b border-gray-200">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-4">
          <div className="flex justify-between items-center">
            <h1 className="text-2xl font-bold text-gray-900">Admin Panel</h1>
            <button
              onClick={() => {
                clearTokens();
                window.location.reload();
              }}
              className="px-4 py-2 text-sm font-medium text-gray-700 bg-white border border-gray-300 rounded-md hover:bg-gray-50"
            >
              Logout
            </button>
          </div>
        </div>
      </div>

      {/* Tabs */}
      <div className="bg-white border-b border-gray-200">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="flex space-x-1">
            {tabs.map((tab) => (
              <Link
                key={tab.id}
                to={tab.path}
                className={`px-4 py-3 font-medium text-sm transition-colors ${
                  activeTab === tab.id
                    ? 'text-orange-600 border-b-2 border-orange-600'
                    : 'text-gray-600 hover:text-gray-900'
                }`}
              >
                {tab.label}
              </Link>
            ))}
          </div>
        </div>
      </div>

      {/* Sub-tabs for Settings */}
      {isInSettings && (
        <div className="bg-gray-50 border-b border-gray-200">
          <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
            <div className="flex space-x-1">
              {settingsSubTabs.map((subTab) => (
                <Link
                  key={subTab.id}
                  to={subTab.path}
                  className={`px-4 py-2 font-medium text-sm transition-colors ${
                    activeSubTab === subTab.id
                      ? 'text-orange-600 border-b-2 border-orange-600 bg-white'
                      : 'text-gray-600 hover:text-gray-900'
                  }`}
                >
                  {subTab.label}
                </Link>
              ))}
            </div>
          </div>
        </div>
      )}

      {/* Content */}
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
        {children}
      </div>
    </div>
  );
}