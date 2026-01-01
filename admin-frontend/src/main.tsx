import { createRoot } from 'react-dom/client';
import App from './App';
import './index.css';

const container = document.getElementById('admin-react-root');

if (!container) {
  throw new Error('Admin React root not found');
}

const root = createRoot(container);
root.render(<App />);
