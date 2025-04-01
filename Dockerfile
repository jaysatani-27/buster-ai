# Build stage for the API
FROM lukemathwalker/cargo-chef AS api-builder
WORKDIR /app/api
COPY api/ .
RUN cargo install diesel_cli --no-default-features --features postgres
RUN cargo build --release --bin bi_api

# Build stage for the web app
FROM node:18 AS web-builder
WORKDIR /app/web
COPY web/ .
RUN npm ci
RUN npm run build
RUN npm prune --production

# Final stage
FROM debian:bookworm-slim
WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    postgresql-client \
    libpq-dev \
    nodejs \
    npm \
    && rm -rf /var/lib/apt/lists/*

# Copy built artifacts
COPY --from=api-builder /app/api/target/release/bi_api ./api/
COPY --from=api-builder /usr/local/cargo/bin/diesel /usr/local/bin/diesel
COPY --from=web-builder /app/web/.next ./web/.next
COPY --from=web-builder /app/web/public ./web/public
COPY --from=web-builder /app/web/package.json ./web/
COPY --from=web-builder /app/web/node_modules ./web/node_modules
COPY docker-compose.yml .
COPY api/migrations ./migrations/
COPY api/diesel.toml .

# Copy entrypoint script
COPY <<EOF /app/entrypoint.sh
#!/bin/bash
set -e

until pg_isready -h db -p 5432; do
  echo "Waiting for database to be ready..."
  sleep 2
done

export DATABASE_URL="postgresql://\${POSTGRES_USER:-postgres}:\${POSTGRES_PASSWORD:-your-super-secret-password}@db:5432/\${POSTGRES_DB:-buster}"

echo "Running diesel migrations..."
diesel migration run

echo "Starting services..."
cd web && npm start & cd .. && docker-compose up
EOF

RUN chmod +x /app/entrypoint.sh
ENTRYPOINT ["/app/entrypoint.sh"] 