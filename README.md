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


## � Roadmap Timeline

```
Stage 0: Testing & CI
	⬇️
Stage 1: MVP Core
	⬇️
Stage 2: Static File Serving
	⬇️
Stage 3: Reverse Proxy
	⬇️
Stage 4: Routing & Modular Handlers
	⬇️
Stage 4.5: Access Control & Authentication
	⬇️
Stage 5: Hot Reload & Dynamic Config
	⬇️
Stage 6: Advanced Features
	⬇️
Stage 7: Observability & Operations
	⬇️
Stage 7.8: Deployment & Containerization
	⬇️
Stage 8: Extensibility & Plugins
```

### 📌 Detail per Stage

- **Stage 0: Testing & CI** — Unit/integration tests, CI pipeline, code coverage, fuzzing
- **Stage 1: MVP Core** — Async HTTP server, config file loading, basic request logging
- **Stage 2: Static File Serving** — Serve static files, directory listing, MIME detection
- **Stage 3: Reverse Proxy** — Proxy_pass, health checks, load balancing
- **Stage 4: Routing & Modular Handlers** — Path-based routing, modular handler, middleware
- **Stage 4.5: Access Control & Authentication** — JWT/API key/OAuth2, access control, RBAC
- **Stage 5: Hot Reload & Dynamic Config** — Reload config tanpa downtime, dynamic routes
- **Stage 6: Advanced Features** — TLS, gzip, caching, rate limiting, security
- **Stage 7: Observability & Operations** — Logging, Prometheus metrics, health endpoints
- **Stage 7.8: Deployment & Containerization** — Docker, Helm, env config, prod/dev separation, CI/CD
- **Stage 8: Extensibility & Plugins** — Plugin system, WebAssembly

**Tambahan:**
- Basic error handling (custom 404/500)
- WebSocket support
- Config validation
- CLI interface

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
