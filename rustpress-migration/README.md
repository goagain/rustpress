# RustPress Migration Tool

A standalone database migration tool that can be run independently without starting the entire application.

## Building

```bash
cargo build --release --bin rustpress-migrate
```

The built executable will be located at: `target/release/rustpress-migrate` (Linux/macOS) or `target/release/rustpress-migrate.exe` (Windows)

## Usage

### Setting Database Connection

Via environment variable:
```bash
export DATABASE_URL="postgres://user:password@localhost:5432/rustpress"
```

Or via command-line argument:
```bash
rustpress-migrate --database-url "postgres://user:password@localhost:5432/rustpress" up
```

### Run All Pending Migrations

```bash
rustpress-migrate up
```

### Run a Specific Number of Migrations

```bash
rustpress-migrate up --steps 1
```

### Rollback Migrations

Rollback the last migration:
```bash
rustpress-migrate down
```

Rollback multiple migrations:
```bash
rustpress-migrate down --steps 2
```

### Check Migration Status

```bash
rustpress-migrate status
```

### Reset Database (Rollback All Migrations)

```bash
rustpress-migrate reset
```

### Refresh Database (Reset + Re-run All Migrations)

```bash
rustpress-migrate refresh
```

## Usage in CI/CD

```yaml
# GitHub Actions example
- name: Run migrations
  run: |
    cargo build --release --bin rustpress-migrate
    ./target/release/rustpress-migrate up
  env:
    DATABASE_URL: ${{ secrets.DATABASE_URL }}
```

## Usage in Docker

```dockerfile
# Build migration tool in Dockerfile
RUN cargo build --release --bin rustpress-migrate

# Run migrations before starting the application
RUN ./target/release/rustpress-migrate up
```
