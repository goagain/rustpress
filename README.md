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

- **API & Documentation**
  - RESTful API with OpenAPI/Swagger documentation
  - Comprehensive error handling
  - Request/response logging

### 🚧 Phase 2: Plugin System & AI Integration (Next)

The next phase will focus on:

- **Plugin Architecture**
  - Plugin API design and implementation
  - Hot-reloadable plugins
  - Plugin lifecycle management
  - Inter-plugin communication

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

7. **Access the application**
   - Frontend: http://localhost:5173
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
│   ├── auth/              # Authentication middleware
│   ├── dto/               # Data Transfer Objects
│   ├── entity/            # SeaORM entities
│   ├── repository/        # Data access layer
│   ├── storage/           # Storage backend abstraction
│   └── main.rs            # Application entry point
├── frontend/              # React frontend
│   └── src/
│       ├── components/    # React components
│       ├── services/      # API client
│       └── utils/         # Utility functions
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
