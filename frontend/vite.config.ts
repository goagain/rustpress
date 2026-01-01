import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react-swc'
import tailwindcss from '@tailwindcss/vite'

// https://vite.dev/config/
export default defineConfig({
  plugins: [
    react(), 
    tailwindcss(),
    // Redirect /admin to admin-frontend dev server
    {
      name: 'redirect-admin',
      configureServer(server) {
        server.middlewares.use((req, res, next) => {
          if (req.url?.startsWith('/admin')) {
            // Redirect to admin-frontend dev server
            // Remove /admin prefix and keep query string if exists
            const url = new URL(req.url, 'http://localhost');
            const targetPath = url.pathname.replace(/^\/admin/, '') || '/';
            const queryString = url.search || '';
            res.writeHead(302, {
              Location: `http://localhost:5174${targetPath}${queryString}`
            });
            res.end();
          } else {
            next();
          }
        });
      },
    },
  ],
  server: {
    port: 5173,
    proxy: {
      // Proxy API requests to Rust backend
      // All frontend routes are handled by Vite
      '/api': {
        target: 'http://localhost:3000',
        changeOrigin: true,
      },
    },
  },
  // @ts-ignore - Vitest types
  test: {
    globals: true,
    environment: 'jsdom',
    setupFiles: './src/test/setup.ts',
  },
})
