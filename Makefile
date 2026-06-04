.PHONY: setup build clean dev stop kill-port

# Server port
PORT := 8089
PID_FILE := ~/.ntd/cloud-dev.pid
LOG_FILE := backend/dev.log

# Detect OS for port killing
_UNAME := $(shell uname -s)
_KILL_PORT = fuser -k $(PORT)/tcp 2>/dev/null || true
ifeq ($(_UNAME),Darwin)
  _KILL_PORT = lsof -ti:$(PORT) | xargs kill -9 2>/dev/null || true
endif

# Setup dependencies
setup:
	@echo "=== Setting up ntd-cloud ==="
	@echo ""
	@echo "[1/3] Checking Rust..."
	@which rustc > /dev/null 2>&1 || echo "  Install Rust: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
	@echo "  Rust: $$(rustc --version 2>/dev/null || echo 'NOT FOUND')"
	@echo ""
	@echo "[2/3] Checking Node.js..."
	@echo "  Node: $$(node --version 2>/dev/null || echo 'NOT FOUND')"
	@echo ""
	@echo "[3/3] Pre-compiling backend..."
	cd backend && cargo fetch

# Build frontend + backend
build:
	cd frontend && npm run build
	cd backend && cargo build --release

# Clean build artifacts
clean:
	rm -rf frontend/dist
	rm -rf backend/target
	rm -f $(LOG_FILE)

# Kill process on port
kill-port:
	@$(_KILL_PORT)

# Stop dev server
stop: kill-port
	-@if [ -f $(PID_FILE) ]; then \
		pid=$$(cat $(PID_FILE)); \
		kill -9 $$pid 2>/dev/null && echo "Killed $$pid" || echo "Process $$pid not found"; \
		rm -f $(PID_FILE); \
	fi
	@sleep 1

# Start dev server
dev: stop
	@echo "Starting ntd-cloud dev server on port $(PORT)..."
	cd backend && RUST_LOG=info cargo run 2>&1 | tee ../$(LOG_FILE) &
	@mkdir -p $$(dirname $(PID_FILE))
	@echo $$! > $(PID_FILE)
	@sleep 3
	@echo ""
	@echo "==========================================="
	@echo "  ntd-cloud running on http://localhost:$(PORT)"
	@echo "==========================================="
	@echo "Logs: tail -f $(LOG_FILE)"
	@echo "Stop:  make stop"
	@echo ""
