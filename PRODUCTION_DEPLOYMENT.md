# Stellar Heads Production Deployment

## System Architecture

**Core Stack**: Bevy 0.16.1 + WASM32 target, Axum backend, PostgreSQL database, Yew frontend, Stellar blockchain integration

**Current Implementation Status**:
- Game engine: Bevy with Avian2D physics, WebGL rendering, PostMessage IPC
- Backend: Axum HTTP server with SQLx PostgreSQL integration
- Authentication: Guest registration with wallet address verification
- Deployment: Development servers on localhost:3000 (backend), localhost:8080 (frontend)

**Production Requirements**: JWT authentication system, managed database hosting, containerized deployment, SSL termination, CDN integration

---

## Development Workflow Integration

### Bevy CLI Integration

Install Bevy CLI for streamlined development workflow:

```bash
cargo install --git https://github.com/TheBevyFlock/bevy_cli --tag cli-v0.1.0-alpha.2 --locked bevy_cli
```

**Development Commands**:

```bash
# WASM Development Server (replaces manual wasm-pack + static server)
cd game
bevy run web --open
# Serves on http://localhost:8080 with hot reload
# Automatically compiles WASM target with optimal flags

# Code Quality Assurance
bevy lint
# Runs clippy with Bevy-specific lints, format checks

# Production WASM Build
bevy build --target wasm32-unknown-unknown --release
# Optimized build with size optimizations enabled
```

**Integration with Existing Build Pipeline**:

Update `yew-frontend/Trunk.toml` pre-build hooks:

```toml
[[hooks]]
stage = "pre_build"
command = "bevy"
command_arguments = ["build", "--target", "wasm32-unknown-unknown", "--release"]
working_directory = "../game"

[[hooks]]
stage = "pre_build"
command = "cp"
command_arguments = ["../game/target/wasm32-unknown-unknown/release/stellar_heads_game.wasm", "dist/game/stellar_heads_game_bg.wasm"]
```

**Asset Pipeline Optimization**:

Bevy CLI asset processing for production builds:

```bash
# Asset optimization for web deployment
bevy process-assets --target web --compress --optimize-images
```

**Docker Build Integration**:

```dockerfile
# Multi-stage build with Bevy CLI
FROM rust:1.75 as bevy-builder
RUN cargo install --git https://github.com/TheBevyFlock/bevy_cli --tag cli-v0.1.0-alpha.2 --locked bevy_cli

WORKDIR /app/game
COPY game/ .
RUN bevy build --target wasm32-unknown-unknown --release

FROM rust:1.75 as backend-builder
WORKDIR /app/backend
COPY backend/ .
RUN cargo build --release --bin backend

FROM debian:bookworm-slim
COPY --from=bevy-builder /app/game/target/wasm32-unknown-unknown/release/ /app/static/game/
COPY --from=backend-builder /app/backend/target/release/backend /usr/local/bin/backend
```

## Database Infrastructure

### PostgreSQL Hosting Options

**Managed Database Providers**:
- AWS RDS PostgreSQL: $15-50/month, automated backups, read replicas available
- DigitalOcean Managed Database: $15/month, automated failover, connection pooling
- Supabase: $25/month, includes authentication APIs, real-time subscriptions
- Railway: $5-20/month, simple deployment, automatic scaling

**Production Configuration**:

```env
DATABASE_URL=postgresql://username:password@host:5432/stellar_heads?sslmode=require
DATABASE_MAX_CONNECTIONS=20
DATABASE_MIN_CONNECTIONS=5
DATABASE_TIMEOUT=30
```

**Migration Deployment**:

```bash
# Production migration execution
sqlx migrate run --database-url $DATABASE_URL
sqlx migrate info --database-url $DATABASE_URL
```

**Security Configuration**:
- SSL/TLS encryption mandatory (`sslmode=require`)
- IP allowlist restricted to application server subnets
- Connection pooling with PgBouncer or built-in pooling
- Automated backup retention policy: 30 days point-in-time recovery

**Performance Optimization**:

```sql
-- Production database tuning
ALTER SYSTEM SET shared_buffers = '256MB';
ALTER SYSTEM SET effective_cache_size = '1GB';
ALTER SYSTEM SET maintenance_work_mem = '64MB';
ALTER SYSTEM SET checkpoint_completion_target = 0.9;
ALTER SYSTEM SET wal_buffers = '16MB';
```

## Authentication System Implementation

### JWT Token Management

**Dependencies** (`backend/Cargo.toml`):

```toml
[dependencies]
jsonwebtoken = "9.3"
chrono = { version = "0.4", features = ["serde"] }
tower-http = { version = "0.6", features = ["auth"] }
```

```rust
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use chrono::{Duration, Utc};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub user_id: i32,
    pub wallet_address: String,
    pub username: String,
    pub exp: usize,
    pub iat: usize,
}

pub fn generate_jwt_token(user_id: i32, wallet_address: String, username: String) -> Result<String, jsonwebtoken::errors::Error> {
    let expiration = Utc::now()
        .checked_add_signed(Duration::hours(24))
        .expect("valid timestamp")
        .timestamp() as usize;

    let claims = Claims {
        user_id,
        wallet_address,
        username,
        exp: expiration,
        iat: Utc::now().timestamp() as usize,
    };

    let secret = std::env::var("JWT_SECRET").expect("JWT_SECRET environment variable required");
    encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_ref()))
}

pub fn verify_jwt_token(token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let secret = std::env::var("JWT_SECRET").expect("JWT_SECRET environment variable required");
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_ref()),
        &Validation::default(),
    )
    .map(|token_data| token_data.claims)
}
```

**Authentication Middleware** (`backend/src/auth/middleware.rs`):

```rust
use axum::{
    extract::Request,
    http::{header::AUTHORIZATION, StatusCode},
    middleware::Next,
    response::Response,
};

pub async fn jwt_auth_middleware(
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let token = request.headers()
        .get(AUTHORIZATION)
        .and_then(|header| header.to_str().ok())
        .and_then(|header| header.strip_prefix("Bearer "));

    match token {
        Some(token) => {
            if let Ok(claims) = crate::auth::jwt::verify_jwt_token(token) {
                request.extensions_mut().insert(claims);
                Ok(next.run(request).await)
            } else {
                Err(StatusCode::UNAUTHORIZED)
            }
        }
        None => Err(StatusCode::UNAUTHORIZED)
    }
}
```

**Frontend Token Management** (`yew-frontend/src/auth/token.rs`):

```rust
use web_sys::window;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};

pub struct TokenManager;

impl TokenManager {
    pub fn store(token: &str) -> Result<(), String> {
        window()
            .ok_or("Window not available")?
            .local_storage()
            .map_err(|_| "LocalStorage not available")?
            .ok_or("LocalStorage not supported")?
            .set_item("stellar_heads_token", token)
            .map_err(|_| "Failed to store token")
    }

    pub fn retrieve() -> Option<String> {
        window()?
            .local_storage().ok()??
            .get_item("stellar_heads_token").ok()?
    }

    pub fn clear() {
        if let Some(storage) = window().and_then(|w| w.local_storage().ok()?) {
            let _ = storage.remove_item("stellar_heads_token");
        }
    }

    pub fn create_auth_headers() -> HeaderMap {
        let mut headers = HeaderMap::new();
        if let Some(token) = Self::retrieve() {
            if let Ok(header_value) = HeaderValue::from_str(&format!("Bearer {}", token)) {
                headers.insert(AUTHORIZATION, header_value);
            }
        }
        headers
    }
}
```

### Environment Configuration

**Runtime Variables** (`.env` production configuration):

```env
DATABASE_URL=postgresql://user:pass@host:5432/stellar_heads?sslmode=require
DATABASE_MAX_CONNECTIONS=20
DATABASE_MIN_CONNECTIONS=5
DATABASE_TIMEOUT=30

JWT_SECRET=256-bit-cryptographically-secure-random-key
JWT_EXPIRATION_HOURS=24

SERVER_HOST=0.0.0.0
SERVER_PORT=3000
RUST_LOG=info,backend=debug

STELLAR_NETWORK=PUBLIC
STELLAR_HORIZON_URL=https://horizon.stellar.org

CORS_ALLOWED_ORIGINS=https://stellarheads.com,https://api.stellarheads.com
RATE_LIMIT_REQUESTS_PER_SECOND=10
RATE_LIMIT_BURST_SIZE=20
```

**Containerization** (`Dockerfile`):

```dockerfile
FROM rust:1.75-slim as bevy-builder
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*
RUN cargo install --git https://github.com/TheBevyFlock/bevy_cli --tag cli-v0.1.0-alpha.2 --locked bevy_cli
WORKDIR /app/game
COPY game/ .
RUN bevy build --target wasm32-unknown-unknown --release

FROM rust:1.75-slim as backend-builder
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*
WORKDIR /app/backend
COPY backend/ .
RUN cargo build --release --bin backend

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates libssl3 && rm -rf /var/lib/apt/lists/*
COPY --from=bevy-builder /app/game/target/wasm32-unknown-unknown/release/ /app/static/game/
COPY --from=backend-builder /app/backend/target/release/backend /usr/local/bin/backend
EXPOSE 3000
USER 1001:1001
CMD ["backend"]
```

**Orchestration** (`docker-compose.yml`):

```yaml
version: '3.8'
services:
  backend:
    build: .
    ports:
      - "3000:3000"
    environment:
      DATABASE_URL: postgresql://postgres:${POSTGRES_PASSWORD}@db:5432/stellar_heads
      JWT_SECRET: ${JWT_SECRET}
    depends_on:
      db:
        condition: service_healthy
    restart: unless-stopped

  db:
    image: postgres:15-alpine
    environment:
      POSTGRES_DB: stellar_heads
      POSTGRES_PASSWORD: ${POSTGRES_PASSWORD}
    volumes:
      - postgres_data:/var/lib/postgresql/data
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U postgres"]
      interval: 10s
      timeout: 5s
      retries: 5

  nginx:
    image: nginx:alpine
    ports:
      - "80:80"
      - "443:443"
    volumes:
      - ./nginx.conf:/etc/nginx/nginx.conf
      - /etc/letsencrypt:/etc/letsencrypt
    depends_on:
      - backend

volumes:
  postgres_data:
```

## Deployment Strategies

### VPS Self-Hosting

**Infrastructure Requirements**:
- 2 vCPU, 4GB RAM, 50GB SSD minimum
- Ubuntu 22.04 LTS or Debian 12
- Docker Engine 24.0+, Docker Compose v2

**Setup Script**:

```bash
#!/bin/bash
# Production deployment script
set -euo pipefail

# System dependencies
apt-get update && apt-get install -y docker.io docker-compose-v2 nginx certbot python3-certbot-nginx

# SSL certificate generation
certbot --nginx -d stellarheads.com -d api.stellarheads.com --non-interactive --agree-tos -m admin@stellarheads.com

# Application deployment
git clone https://github.com/your-org/stellar-heads.git
cd stellar-heads
docker compose --profile production up -d

# Database initialization
docker compose exec backend sqlx migrate run
```

**Cost Analysis**:
- DigitalOcean Droplet (4GB): $24/month
- Managed PostgreSQL: $15/month
- Domain registration: $12/year
- Total: ~$40/month

### Platform-as-a-Service Options

**Railway Deployment**:

```toml
# railway.toml
[build]
builder = "DOCKERFILE"
buildCommand = "docker build -t stellar-heads ."

[deploy]
startCommand = "backend"
healthcheckPath = "/health"
healthcheckTimeout = 300

[[services]]
name = "backend"
source = "."

[[services]]
name = "postgres"
source = "postgres:15"
```

**Render Configuration** (`render.yaml`):

```yaml
services:
- type: web
  name: stellar-heads-backend
  env: docker
  dockerfilePath: ./Dockerfile
  envVars:
  - key: DATABASE_URL
    fromDatabase:
      name: stellar-heads-db
      property: connectionString
  - key: JWT_SECRET
    generateValue: true

databases:
- name: stellar-heads-db
  databaseName: stellar_heads
  user: stellar_heads
```

### Frontend Distribution

**Static Build Configuration**:

```bash
# Production build with asset optimization
cd yew-frontend
export TRUNK_PUBLIC_URL=https://cdn.stellarheads.com
trunk build --release --dist dist-prod

# Asset compression and CDN preparation
find dist-prod -name "*.js" -exec gzip -9 -k {} \;
find dist-prod -name "*.wasm" -exec gzip -9 -k {} \;
```

**CDN Integration** (CloudFront configuration):

```json
{
  "Origins": [
    {
      "DomainName": "stellarheads-assets.s3.amazonaws.com",
      "Id": "S3-stellar-heads-assets",
      "S3OriginConfig": {
        "OriginAccessIdentity": ""
      }
    }
  ],
  "DefaultCacheBehavior": {
    "TargetOriginId": "S3-stellar-heads-assets",
    "ViewerProtocolPolicy": "redirect-to-https",
    "Compress": true,
    "CachePolicyId": "4135ea2d-6df8-44a3-9df3-4b5a84be39ad"
  }
}
```

## Security Implementation

### Rate Limiting

**Dependencies** (`backend/Cargo.toml`):

```toml
[dependencies]
tower-governor = "0.4"
governor = "0.6"
```

**Implementation**:

```rust
use tower_governor::{governor::GovernorConfigBuilder, GovernorLayer, key_extractor::SmartIpKeyExtractor};
use std::time::Duration;

fn create_rate_limiter() -> GovernorLayer<SmartIpKeyExtractor> {
    let governor_config = GovernorConfigBuilder::default()
        .requests_per_period(NonZeroU32::new(100).unwrap())
        .period(Duration::from_secs(60))
        .burst_size(NonZeroU32::new(20).unwrap())
        .finish()
        .unwrap();

    GovernorLayer {
        config: Arc::new(governor_config),
    }
}
```

### Input Validation

**Validation Framework**:

```rust
use validator::{Validate, ValidationError};
use regex::Regex;

lazy_static::lazy_static! {
    static ref STELLAR_ADDRESS_REGEX: Regex = Regex::new(r"^G[0-9A-Z]{55}$").unwrap();
}

#[derive(Deserialize, Validate)]
pub struct StoreGameResultRequest {
    #[validate(length(min = 1, max = 50))]
    pub player_username: String,

    #[validate(regex = "STELLAR_ADDRESS_REGEX")]
    pub player_wallet_address: String,

    #[validate(range(min = 0, max = 1000))]
    pub player_score: u32,

    #[validate(range(min = 0, max = 1000))]
    pub opponent_score: u32,

    #[validate(range(min = 0.0, max = 7200.0))]
    pub duration_seconds: f32,
}
```

### Security Headers

```rust
use tower_http::set_header::SetResponseHeaderLayer;
use http::header::{STRICT_TRANSPORT_SECURITY, X_CONTENT_TYPE_OPTIONS, X_FRAME_OPTIONS};

let security_headers = Router::new()
    .layer(SetResponseHeaderLayer::overriding(
        STRICT_TRANSPORT_SECURITY,
        HeaderValue::from_static("max-age=31536000; includeSubDomains"),
    ))
    .layer(SetResponseHeaderLayer::overriding(
        X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    ))
    .layer(SetResponseHeaderLayer::overriding(
        X_FRAME_OPTIONS,
        HeaderValue::from_static("DENY"),
    ));
```

## Observability

### Metrics Collection

**Prometheus Integration**:

```rust
use axum_prometheus::PrometheusMetricLayer;
use prometheus::{IntCounterVec, HistogramVec, Opts, Registry};

fn setup_metrics() -> PrometheusMetricLayer<()> {
    let registry = Registry::new();

    let http_requests = IntCounterVec::new(
        Opts::new("http_requests_total", "Total HTTP requests"),
        &["method", "route", "status"]
    ).unwrap();

    let game_sessions = HistogramVec::new(
        prometheus::HistogramOpts::new("game_session_duration_seconds", "Game session duration"),
        &["result"]
    ).unwrap();

    registry.register(Box::new(http_requests.clone())).unwrap();
    registry.register(Box::new(game_sessions.clone())).unwrap();

    PrometheusMetricLayer::new()
}
```

### Structured Logging

```rust
use tracing::{info, warn, error, instrument};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[instrument(skip(pool))]
pub async fn store_game_result(
    State(pool): State<PgPool>,
    Json(payload): Json<StoreGameResultRequest>,
) -> Result<Json<StoreGameResultResponse>, StatusCode> {
    info!(
        player_address = %payload.player_wallet_address,
        player_score = payload.player_score,
        duration = payload.duration_seconds,
        "Processing game result"
    );

    // Implementation...
}
```

## Performance Optimization

### Database Connection Pooling

```rust
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::time::Duration;

pub async fn create_production_pool() -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(20)
        .min_connections(5)
        .acquire_timeout(Duration::from_secs(30))
        .idle_timeout(Duration::from_secs(600))
        .max_lifetime(Duration::from_secs(1800))
        .connect(&std::env::var("DATABASE_URL").expect("DATABASE_URL required"))
        .await
}
```

### WASM Optimization

**Optimization Flags** (`game/Cargo.toml`):

```toml
[profile.release]
opt-level = "s"
lto = true
codegen-units = 1
panic = "abort"

[profile.release.package."*"]
opt-level = "s"
```

**Build Script Integration**:

```bash
#!/bin/bash
# Optimized WASM build pipeline
set -euo pipefail

cd game

# Build with size optimizations
bevy build --target wasm32-unknown-unknown --release

# Post-process WASM
wasm-opt --enable-bulk-memory -Os -o pkg/stellar_heads_game_bg.wasm.opt pkg/stellar_heads_game_bg.wasm
mv pkg/stellar_heads_game_bg.wasm.opt pkg/stellar_heads_game_bg.wasm

# Generate compressed versions
gzip -9 -k pkg/stellar_heads_game_bg.wasm
brotli -q 11 -o pkg/stellar_heads_game_bg.wasm.br pkg/stellar_heads_game_bg.wasm
```

## Cost Analysis

### Infrastructure Pricing

**Development Environment**:
- Local development: $0/month
- Testing database (Supabase free tier): $0/month
- Total: $0/month

**Production MVP**:
- DigitalOcean Droplet (2 vCPU, 4GB): $24/month
- Managed PostgreSQL (1GB): $15/month
- Domain registration: $12/year ($1/month)
- SSL certificate (Let's Encrypt): $0/month
- Total: $40/month

**Scaling Infrastructure**:
- AWS ECS Fargate (0.5 vCPU, 1GB): $15/month
- RDS PostgreSQL (db.t3.micro): $25/month
- CloudFront CDN: $10/month
- Route 53 DNS: $1/month
- Application Load Balancer: $20/month
- Total: $71/month

**Enterprise Setup**:
- Multi-AZ RDS (db.r5.large): $200/month
- ECS with auto-scaling: $100-300/month
- ElastiCache Redis: $50/month
- CloudWatch monitoring: $30/month
- WAF protection: $50/month
- Total: $430-630/month

## Implementation Roadmap

### Phase 1: Authentication (Week 1-2)
1. Implement JWT token service with proper key rotation
2. Add authentication middleware with role-based access control
3. Update frontend authentication flow with token refresh
4. Implement logout and session management
5. Add password reset functionality for email users

### Phase 2: Database Migration (Week 2-3)
1. Provision managed PostgreSQL instance
2. Configure connection pooling and SSL
3. Execute schema migrations with rollback strategy
4. Implement database backup and recovery procedures
5. Performance testing and query optimization

### Phase 3: Production Deployment (Week 3-4)
1. Container image optimization and security scanning
2. Infrastructure-as-Code with Terraform/CDK
3. CI/CD pipeline with automated testing
4. Blue-green deployment strategy
5. Health checks and monitoring setup

### Phase 4: Optimization & Monitoring (Week 4-6)
1. Application performance monitoring integration
2. Error tracking and alerting systems
3. User analytics and game metrics collection
4. Load testing and capacity planning
5. Security audit and penetration testing

## Architecture Summary

**Current State**: Fully functional blockchain game with Bevy WASM engine, Axum backend, PostgreSQL database, and Stellar wallet integration. PostMessage IPC bridge enables secure communication between game iframe and parent application.

**Production Requirements**: JWT authentication system, managed database hosting, containerized deployment with orchestration, SSL termination, CDN integration, comprehensive monitoring and logging.

**Technology Stack**: Rust ecosystem (Bevy, Axum, SQLx, Yew), PostgreSQL database, Docker containerization, Stellar blockchain integration, WebAssembly compilation target.

**Deployment Target**: Cloud-native architecture with horizontal scaling capability, multi-region deployment potential, comprehensive observability, and enterprise-grade security implementations.