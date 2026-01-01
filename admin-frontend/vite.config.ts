import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react-swc'
import tailwindcss from '@tailwindcss/vite'

// https://vite.dev/config/
export default defineConfig(({ command, mode }) => {
  // In development, use '/' as base (runs on separate port)
  // In production build, use '/admin/' as base (served from backend)
  const base = command === 'serve' ? '/' : '/admin/';
  
  return {
    base,
    plugins: [react(), tailwindcss()],
    server: {
      port: 5174,
      proxy: {
        // Proxy API requests to Rust backend
        '/api': {
          target: 'http://localhost:3000',
          changeOrigin: true,
        },
      },
    },
    build: {
      outDir: 'dist',
      assetsDir: 'assets',
    },
  };
})
