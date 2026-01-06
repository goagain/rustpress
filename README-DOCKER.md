# Docker Deployment Guide

## Quick Start

### Using Docker Compose

1. Create environment variables file (optional):
```bash
# .env
JWT_SECRET=your-secret-key-change-in-production
ROOT_PASSWORD=changeme
METRICS_USERNAME=admin
METRICS_PASSWORD=metrics_password
```

2. Start services:
```bash
docker-compose up -d
```

3. Access the application:
- Frontend: http://localhost:3000
- API Documentation: http://localhost:3000/swagger-ui
- PostgreSQL: localhost:5432

### Stop Services

```bash
docker-compose down
```

### View Logs

```bash
docker-compose logs -f app
docker-compose logs -f postgres
```

## Environment Variables

### Application Service (app)

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `DATABASE_URL` | ✅ | `postgres://postgres:password@postgres:5432/rustpress` | PostgreSQL connection string |
| `JWT_SECRET` | ✅ | `your-secret-key-change-in-production` | JWT secret key for authentication tokens |
| `ROOT_PASSWORD` | ❌ | `changeme` | Root user password (auto-generated if not set) |
| `STORAGE_DIR` | ❌ | `/app/uploads` | Directory for uploaded files |
| `STORAGE_BASE_URL` | ❌ | `http://localhost:3000/uploads` | Base URL for accessing uploaded files |
| `RUST_LOG` | ❌ | `info` | Log level (trace, debug, info, warn, error) |
| `METRICS_USERNAME` | ❌ | - | Username for Prometheus metrics basic authentication |
| `METRICS_PASSWORD` | ❌ | - | Password for Prometheus metrics basic authentication |

### PostgreSQL Service (postgres)

- `POSTGRES_USER`: Database user (default: `postgres`)
- `POSTGRES_PASSWORD`: Database password (default: `password`)
- `POSTGRES_DB`: Database name (default: `rustpress`)

## Manual Image Build

```bash
docker build -t rustpress:latest .
```

## Data Persistence

Docker Compose uses named volumes to persist data:

- `postgres_data`: PostgreSQL data
- `uploads_data`: Uploaded files

## GitHub Actions Configuration

### Required Secrets

Add the following secrets in your GitHub repository settings:

1. `DOCKER_TOKEN`: Docker registry access token
   - For GitHub Container Registry (ghcr.io): Use a Personal Access Token (PAT) with `write:packages` permission
   - For Docker Hub: Use a Docker Hub access token

2. `DOCKER_REGISTRY` (optional): Image registry address, defaults to `ghcr.io`
   - Examples: `ghcr.io`, `docker.io`, `registry.example.com`

3. `DOCKER_REPOSITORY` (optional): Image repository name, defaults to `github.repository`
   - Examples: `username/rustpress`, `myorg/rustpress`

### Automatic Push (main branch)

When code is pushed to the `main` branch, it will automatically build and push an image with the `latest` tag.

### Manual Version Push

1. Go to the GitHub Actions page
2. Select the "Build and Push Docker Image" workflow
3. Click "Run workflow"
4. Enter a version number (e.g., `v1.0.0` or `1.0.0`)
5. Click "Run workflow"

The image will be pushed to:
- `{IMAGE_NAME}:{VERSION}` (e.g., `ghcr.io/username/rustpress:1.0.0`)
- `{IMAGE_NAME}:latest`

### Using Pushed Images

```bash
# Pull latest version
docker pull ghcr.io/username/rustpress:latest

# Pull specific version
docker pull ghcr.io/username/rustpress:1.0.0

# Run container
docker run -d \
  -p 3000:3000 \
  -e DATABASE_URL=postgres://postgres:password@postgres:5432/rustpress \
  -e JWT_SECRET=your-secret-key \
  ghcr.io/username/rustpress:latest
```

## Troubleshooting

### Database Connection Failed

Ensure the PostgreSQL service is started and healthy:

```bash
docker-compose ps
docker-compose logs postgres
```

### Frontend Not Accessible

Check if frontend build files exist:

```bash
docker-compose exec app ls -la /app/frontend/dist
```

### Permission Issues

If you encounter permission issues, check file permissions:

```bash
docker-compose exec app ls -la /app/uploads
```
