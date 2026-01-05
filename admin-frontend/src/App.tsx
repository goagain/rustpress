import { useState, useEffect } from 'react';
import { Outlet } from 'react-router-dom';
import { AdminPanel } from './components/Admin/AdminPanel';
import { LoginForm } from './components/Auth/LoginForm';
import { api, isAuthenticated, clearTokens } from './services/api';

export default function App() {
  const [authenticated, setAuthenticated] = useState(false);
  const [checking, setChecking] = useState(true);

  useEffect(() => {
    const checkAuth = async () => {
      if (isAuthenticated()) {
        try {
          const user = await api.getCurrentUser();
          if (user.role === 'Admin' || user.role === 'Root') {
            setAuthenticated(true);
          } else {
            clearTokens();
            setAuthenticated(false);
          }
        } catch (err) {
          clearTokens();
          setAuthenticated(false);
        }
      } else {
        setAuthenticated(false);
      }
      setChecking(false);
    };

    checkAuth();
    // Periodically check authentication state
    const interval = setInterval(checkAuth, 1000);
    return () => clearInterval(interval);
  }, []);

  const handleLoginSuccess = () => {
    setAuthenticated(true);
  };

  if (checking) {
    return (
      <div className="min-h-screen flex items-center justify-center bg-gray-50">
        <div className="text-gray-500">Checking authentication status...</div>
      </div>
    );
  }

  if (!authenticated) {
    return <LoginForm onSuccess={handleLoginSuccess} />;
  }

  return (
    <AdminPanel>
      <Outlet />
    </AdminPanel>
  );
}

// Wrapper component that provides authentication context
export function AuthenticatedApp() {
  return <App />;
}
