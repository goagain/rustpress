import type { SettingsTabComponentProps } from '../SettingsTabRegistry';

export function GeneralSettingsTab({
  tab,
  settings,
  onUpdate,
  onSave,
  loading,
}: SettingsTabComponentProps) {
  return (
    <div className="bg-white rounded-lg shadow p-6 space-y-6">
      {tab.description && (
        <p className="text-sm text-gray-600 mb-4">{tab.description}</p>
      )}
      
      {tab.items.map((item) => (
        <div key={item.key}>
          {item.input_type === 'checkbox' ? (
            <label className="flex items-center space-x-3">
              <input
                type="checkbox"
                checked={settings[item.key] || false}
                onChange={(e) => onUpdate(item.key, e.target.checked)}
                className="w-5 h-5 text-orange-600 rounded"
              />
              <div>
                <span className="text-gray-700">{item.label}</span>
                {item.description && (
                  <p className="text-sm text-gray-500 mt-1">{item.description}</p>
                )}
              </div>
            </label>
          ) : item.input_type === 'text' ? (
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-2">
                {item.label}
              </label>
              <input
                type="text"
                value={settings[item.key] || ''}
                onChange={(e) => onUpdate(item.key, e.target.value)}
                className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-orange-500"
              />
              {item.description && (
                <p className="text-sm text-gray-500 mt-1">{item.description}</p>
              )}
            </div>
          ) : item.input_type === 'textarea' ? (
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-2">
                {item.label}
              </label>
              <textarea
                value={settings[item.key] || ''}
                onChange={(e) => onUpdate(item.key, e.target.value)}
                rows={4}
                className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-orange-500"
              />
              {item.description && (
                <p className="text-sm text-gray-500 mt-1">{item.description}</p>
              )}
            </div>
          ) : (
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-2">
                {item.label}
              </label>
              <input
                type={item.input_type}
                value={settings[item.key] || ''}
                onChange={(e) => onUpdate(item.key, e.target.value)}
                className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-orange-500"
              />
              {item.description && (
                <p className="text-sm text-gray-500 mt-1">{item.description}</p>
              )}
            </div>
          )}
        </div>
      ))}

      <button
        onClick={onSave}
        disabled={loading}
        className="px-4 py-2 bg-orange-600 text-white rounded hover:bg-orange-700 transition-colors disabled:bg-gray-400 disabled:cursor-not-allowed"
      >
        {loading ? 'Saving...' : 'Save Settings'}
      </button>
    </div>
  );
}
