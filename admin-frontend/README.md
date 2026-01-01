# RustPress Admin Frontend

This is the standalone admin panel frontend application for the RustPress blog system.

## Features

- **Settings Management**: Control external user registration and maintenance mode
- **User Management**: View user list, ban/unban users, reset user passwords
- **Post Management**: View all posts, delete posts
- **Plugin Management**: View plugin list, enable/disable plugins (reserved for future functionality)

## Development

```bash
# Install dependencies
npm install

# Start development server (port 5174)
npm run dev

# Build for production
npm run build

# Preview production build
npm run preview
```

## Configuration

The admin frontend runs on port `5174` by default. API requests are proxied to `http://localhost:3000`.

Modify `vite.config.ts` to change the port and proxy settings.

## Authentication

Only users with `Admin` or `Root` roles can access the admin panel. Non-admin users will be automatically logged out after login.
