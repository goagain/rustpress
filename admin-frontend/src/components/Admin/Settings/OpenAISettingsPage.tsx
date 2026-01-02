import { useState, useEffect } from 'react';
import { api } from '../../../services/api';
import type { SettingsTab, AdminSettingsTabsResponse } from '../../../types';
import { getSettingsTabComponent } from '../Settings/SettingsTabRegistry';

export function OpenAISettingsPage() {
  const [tabs, setTabs] = useState<SettingsTab[]>([]);
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
      await api.updateAdminSettings({ settings });
      alert('Settings saved successfully');
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to save settings');
    } finally {
      setSaving(false);
    }
  };

  const openaiTab = tabs.find((tab) => tab.id === 'openai');
  const TabComponent = openaiTab ? getSettingsTabComponent(openaiTab.id) : null;

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

      {/* Active Tab Content */}
      {openaiTab && TabComponent && (
        <TabComponent
          tab={openaiTab}
          settings={settings}
          onUpdate={handleUpdateSetting}
          onSave={handleSaveSettings}
          loading={saving}
        />
      )}
    </div>
  );
}