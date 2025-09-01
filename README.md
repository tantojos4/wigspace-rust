# 🚀 Wigspace Rust: Nginx-like HTTP Server in Rust

[![Build Status](https://img.shields.io/badge/build-passing-brightgreen)](https://github.com/your/repo/actions)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

A high-performance, extensible, and production-ready HTTP server inspired by Nginx, built with Rust async ecosystem (tokio, hyper, etc).

---

## ✨ Features
- Asynchronous HTTP server (tokio, hyper)
- Configurable via YAML (address, port, static_dir, proxy, logging, etc)
- Modular handler system
- Nginx-style access & error logging (flexi_logger)
- Ready for static file serving, reverse proxy, routing, middleware, TLS, metrics, and more

---

## 🚦 Quickstart
```bash
# Clone & build
cargo build --release

# Edit config.yaml as needed

# Run
cargo run --release
```

---

## 🗺️ Roadmap

<details>
<summary><strong>Click to expand full roadmap</strong></summary>

### Stage 0: Testing & CI
- Unit/integration tests, CI pipeline, code coverage, fuzzing

### Stage 1: MVP Core
- Async HTTP server
- Config file loading (YAML/TOML/JSON)
- Basic request logging

### Stage 2: Static File Serving
- Serve static files from configurable directory
- Directory listing (optional)
- MIME type detection

### Stage 3: Reverse Proxy
- Proxy requests to backend servers (proxy_pass)
- Health checks, round-robin load balancing

### Stage 4: Routing & Modular Handlers
- Path-based routing (/static/, /api/)
- Modular handler system
- Middleware support (logging, auth, etc)

### Stage 4.5: Access Control & Authentication
- Middleware for JWT/API key/OAuth2
- Path-based access control
- Role-based authorization (optional)

### Stage 5: Hot Reload & Dynamic Config
- Reload config without downtime (SIGHUP/file watcher)
- Dynamic updates to routes/handlers

### Stage 6: Advanced Features
- TLS/HTTPS, gzip, caching, rate limiting, security filters

### Stage 7: Observability & Operations
- Access/error logging, Prometheus metrics, health endpoints

### Stage 7.8: Deployment & Containerization
- Dockerfile, Helm chart/Compose, env config, prod/dev separation, CI/CD

### Stage 8: Extensibility & Plugins
- Plugin system for custom handlers/middleware
- Dynamic module loading (WebAssembly)

#### Additional Suggestions
- Basic error handling (custom 404/500)
- WebSocket support
- Config validation
- CLI interface

</details>

---

## 📂 Project Structure
- `src/` — Main source code (modularized)
- `config.yaml` — Main configuration file
- `log/` — Log files (access, error)
- `.github/prompts/roadmap.prompt.md` — Full roadmap

---

## 🤝 Contributing
PRs, issues, and suggestions welcome! See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

---

## 📄 License
MIT
