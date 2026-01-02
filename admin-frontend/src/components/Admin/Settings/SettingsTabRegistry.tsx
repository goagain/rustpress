import type { SettingsTab } from '../../../types';
import { GeneralSettingsTab } from './tabs/GeneralSettingsTab';
import { OpenAISettingsTab } from './tabs/OpenAISettingsTab';

export interface SettingsTabComponentProps {
  tab: SettingsTab;
  settings: Record<string, any>;
  onUpdate: (key: string, value: any) => void;
  onSave: () => Promise<void>;
  loading?: boolean;
}

export interface SettingsTabComponent {
  id: string;
  component: React.ComponentType<SettingsTabComponentProps>;
}

// Registry of settings tab components
const settingsTabComponents: SettingsTabComponent[] = [
  {
    id: 'general',
    component: GeneralSettingsTab,
  },
  {
    id: 'openai',
    component: OpenAISettingsTab,
  },
];

/**
 * Get component for a settings tab
 */
export function getSettingsTabComponent(tabId: string): React.ComponentType<SettingsTabComponentProps> | null {
  const tabComponent = settingsTabComponents.find((tc) => tc.id === tabId);
  return tabComponent ? tabComponent.component : null;
}

/**
 * Register a new settings tab component (for plugin system)
 */
export function registerSettingsTab(tabComponent: SettingsTabComponent): void {
  const existingIndex = settingsTabComponents.findIndex((tc) => tc.id === tabComponent.id);
  if (existingIndex >= 0) {
    settingsTabComponents[existingIndex] = tabComponent;
  } else {
    settingsTabComponents.push(tabComponent);
  }
}

/**
 * Get all registered tab IDs
 */
export function getRegisteredTabIds(): string[] {
  return settingsTabComponents.map((tc) => tc.id);
}
