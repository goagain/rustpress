import { useState, useEffect } from 'react';
import { api } from '../../../services/api';
import type { SettingsTab, AdminSettingsTabsResponse } from '../../../types';
import { getSettingsTabComponent } from './SettingsTabRegistry';

export function SettingsPanel() {
  const [tabs, setTabs] = useState<SettingsTab[]>([]);
  const [activeTabId, setActiveTabId] = useState<string>('');
  const [settings, setSettings] = useState<Record<string, any>>({});
  const [loading, setLoading] = useState(false);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    loadSettingsTabs();
  }, []);

  const loadSettingsTabs = async () => {
    try {
      setLoading(true);
      setError(null);
      const data: AdminSettingsTabsResponse = await api.getAdminSettingsTabs();
      setTabs(data.tabs);
      
      // Set active tab to first tab if available
      if (data.tabs.length > 0 && !activeTabId) {
        setActiveTabId(data.tabs[0].id);
      }

      // Build settings map from tabs
      const settingsMap: Record<string, any> = {};
      data.tabs.forEach((tab) => {
        tab.items.forEach((item) => {
          settingsMap[item.key] = item.value;
        });
      });
      setSettings(settingsMap);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load settings');
    } finally {
      setLoading(false);
    }
  };

  const handleUpdateSetting = (key: string, value: any) => {
    setSettings((prev) => ({
      ...prev,
      [key]: value,
    }));
  };

  const handleSaveSettings = async () => {
    try {
      setSaving(true);
      setError(null);
      await api.updateAdminSettings({ settings });
      await loadSettingsTabs();
      alert('Settings updated successfully');
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to update settings');
    } finally {
      setSaving(false);
    }
  };

  const activeTab = tabs.find((tab) => tab.id === activeTabId);
  const TabComponent = activeTab ? getSettingsTabComponent(activeTab.id) : null;

  if (loading) {
    return <div className="text-center py-8 text-gray-500">Loading settings...</div>;
  }

  return (
    <div>
      {error && (
        <div className="mb-4 p-3 bg-red-50 border border-red-200 text-red-700 rounded">
          {error}
        </div>
      )}

      {/* Settings Tabs Navigation */}
      {tabs.length > 0 && (
        <div className="mb-6 border-b border-gray-200">
          <div className="flex space-x-1">
            {tabs.map((tab) => (
              <button
                key={tab.id}
                onClick={() => setActiveTabId(tab.id)}
                className={`px-4 py-3 font-medium text-sm transition-colors ${
                  activeTabId === tab.id
                    ? 'text-orange-600 border-b-2 border-orange-600'
                    : 'text-gray-600 hover:text-gray-900'
                }`}
              >
                {tab.label}
              </button>
            ))}
          </div>
        </div>
      )}

      {/* Active Tab Content */}
      {activeTab && TabComponent && (
        <TabComponent
          tab={activeTab}
          settings={settings}
          onUpdate={handleUpdateSetting}
          onSave={handleSaveSettings}
          loading={saving}
        />
      )}

      {activeTab && !TabComponent && (
        <div className="bg-white rounded-lg shadow p-6">
          <p className="text-gray-500">
            Settings tab "{activeTab.label}" component not found. This may be a plugin tab that needs to be registered.
          </p>
        </div>
      )}
    </div>
  );
}
