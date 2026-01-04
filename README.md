# RustPress

A modern, pluggable Content Management System (CMS) built with Rust and React. RustPress is designed to be extensible, performant, and developer-friendly, with a focus on modularity and plugin architecture.

## 🎯 Project Vision

RustPress aims to be a fully pluggable CMS system that allows developers to extend functionality through a robust plugin ecosystem. The project is being developed in phases, with each phase building upon the previous foundation.

## 📋 Development Roadmap

### ✅ Phase 1: Core CMS Functionality (Current)

We are currently implementing the fundamental CMS features:

- **Content Management**
  - Blog post creation, editing, and deletion
  - Markdown support with syntax highlighting
  - Category organization
  - Draft system with auto-save
  - Version history and restoration

- **User Management**
  - JWT-based authentication
  - Role-based access control (Root, Admin, User)
  - User profile management

- **Media Management**
  - Image upload and storage
  - Flexible storage backend (currently local filesystem, extensible to S3)

- **Admin Panel**
  - Settings management (external registration, maintenance mode)
  - User management (view, ban/unban, reset password)
  - Post management (view all posts, delete any post)
  - Plugin management (enable/disable, reserved for future plugin system)

- **API & Documentation**
  - RESTful API with OpenAPI/Swagger documentation
  - Admin API endpoints at `/api/admin/*`
  - Comprehensive error handling
  - Request/response logging

### ✅ Phase 2: Plugin System & AI Integration (Current)

The current phase implements:

- **Plugin Architecture**
  - ✅ RPK package format (.rpk ZIP files)
  - ✅ TOML manifest with required/optional permissions
  - ✅ Permission-based access control with granular management
  - ✅ Hook system (action/filter pattern like WordPress)
  - ✅ Plugin lifecycle management (install, enable/disable)
  - ✅ Plugin manager with async hook execution
  - ✅ Permission denied error handling and graceful degradation
  - 🔄 Plugin execution runtime (WebAssembly integration planned)

- **AI Integration**
  - AI-powered content generation
  - Smart content suggestions
  - Automated tagging and categorization
  - Content optimization recommendations

### 🔮 Phase 3: Plugin Center & Admin Panel (Future)

Future enhancements include:

- **Plugin Center**
  - Plugin marketplace
  - Plugin discovery and installation
  - Version management
  - Community ratings and reviews

- **Admin Panel**
  - Comprehensive dashboard
  - System configuration
  - Plugin management interface
  - Analytics and reporting
  - User activity monitoring

- **Theme System**
  - Customizable frontend themes
  - Theme marketplace
  - Theme editor and customization tools
  - Live preview functionality
  - Responsive theme templates

## 🔌 Plugin System

RustPress features a comprehensive plugin system using RPK (RustPress Plugin) packages. RPK is a ZIP-based format containing plugin code, assets, and a TOML manifest with explicit permission declarations.

### RPK Package Format

RPK packages are ZIP files with `.rpk` extension containing:

```
plugin.rpk/
├── manifest.toml          # Plugin manifest (TOML format)
├── plugin.wasm           # WebAssembly plugin binary
├── frontend/             # Frontend assets (future use)
│   └── static/
├── admin_frontend/       # Admin assets (future use)
│   └── static/
└── assets/               # Plugin-specific assets
```

### Plugin Manifest (TOML)

```toml
[package]
id = "com.rui.editor"
name = "AI Editor Assistant"
version = "1.0.0"
description = "AI-powered content assistant for blog posts"
author = "Rui Team"

# Required permissions (automatically granted)
permissions = [
    "fs:read-assets"  # Must be able to read own static assets
]

# Optional permissions with descriptions
[optional_permissions]
"ai:summary" = "Generate AI-powered article summaries"
"net:unsplash" = "Search and insert images from Unsplash"
"post:read" = "Read existing posts for content analysis"
"post:write" = "Create and modify posts"

# Hooks this plugin registers
hooks = [
    "action_post_published",
    "filter_post_published"
]
```

### Permission System

Plugins use a two-tier permission system:

#### Required Permissions
- Automatically granted when plugin is enabled
- Cannot be revoked by administrators
- Essential for plugin core functionality
- Examples: `fs:read-assets`, `post:read`

#### Optional Permissions
- Must be explicitly granted by administrators
- Can be enabled/disabled per plugin
- For enhanced features that users may want to control
- Examples: `ai:summary`, `net:unsplash`

### Available Permissions

**Core Permissions:**
- `post:read` - Read posts and content
- `post:write` - Create/modify posts
- `user:read` - Read user information
- `user:write` - Create/modify users

**AI & External Services:**
- `ai:summary` - Use AI for content summarization
- `ai:chat` - General AI chat capabilities

**File System & Assets:**
- `fs:read-assets` - Read plugin's own assets
- `upload:read` - Read uploaded files
- `upload:write` - Upload files

**Network & External APIs:**
- `net:unsplash` - Access Unsplash image API

**System Settings:**
- `settings:read` - Read system configuration
- `settings:write` - Modify system settings

### Permission Inflation Detection

RustPress implements **Permission Inflation Detection** to prevent plugins from gaining unauthorized permissions during updates. This security mechanism:

1. **Compares Versions**: Analyzes permission differences between current and new versions
2. **Detects Inflation**: Identifies new required or optional permissions
3. **Requires Review**: Suspends plugin activation until administrator approval
4. **Maintains Security**: Ensures no permission creep without explicit consent

#### Update Flow

```
Plugin Update → Permission Analysis → Needs Review? → Admin Approval → Activation
     ↓                ↓                    ↓              ↓            ↓
  Upload RPK      Compare manifests    If new perms    Review UI    Enable plugin
```

#### Example Scenario

**Plugin v1.0.0** (currently installed):
```toml
permissions = ["post:read"]
```

**Plugin v2.0.0** (update):
```toml
permissions = ["post:read", "ai:summary"]  # NEW: ai:summary
[optional_permissions]
"ai:chat" = "Advanced AI features"  # NEW: optional permission
```

**Result**: Plugin status becomes `pending_review`, administrator must approve new permissions before plugin can be enabled.

#### Administrator Review Interface

When a plugin requires permission review, administrators see:
- ✅ Currently granted permissions (grayed out)
- ⚠️ New required permissions (must be approved)
- 🔄 New optional permissions (can be toggled)
- 📋 Permission descriptions and rationale

Only after explicit approval does the plugin become active with the new permissions.

### Hook System

Plugins can hook into system events using actions and filters:

#### Actions (Asynchronous, Non-blocking)
Actions are fired when events occur and don't expect return values. They're executed asynchronously and won't block the main flow.

- `action_post_published` - Fired when a post is published
- `action_user_created` - Fired when a user is created
- `action_user_login` - Fired when a user logs in

#### Filters (Synchronous, Can Modify Data)
Filters are fired during data processing and can modify the data being processed. They're executed synchronously.

- `filter_post_published` - Can modify post data during publishing
- `filter_user_created` - Can modify user data during creation
- `filter_authenticate` - Can modify authentication process

### Security Gatekeeper System

RustPress implements a **Host-Side Truth** security model to prevent data leakage vulnerabilities. The system validates plugin permissions against hook requirements at installation and load time.

#### Hook Permission Requirements

Each hook is explicitly defined with its permission requirements:

| Hook | Required Permission | Data Exposure | Description |
|------|-------------------|---------------|-------------|
| `action_post_published` | `post:read` | High | Receives full post content |
| `filter_post_published` | `post:write` | High | Can modify post data |
| `action_user_created` | `user:read` | Medium | Receives user information |
| `action_user_login` | `user:read` | Medium | Receives login events |
| `filter_authenticate` | `user:write` | High | Can modify authentication |
| `action_system_startup` | None | None | Pure notification, no data |
| `action_system_shutdown` | None | None | Pure notification, no data |

#### Security Validation Process

1. **Installation Time**: Plugin manifest is validated against hook registry
2. **Load Time**: Only hooks with proper permissions are registered
3. **Runtime**: Permission checks on every API call

**Example Security Violation Prevention:**

```toml
# Malicious plugin tries to register hook without permission
[package]
id = "evil.plugin"

# No post:read permission declared!
permissions = []

[optional_permissions]
# Even if optional, not granted

hooks = ["action_post_published"] # ❌ SECURITY VIOLATION
```

**Result**: Installation fails with `PluginSecurityViolation` error.

#### Plugin API with Security

Plugins communicate with the host through a permission-checked API:

```rust
// Plugin code (WebAssembly)
let host_api = PluginHostApi::new(plugin_manager, "myplugin".to_string());

// All calls automatically check permissions
match host_api.ai_chat(messages).await {
    Ok(result) => println!("AI response: {}", result),
    Err(PermissionDeniedError { permission, .. }) => {
        // Handle permission denial gracefully
        println!("AI features disabled (missing: {})", permission);
    }
}
```

#### Host API Methods

- `get_posts(query)` - Read posts (requires `post:read`)
- `save_post(post)` - Create/modify posts (requires `post:write`)
- `ai_chat(messages)` - Use AI features (requires `ai:*` permissions)
- `get_settings(keys)` - Read settings (requires `settings:read`)
- `update_settings(settings)` - Modify settings (requires `settings:write`)

### Installation & Management

#### Installing Plugins

Upload RPK files through the admin interface or API:

```bash
POST /api/admin/plugins
{
  "rpk_data": "base64-encoded-rpk-file",
  "permission_grants": {
    "ai:summary": true,
    "net:unsplash": false
  }
}
```

During installation:
1. RPK file is validated and extracted
2. Manifest is parsed and permissions initialized
3. Required permissions are automatically granted
4. Optional permissions use provided grants (default: false)

#### Permission Management

Administrators can manage optional permissions through the admin interface:

```bash
GET /api/admin/plugins/{plugin_id}/permissions
PUT /api/admin/plugins/{plugin_id}/permissions
{
  "permissions": {
    "ai:summary": true,
    "net:unsplash": true
  }
}
```

#### Runtime Behavior

Plugins handle permission denials gracefully:

```rust
// Plugin code example
match host_api.ai_chat(messages) {
    Ok(result) => {
        // Success: use AI features
        apply_summary(result);
    }
    Err(PermissionDeniedError { .. }) => {
        // Graceful degradation: skip AI features
        show_basic_interface();
    }
}
```

## 🛠️ Technology Stack

### Backend
- **Rust** - Core application language
- **Axum** - Web framework
- **SeaORM** - ORM for database operations
- **PostgreSQL** - Primary database
- **JWT** - Authentication tokens
- **Tokio** - Async runtime

### Frontend
- **React 19** - UI framework
- **TypeScript** - Type safety
- **Tailwind CSS** - Styling
- **Vite** - Build tool
- **React Markdown** - Markdown rendering

**Note**: The project uses two separate frontend applications:
- `frontend/` - Main user-facing blog interface
- `admin-frontend/` - Standalone admin panel for system administration

### DevOps
- **Docker** - Containerization
- **Docker Compose** - Multi-container orchestration
- **GitHub Actions** - CI/CD pipeline

## 🚀 Quick Start

### Prerequisites

- Rust 1.91+ ([Install Rust](https://www.rust-lang.org/tools/install))
- Node.js 20+ ([Install Node.js](https://nodejs.org/))
- PostgreSQL 16+ ([Install PostgreSQL](https://www.postgresql.org/download/))

### Local Development

1. **Clone the repository**
   ```bash
   git clone <repository-url>
   cd rustpress
   ```

2. **Set up the database**
   ```bash
   # Create a PostgreSQL database
   createdb rustpress
   ```

3. **Configure environment variables**
   ```bash
   # Create a .env file
   DATABASE_URL=postgres://postgres:password@localhost:5432/rustpress
   JWT_SECRET=your-secret-key-change-in-production
   ROOT_PASSWORD=changeme
   STORAGE_DIR=uploads
   STORAGE_BASE_URL=http://localhost:3000/uploads
   RUST_LOG=info
   ```

4. **Run database migrations**
   ```bash
   cd rustpress-migration
   cargo build --release --bin rustpress-migrate
   ./target/release/rustpress-migrate up
   ```

5. **Start the backend**
   ```bash
   cargo run
   ```

6. **Start the frontend** (in a new terminal)
   ```bash
   cd frontend
   npm install
   npm run dev
   ```

7. **Start the admin frontend** (optional, in another terminal)
   ```bash
   cd admin-frontend
   npm install
   npm run dev
   ```

8. **Access the application**
   - Main Frontend: http://localhost:5173
   - Admin Frontend: http://localhost:5174
   - API: http://localhost:3000
   - API Docs: http://localhost:3000/swagger-ui

### Docker Deployment

See [README-DOCKER.md](./README-DOCKER.md) for detailed Docker deployment instructions.

```bash
docker-compose up -d
```

## 📁 Project Structure

```
rustpress/
├── src/                    # Rust backend source code
│   ├── api/               # API controllers and routes
│   │   └── admin_controller.rs  # Admin API endpoints
│   ├── auth/              # Authentication middleware
│   ├── dto/               # Data Transfer Objects
│   │   └── admin.rs       # Admin DTOs
│   ├── entity/            # SeaORM entities
│   ├── repository/        # Data access layer
│   ├── storage/           # Storage backend abstraction
│   └── main.rs            # Application entry point
├── frontend/              # Main React frontend (user-facing)
│   └── src/
│       ├── components/    # React components
│       ├── services/      # API client
│       └── utils/         # Utility functions
├── admin-frontend/        # Admin React frontend (separate project)
│   └── src/
│       ├── components/    # Admin components
│       └── services/      # Admin API client
├── rustpress-migration/   # Standalone migration tool
│   └── src/
│       ├── lib.rs        # Migration library
│       └── main.rs       # Migration CLI tool
├── Dockerfile            # Docker build configuration
└── docker-compose.yml   # Docker Compose configuration
```

## 🔐 Authentication

RustPress uses JWT-based authentication with role-based access control:

- **Root** - Full system access
- **Admin** - Administrative privileges
- **User** - Standard user access

Default root user is created on first startup. Set `ROOT_PASSWORD` environment variable to customize the password.

## 📚 API Documentation

Interactive API documentation is available at `/swagger-ui` when the server is running. The OpenAPI specification can be accessed at `/api-doc/openapi.json`.

## 🧪 Development

### Running Tests

```bash
# Backend tests
cargo test

# Frontend tests
cd frontend
npm test
```

### Database Migrations

The project includes a standalone migration tool. See [rustpress-migration/README.md](./rustpress-migration/README.md) for details.

```bash
cd rustpress-migration
cargo build --release --bin rustpress-migrate
./target/release/rustpress-migrate status
./target/release/rustpress-migrate up
```

## 🤝 Contributing

Contributions are welcome! As this project is in active development, please check the roadmap to see what's being worked on. For major changes, please open an issue first to discuss what you would like to change.

## 📝 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

**Why MIT License?**
- Allows commercial use and evaluation, making it easy for potential employers to assess your code
- Simple and permissive, encouraging adoption and contributions
- Widely recognized and trusted in the industry
- Perfect for showcasing your work and building your professional reputation

## 🙏 Acknowledgments

Built with ❤️ using Rust and React.

---

**Note**: This project is under active development. Features and APIs may change without notice. See the roadmap above for planned features and current development status.
