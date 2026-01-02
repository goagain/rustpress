import { useState, useEffect } from 'react';
import { api } from '../../../../services/api';
import type { SettingsTabComponentProps } from '../SettingsTabRegistry';
import type { OpenAIApiKeyResponse } from '../../../../types';

export function OpenAISettingsTab({
  tab,
}: SettingsTabComponentProps) {
  const [apiKeys, setApiKeys] = useState<OpenAIApiKeyResponse[]>([]);
  const [loadingKeys, setLoadingKeys] = useState(false);
  const [showAddForm, setShowAddForm] = useState(false);
  const [newKeyName, setNewKeyName] = useState('');
  const [newKeyValue, setNewKeyValue] = useState('');
  const [newKeyEndpoint, setNewKeyEndpoint] = useState('https://api.openai.com/v1');
  const [editingKeyId, setEditingKeyId] = useState<number | null>(null);
  const [editKeyName, setEditKeyName] = useState('');
  const [editKeyValue, setEditKeyValue] = useState('');
  const [editKeyEndpoint, setEditKeyEndpoint] = useState('');
  const [editKeyDefaultModel, setEditKeyDefaultModel] = useState('');
  const [testingKeyId, setTestingKeyId] = useState<number | null>(null);
  const [selectedKeyId, setSelectedKeyId] = useState<number | null>(null);
  const [models, setModels] = useState<any[]>([]);
  const [loadingModels, setLoadingModels] = useState(false);

  useEffect(() => {
    loadApiKeys();
  }, []);

  const loadApiKeys = async () => {
    try {
      setLoadingKeys(true);
      const keys = await api.getOpenAIApiKeys();
      setApiKeys(keys);
    } catch (err) {
      console.error('Failed to load API keys:', err);
    } finally {
      setLoadingKeys(false);
    }
  };

  const handleAddKey = async () => {
    if (!newKeyName.trim() || !newKeyValue.trim()) {
      alert('Please enter both name and API key');
      return;
    }
    try {
      setLoadingKeys(true);
      await api.createOpenAIApiKey({
        name: newKeyName,
        api_key: newKeyValue,
        endpoint: newKeyEndpoint.trim() || undefined,
        is_default: apiKeys.length === 0,
      });
      setNewKeyName('');
      setNewKeyValue('');
      setNewKeyEndpoint('https://api.openai.com/v1');
      setShowAddForm(false);
      await loadApiKeys();
      alert('API key added successfully');
    } catch (err) {
      alert(`Failed to add API key: ${err instanceof Error ? err.message : 'Unknown error'}`);
    } finally {
      setLoadingKeys(false);
    }
  };

  const handleTestKey = async (keyId: number) => {
    try {
      setTestingKeyId(keyId);
      const result = await api.testOpenAIApiKey(keyId);
      if (result.success) {
        let message = 'API key test successful!';
        if (result.response_content) {
          message += `\n\nAI Response: ${result.response_content}`;
        }
        if (result.model_used) {
          message += `\n\nModel Used: ${result.model_used}`;
        }
        if (result.token_usage) {
          message += `\n\nToken Usage:`;
          message += `\n  Prompt: ${result.token_usage.prompt_tokens} tokens`;
          message += `\n  Completion: ${result.token_usage.completion_tokens} tokens`;
          message += `\n  Total: ${result.token_usage.total_tokens} tokens`;
        }
        alert(message);
      } else {
        alert(`API key test failed: ${result.message}`);
      }
    } catch (err) {
      alert(`Test failed: ${err instanceof Error ? err.message : 'Unknown error'}`);
    } finally {
      setTestingKeyId(null);
    }
  };

  const handleEditKey = (key: OpenAIApiKeyResponse) => {
    setEditingKeyId(key.id);
    setEditKeyName(key.name);
    setEditKeyValue(''); // Don't show existing key value for security
    setEditKeyEndpoint(key.endpoint || 'https://api.openai.com/v1');
    setEditKeyDefaultModel(key.default_model || '');
  };

  const handleCancelEdit = () => {
    setEditingKeyId(null);
    setEditKeyName('');
    setEditKeyValue('');
    setEditKeyEndpoint('');
    setEditKeyDefaultModel('');
  };

  const handleSaveEdit = async (keyId: number) => {
    console.log('handleSaveEdit called with keyId:', keyId);
    console.log('editKeyName:', editKeyName);
    console.log('editKeyEndpoint:', editKeyEndpoint);
    
    if (!editKeyName.trim()) {
      alert('Key name cannot be empty');
      return;
    }
    
    try {
      setLoadingKeys(true);
      const updateData: any = {
        name: editKeyName.trim(),
        // Always include endpoint in the update request
        endpoint: editKeyEndpoint.trim() || '',
      };

      // Only update API key if a new value was provided
      if (editKeyValue.trim()) {
        updateData.api_key = editKeyValue.trim();
      }

      // Handle default model - always include to allow setting to default
      if (editKeyDefaultModel.trim()) {
        updateData.default_model = editKeyDefaultModel.trim();
      } else {
        updateData.default_model = ''; // Empty string means use API default
      }
      
      console.log('Sending update request:', updateData);
      await api.updateOpenAIApiKey(keyId, updateData);
      await loadApiKeys();
      handleCancelEdit();
      alert('API key updated successfully');
    } catch (err) {
      console.error('Error updating API key:', err);
      alert(`Failed to update API key: ${err instanceof Error ? err.message : 'Unknown error'}`);
    } finally {
      setLoadingKeys(false);
    }
  };

  const handleDeleteKey = async (keyId: number) => {
    if (!confirm('Are you sure you want to delete this API key?')) return;
    try {
      setLoadingKeys(true);
      await api.deleteOpenAIApiKey(keyId);
      await loadApiKeys();
      alert('API key deleted successfully');
    } catch (err) {
      alert(`Failed to delete API key: ${err instanceof Error ? err.message : 'Unknown error'}`);
    } finally {
      setLoadingKeys(false);
    }
  };

  const handleLoadModels = async (keyId: number) => {
    try {
      setLoadingModels(true);
      setSelectedKeyId(keyId);
      const result = await api.listOpenAIModels(keyId);
      setModels(result.models);
    } catch (err) {
      alert(`Failed to load models: ${err instanceof Error ? err.message : 'Unknown error'}`);
    } finally {
      setLoadingModels(false);
    }
  };

  const handleSetDefaultModel = async (keyId: number, modelId: string) => {
    try {
      await api.setDefaultModel(keyId, { model_id: modelId });
      await loadApiKeys();
      alert('Default model set successfully');
    } catch (err) {
      alert(`Failed to set default model: ${err instanceof Error ? err.message : 'Unknown error'}`);
    }
  };

  return (
    <div className="bg-white rounded-lg shadow p-6 space-y-6">
      {tab.description && (
        <p className="text-sm text-gray-600 mb-4">{tab.description}</p>
      )}

      <div>
        <div className="flex justify-between items-center mb-4">
          <h3 className="text-lg font-semibold text-gray-900">OpenAI API Keys</h3>
          <button
            onClick={() => setShowAddForm(!showAddForm)}
            className="px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700 transition-colors"
          >
            {showAddForm ? 'Cancel' : 'Add API Key'}
          </button>
        </div>

        {showAddForm && (
          <div className="mb-6 p-4 border border-gray-200 rounded-lg space-y-4">
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-2">
                Key Name
              </label>
              <input
                type="text"
                value={newKeyName}
                onChange={(e) => setNewKeyName(e.target.value)}
                placeholder="e.g., Production Key"
                className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-orange-500"
              />
            </div>
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-2">
                API Key
              </label>
              <input
                type="password"
                value={newKeyValue}
                onChange={(e) => setNewKeyValue(e.target.value)}
                placeholder="sk-..."
                className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-orange-500"
              />
            </div>
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-2">
                API Endpoint
              </label>
              <input
                type="text"
                value={newKeyEndpoint}
                onChange={(e) => setNewKeyEndpoint(e.target.value)}
                placeholder="https://api.openai.com/v1"
                className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-orange-500"
              />
              <p className="mt-1 text-xs text-gray-500">
                Leave empty or use default for OpenAI. Use custom endpoint for compatible AI services.
              </p>
            </div>
            <button
              onClick={handleAddKey}
              disabled={loadingKeys}
              className="px-4 py-2 bg-green-600 text-white rounded hover:bg-green-700 transition-colors disabled:bg-gray-400"
            >
              {loadingKeys ? 'Adding...' : 'Add Key'}
            </button>
          </div>
        )}

        {loadingKeys ? (
          <div className="text-center py-4 text-gray-500">Loading API keys...</div>
        ) : apiKeys.length === 0 ? (
          <div className="text-center py-4 text-gray-500">No API keys configured</div>
        ) : (
          <div className="space-y-4">
            {apiKeys.map((key) => (
              <div key={key.id} className="border border-gray-200 rounded-lg p-4">
                {editingKeyId === key.id ? (
                  // Edit form
                  <div className="space-y-4">
                    <div>
                      <label className="block text-sm font-medium text-gray-700 mb-2">
                        Key Name
                      </label>
                      <input
                        type="text"
                        value={editKeyName}
                        onChange={(e) => setEditKeyName(e.target.value)}
                        className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-orange-500"
                      />
                    </div>
                    <div>
                      <label className="block text-sm font-medium text-gray-700 mb-2">
                        API Key (leave empty to keep current)
                      </label>
                      <input
                        type="password"
                        value={editKeyValue}
                        onChange={(e) => setEditKeyValue(e.target.value)}
                        placeholder="Enter new API key or leave empty"
                        className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-orange-500"
                      />
                    </div>
                    <div>
                      <label className="block text-sm font-medium text-gray-700 mb-2">
                        API Endpoint
                      </label>
                      <input
                        type="text"
                        value={editKeyEndpoint}
                        onChange={(e) => setEditKeyEndpoint(e.target.value)}
                        placeholder="https://api.openai.com/v1"
                        className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-orange-500"
                      />
                    </div>

                    <div>
                      <label className="block text-sm font-medium text-gray-700 mb-2">
                        Default Model
                      </label>
                      <select
                        value={editKeyDefaultModel || ''}
                        onChange={(e) => setEditKeyDefaultModel(e.target.value)}
                        className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-orange-500"
                      >
                        <option value="">Default (use API default)</option>
                        {models.map((model) => (
                          <option key={model.id} value={model.id}>
                            {model.id}
                          </option>
                        ))}
                      </select>
                      <p className="mt-1 text-sm text-gray-500">
                        Choose a specific model or leave as "Default" to use API's default model
                      </p>
                    </div>

                    <div className="flex space-x-2">
                      <button
                        type="button"
                        onClick={(e) => {
                          e.preventDefault();
                          handleSaveEdit(key.id);
                        }}
                        disabled={loadingKeys || !editKeyName.trim()}
                        className="px-4 py-2 bg-green-600 text-white rounded hover:bg-green-700 transition-colors disabled:bg-gray-400 disabled:cursor-not-allowed"
                      >
                        {loadingKeys ? 'Saving...' : 'Save'}
                      </button>
                      <button
                        type="button"
                        onClick={(e) => {
                          e.preventDefault();
                          handleCancelEdit();
                        }}
                        disabled={loadingKeys}
                        className="px-4 py-2 bg-gray-200 text-gray-700 rounded hover:bg-gray-300 transition-colors disabled:bg-gray-100"
                      >
                        Cancel
                      </button>
                    </div>
                  </div>
                ) : (
                  // Display mode
                  <>
                    <div className="flex justify-between items-start mb-2">
                      <div>
                        <h4 className="font-medium text-gray-900">{key.name}</h4>
                        <p className="text-sm text-gray-500">
                          {key.api_key_masked} {key.is_default && <span className="text-blue-600">(Default)</span>}
                        </p>
                        <p className="text-sm text-gray-500 mt-1">
                          Endpoint: <span className="font-mono text-xs">{key.endpoint || 'https://api.openai.com/v1 (default)'}</span>
                        </p>
                        {key.default_model && (
                          <p className="text-sm text-gray-500 mt-1">
                            Default Model: <span className="font-mono">{key.default_model}</span>
                          </p>
                        )}
                      </div>
                      <div className="flex space-x-2">
                        <button
                          onClick={() => handleEditKey(key)}
                          className="px-3 py-1 bg-yellow-100 text-yellow-800 rounded text-sm hover:bg-yellow-200"
                        >
                          Edit
                        </button>
                        <button
                          onClick={() => handleTestKey(key.id)}
                          disabled={testingKeyId === key.id}
                          className="px-3 py-1 bg-blue-100 text-blue-800 rounded text-sm hover:bg-blue-200 disabled:bg-gray-100"
                        >
                          {testingKeyId === key.id ? 'Testing...' : 'Test'}
                        </button>
                        <button
                          onClick={() => handleLoadModels(key.id)}
                          disabled={loadingModels && selectedKeyId === key.id}
                          className="px-3 py-1 bg-purple-100 text-purple-800 rounded text-sm hover:bg-purple-200 disabled:bg-gray-100"
                        >
                          {loadingModels && selectedKeyId === key.id ? 'Loading...' : 'Models'}
                        </button>
                        <button
                          onClick={() => handleDeleteKey(key.id)}
                          className="px-3 py-1 bg-red-100 text-red-800 rounded text-sm hover:bg-red-200"
                        >
                          Delete
                        </button>
                      </div>
                    </div>
                  </>
                )}

                {selectedKeyId === key.id && models.length > 0 && (
                  <div className="mt-4 pt-4 border-t border-gray-200">
                    <h5 className="font-medium text-gray-900 mb-2">Available Models</h5>
                    <div className="space-y-2 max-h-60 overflow-y-auto">
                      {models.map((model) => (
                        <div
                          key={model.id}
                          className="flex justify-between items-center p-2 hover:bg-gray-50 rounded"
                        >
                          <div>
                            <span className="font-mono text-sm">{model.id}</span>
                            <span className="text-xs text-gray-500 ml-2">({model.owned_by})</span>
                          </div>
                          <button
                            onClick={() => handleSetDefaultModel(key.id, model.id)}
                            className={`px-2 py-1 rounded text-xs ${
                              key.default_model === model.id
                                ? 'bg-green-100 text-green-800'
                                : 'bg-gray-100 text-gray-800 hover:bg-gray-200'
                            }`}
                          >
                            {key.default_model === model.id ? 'Current' : 'Set Default'}
                          </button>
                        </div>
                      ))}
                    </div>
                  </div>
                )}
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
