# 🦀 Roadmap: Building an Nginx-like Modular HTTP Server in Rust

## Phase 1: Core Async HTTP Server
- [x] **Initialize async TCP/HTTP server (tokio, hyper)**
	- Setup `tokio::main` runtime, bind TCP listener, dan gunakan `hyper` untuk HTTP layer.
	- _Milestone_: Server bisa listen di port, siap menerima koneksi.
- [x] **Parse minimal HTTP requests (GET, POST, path)**
	- Ambil method (GET/POST) dan path dari objek request (`req.method()`, `req.uri()`).
	- _Milestone_: Bisa log dan routing berdasarkan method/path.
- [x] **Send basic HTTP responses (status, headers, body)**
	- Kirim response HTTP sederhana (status code, header minimal, body string).
	- _Milestone_: Client menerima response valid (misal: "Hello, World!").
- [x] **Handle multiple clients (tokio tasks)**
	- Setiap koneksi di-accept dan di-handle dalam task async terpisah (`tokio::task::spawn`).
	- _Milestone_: Server tetap responsif walau banyak client.
- [x] **Graceful shutdown (signal handling, cleanup)**
	- Tangani sinyal (Ctrl+C/SIGTERM) untuk shutdown bersih, pastikan resource dibersihkan.
	- _Milestone_: Server bisa dimatikan tanpa resource leak.

---


## Phase 2: Modular Architecture (Multi-Language Plugin System)
- [x] **Define trait-based handler/middleware API (Handler, Middleware traits)**
	- Trait `Handler` dan `Middleware` sudah diimplementasikan untuk extensibility.
	- _Milestone tercapai_: Handler dan middleware sudah plug-and-play, siap untuk extensi modular.

 - [x] **Support dynamic module loading: Multi-Language Plugins**
	 - **C ABI `.so` Modules**: Load C/C++/Go/Rust (with extern "C") dynamic libraries via FFI (`libloading`).
		 - _Milestone tercapai_: Server bisa load & invoke plugin C ABI `.so` secara dinamis (sudah teruji end-to-end).
	 - **Rust Dynamic Libraries**: Loader stub tersedia, siap diimplementasikan untuk plugin Rust native, idiomatik, dan aman.
		 - _Milestone_: Loader stub ada, tinggal implementasi real plugin Rust.
	 - **[NEW] Advanced Rust dylib plugin features: lifecycle, reload, sandboxing, etc.**
		 - _Milestone_: Implementasi lifecycle management (init, shutdown, reload), sandboxing, dan fitur advanced lain untuk plugin Rust dylib.
	 - **WASM Modules**: Loader stub tersedia, siap diintegrasikan dengan runtime WASM (`wasmtime`, `wasmer`).
		 - _Milestone_: Loader stub ada, integrasi runtime WASM berikutnya.
	 - **Scripting Modules**: Loader stub tersedia, siap dihubungkan ke engine scripting (Lua, JS, Python, dll).
		 - _Milestone_: Loader stub ada, tinggal integrasi engine scripting.

- [x] **Support static module registration (compile-time, crate features)**
	- Modul juga bisa di-link statis (fitur compile-time, crate features).
	- _Milestone_: Modul core selalu aktif tanpa dynamic loading.

- [x] **Example modules for each type**
	- Modul contoh: logging request (C ABI), custom response (Rust), WASM hello world, Lua/JS script handler.
	- _Milestone_: Ada minimal 1 modul eksternal untuk tiap tipe (C ABI, Rust, WASM, scripting).

- [x] **Module lifecycle management (init, shutdown, reload)**
	- Pastikan modul bisa di-init, shutdown, dan reload dengan aman (termasuk WASM/scripting sandboxing).
	- _Milestone_: Modul bisa diaktif/nonaktifkan tanpa crash.

---

### Plugin Type Comparison & Use-Cases

| Type                | Bahasa/Format         | Keunggulan                | Use-case utama                |
|---------------------|----------------------|---------------------------|-------------------------------|
| C ABI `.so`         | C, C++, Go, Rust     | Legacy, ekosistem luas    | Integrasi modul C, performa   |
| Rust Dynamic Lib    | Rust                 | Idiomatik, type-safe      | Plugin native, komunitas Rust |
| WASM                | Rust, Go, C, Zig, dsb| Sandbox, lintas bahasa    | Plugin eksternal, keamanan    |
| Scripting           | Lua, JS, Python, dsb | Dinamis, mudah diubah     | Modul cepat, scripting admin  |

---
## Phase 3: Configuration System
- [x] **Design config file format (YAML, TOML, JSON)**
	- Pilih format config yang mudah dibaca (YAML/TOML/JSON), dokumentasikan schema.
	- _Milestone_: Ada file config yang bisa di-edit user.
- [x] **Parse config at startup (address, port, modules, routes, etc)**
	- Baca config saat startup, gunakan untuk inisialisasi server.
	- _Milestone_: Server bisa diatur via config file.
- [ ] **Hot-reload config (notify crate, Arc<RwLock<Config>>)**
	- Implementasi reload config tanpa restart server (pakai crate `notify`, Arc<RwLock<Config>>).
	- _Milestone_: Config bisa diubah tanpa downtime.

	- [ ] **Auto-discovery custom modules directory for plugins (configurable via config.yaml)**
		- Implementasi agar direktori plugin bisa diatur di config.yaml dan modul-modul di-load otomatis saat startup.
		- _Milestone_: Server otomatis mendeteksi dan me-load plugin dari direktori custom.

- [ ] **Configurable plugin endpoint paths via config.yaml**
	- Mapping endpoint HTTP ke plugin/module bisa diatur di config.yaml, bukan hardcoded di main.rs.
	- _Milestone_: Endpoint plugin bisa diubah/ditambah lewat config tanpa rebuild.

---

## Phase 4: Routing & Request Handling
- [ ] **Implement path-based and/or regex routing (route-recognizer, matchit)**
	- Routing berdasarkan path atau regex, arahkan ke handler/modul sesuai pola URL.
	- _Milestone_: Bisa mapping /static/*, /api/*, dsb.
- [ ] **Route requests to modules or core handlers**
	- Request bisa diproses modul atau core handler sesuai routing.
	- _Milestone_: Handler modular, mudah extensi.
- [ ] **Support for static file serving (tokio::fs, mime_guess)**
	- Melayani file statis (HTML, CSS, dsb) dari folder tertentu, deteksi MIME.
	- _Milestone_: Bisa serve file statis dengan benar.
- [ ] **Query string and header parsing**
	- Parsing query string dan header HTTP untuk kebutuhan lanjutan (auth, dsb).
	- _Milestone_: Handler bisa akses query/header dengan mudah.

- [ ] **Dynamic routing for plugin endpoints from config**
	- Routing plugin sepenuhnya dinamis sesuai config, mendukung banyak plugin/endpoint tanpa hardcoded.
	- _Milestone_: Plugin endpoint bisa diatur/ditambah tanpa perubahan kode.

---

## Phase 5: Performance & Scalability
- [ ] **Non-blocking I/O (tokio, hyper)**
	- Pastikan semua I/O non-blocking, manfaatkan async Rust sepenuhnya.
	- _Milestone_: Server tetap responsif di beban tinggi.
- [ ] **Connection keep-alive**
	- Support HTTP keep-alive agar client bisa reuse koneksi.
	- _Milestone_: Koneksi tidak langsung close setelah 1 request.
- [ ] **Worker pool/multi-core scaling (tokio multi-threaded runtime)**
	- Gunakan runtime multi-threaded, scaling ke banyak core.
	- _Milestone_: Server bisa scaling di multi-core.
- [ ] **Benchmarking and profiling (criterion, tokio-console)**
	- Tambahkan tools untuk profiling dan benchmarking performa server.
	- _Milestone_: Ada data performa dan bottleneck.

---

## Phase 6: Logging & Monitoring
- [x] **Access and error logging (flexi_logger, tracing)**
	- Logging akses dan error ke file/stdout, format mirip Nginx.
	- _Milestone_: Semua request dan error tercatat rapi.
- [ ] **Module for custom logging (trait-based, pluggable)**
	- Logging bisa di-extensi via modul/trait custom.
	- _Milestone_: Bisa plug-in logger custom.
- [ ] **Basic metrics (requests/sec, errors, etc, prometheus_exporter)**
	- Statistik sederhana: request per detik, error count, dsb, expose ke Prometheus.
	- _Milestone_: Bisa monitoring traffic dan error.

---

## Phase 7: Security & Robustness
- [ ] **Input validation and sanitization**
	- Validasi input, sanitasi data untuk mencegah exploitasi.
	- _Milestone_: Tidak ada request aneh yang bisa crash server.
- [ ] **Limit request size, rate limiting (governor crate)**
	- Batasi ukuran request dan rate limit per client/IP.
	- _Milestone_: Server tahan DDoS/request abuse.
- [ ] **TLS/HTTPS support (rustls)**
	- Support HTTPS via rustls, config sertifikat.
	- _Milestone_: Server bisa serve HTTPS.
- [ ] **Sandbox modules (WASM, process isolation)**
	- Jalankan modul di sandbox/isolasi untuk keamanan ekstra.
	- _Milestone_: Modul eksternal tidak bisa mengganggu core.

---

## Phase 8: Advanced Features (Optional/Inspirational)
- [ ] **Reverse proxy support (hyper, reqwest)**
	- Fitur proxy ke backend lain, load balancing upstream.
	- _Milestone_: Bisa acting sebagai reverse proxy.
- [ ] **Caching (in-memory: dashmap, disk: tokio::fs)**
	- Cache response statis/dinamis untuk akselerasi.
	- _Milestone_: Response cepat untuk request berulang.
- [ ] **Gzip/deflate compression (flate2)**
	- Kompresi response HTTP (gzip/deflate).
	- _Milestone_: Bandwidth lebih hemat.
- [ ] **HTTP/2 support (hyper)**
	- Dukungan protokol HTTP/2.
	- _Milestone_: Client modern bisa pakai HTTP/2.
- [ ] **WebSocket support (tokio-tungstenite)**
	- Support WebSocket untuk aplikasi real-time.
	- _Milestone_: Bisa handle upgrade ke WebSocket.
- [ ] **Scripting module (WASM, Deno, Lua)**
	- Modul scripting (WASM, Deno, Lua) untuk extensibility dinamis.
	- _Milestone_: Handler bisa di-script tanpa rebuild.
- [ ] **Admin web UI (Yew, Leptos, Tauri)**
	- UI web untuk administrasi/server monitoring.
	- _Milestone_: Admin bisa manage server via web.

---


## Creative/Challenge Ideas
- [ ] **Hot-reload modules without downtime (dynamic reload, WASM, scripting)**
	- Reload modul (C ABI, Rust, WASM, scripting) tanpa restart server (zero downtime deploy).
- [ ] **Hybrid plugin system (WASM <-> native, scripting <-> native)**
	- Modul WASM bisa call native, scripting bisa akses API Rust/C, plugin hybrid.
- [ ] **Dynamic module marketplace/installer (crates.io, plugin registry)**
	- Marketplace modul, install/uninstall modul secara dinamis (semua tipe: C, Rust, WASM, scripting).
- [ ] **Visual module builder (web UI, DSL)**
	- Builder visual untuk membuat modul tanpa coding manual, output ke Rust, WASM, atau script.
- [ ] **Self-tuning performance engine (auto-tune thread pool, cache)**
	- Engine yang bisa auto-tune parameter performa.
- [ ] **AI-powered request routing (ML-based, traffic prediction)**
	- Routing request berbasis AI/ML untuk optimasi traffic.

---

## Tips for Success
- Keep the core async and modular
- Document trait/module API clearly
- Write tests for each phase (cargo test)
- Profile and optimize iteratively
- Encourage community crate/module development

---

**This roadmap is designed for stepwise, modular, and creative development in Rust. Setiap phase membangun fondasi kuat untuk server Nginx-like yang modern, scalable, dan extensible!**
