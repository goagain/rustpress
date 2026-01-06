import { createRoot } from 'react-dom/client';
import { BrowserRouter, Routes, Route, Navigate } from 'react-router-dom';
import { AuthenticatedApp } from './App';
import { GeneralSettingsPage } from './components/Admin/Settings/GeneralSettingsPage';
import { OpenAISettingsPage } from './components/Admin/Settings/OpenAISettingsPage';
import { UsersPage } from './components/Admin/UsersPage';
import { PostsPage } from './components/Admin/PostsPage';
import { PluginsPage } from './components/Admin/PluginsPage';
import './index.css';

const container = document.getElementById('admin-react-root');

if (!container) {
  throw new Error('Admin React root not found');
}

const root = createRoot(container);
root.render(
  <BrowserRouter basename="/admin">
    <Routes>
      <Route path="/" element={<AuthenticatedApp />}>
        <Route index element={<Navigate to="/settings/general" replace />} />
        <Route path="settings" element={<Navigate to="/settings/general" replace />} />
        <Route path="settings/general" element={<GeneralSettingsPage />} />
        <Route path="settings/openai" element={<OpenAISettingsPage />} />
        <Route path="users" element={<UsersPage />} />
        <Route path="posts" element={<PostsPage />} />
        <Route path="plugins" element={<PluginsPage />} />
      </Route>
    </Routes>
  </BrowserRouter>
);
