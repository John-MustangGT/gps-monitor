.PHONY: setup-gui
setup-gui: setup
	@echo "$(BLUE)Setting up GUI development dependencies...$(NC)"
	@echo "$(YELLOW)Installing X11/GUI system dependencies...$(NC)"
	@if command -v apt-get > /dev/null; then \
		sudo apt-get update && \
		sudo apt-get install -y libx11-dev libxrandr-dev libxinerama-dev libxcursor-dev libxi-dev libgl1-mesa-dev; \
	elif command -v yum > /dev/null; then \
		sudo yum install -y libX11-devel libXrandr-devel libXinerama-devel libXcursor-devel libXi-devel mesa-libGL-devel; \
	elif command -v pacman > /dev/null; then \
		sudo pacman -S --noconfirm libx11 libxrandr libxinerama libxcursor libxi mesa; \
	else \
		echo "$(YELLOW)Please install X11 development libraries for your distribution$(NC)"; \
	fi
	@echo "$(GREEN)GUI development environment setup complete!$(NC)"

.PHONY: display-check
display-check:
	@echo "$(BLUE)Checking display environment...$(NC)"
	@if [ -n "$DISPLAY" ]; then \
		echo "$(GREEN)DISPLAY is set: $DISPLAY$(NC)"; \
	else \
		echo "$(RED)DISPLAY is not set$(NC)"; \
	fi
	@if [ -n "$WAYLAND_DISPLAY" ]; then \
		echo "$(GREEN)WAYLAND_DISPLAY is set: $WAYLAND_DISPLAY$(NC)"; \
	else \
		echo "$(YELLOW)WAYLAND_DISPLAY is not set$(NC)"; \
	fi
	@if command -v xrandr > /dev/null; then \
		echo "$(YELLOW)Available displays:$(NC)"; \
		xrandr --query | head -5 || echo "$(RED)No displays found$(NC)"; \
	else \
		echo "$(YELLOW)xrandr not available$(NC)"; \
	fi# Makefile for GPS Monitor (Rust)
# Cross-platform GPS monitoring tool

# Project configuration
PROJECT_NAME := gps-monitor
BINARY_NAME := gps-monitor
VERSION := $(shell grep '^version' Cargo.toml | cut -d'"' -f2)

# Build configuration
CARGO := cargo
RELEASE_FLAGS := --release
DEBUG_FLAGS := 
GUI_FLAGS := --features gui
TARGET_DIR := target
RELEASE_DIR := $(TARGET_DIR)/release
DEBUG_DIR := $(TARGET_DIR)/debug

# Installation paths
PREFIX ?= /usr/local
BINDIR := $(PREFIX)/bin
MANDIR := $(PREFIX)/share/man/man1

# Cross-compilation targets
LINUX_TARGET := x86_64-unknown-linux-gnu
LINUX_MUSL_TARGET := x86_64-unknown-linux-musl
ARM64_TARGET := aarch64-unknown-linux-gnu
ARMV7_TARGET := armv7-unknown-linux-gnueabihf
WINDOWS_TARGET := x86_64-pc-windows-gnu

# Colors for output
RED := \033[0;31m
GREEN := \033[0;32m
YELLOW := \033[0;33m
BLUE := \033[0;34m
PURPLE := \033[0;35m
CYAN := \033[0;36m
NC := \033[0m # No Color

# Default target
.PHONY: all
all: build

# Help target
.PHONY: help
help:
	@echo "$(CYAN)GPS Monitor Makefile$(NC)"
	@echo "===================="
	@echo ""
	@echo "$(YELLOW)Main targets:$(NC)"
	@echo "  $(GREEN)build$(NC)           - Build debug version"
	@echo "  $(GREEN)release$(NC)         - Build release version"
	@echo "  $(GREEN)build-gui$(NC)       - Build with GUI support (Linux X11)"
	@echo "  $(GREEN)release-gui$(NC)     - Build release with GUI support"
	@echo "  $(GREEN)test$(NC)            - Run tests"
	@echo "  $(GREEN)install$(NC)         - Install to system (requires sudo)"
	@echo "  $(GREEN)uninstall$(NC)       - Remove from system (requires sudo)"
	@echo "  $(GREEN)clean$(NC)           - Clean build artifacts"
	@echo ""
	@echo "$(YELLOW)Development:$(NC)"
	@echo "  $(GREEN)check$(NC)           - Check code without building"
	@echo "  $(GREEN)clippy$(NC)          - Run clippy linter"
	@echo "  $(GREEN)fmt$(NC)             - Format code"
	@echo "  $(GREEN)fmt-check$(NC)       - Check code formatting"
	@echo "  $(GREEN)doc$(NC)             - Generate documentation"
	@echo "  $(GREEN)bench$(NC)           - Run benchmarks"
	@echo ""
	@echo "$(YELLOW)Cross-compilation:$(NC)"
	@echo "  $(GREEN)build-linux$(NC)     - Build for Linux x86_64"
	@echo "  $(GREEN)build-musl$(NC)      - Build for Linux x86_64 (musl)"
	@echo "  $(GREEN)build-arm64$(NC)     - Build for ARM64"
	@echo "  $(GREEN)build-armv7$(NC)     - Build for ARMv7"
	@echo "  $(GREEN)build-windows$(NC)   - Build for Windows"
	@echo "  $(GREEN)build-all$(NC)       - Build for all targets"
	@echo ""
	@echo "$(YELLOW)Distribution:$(NC)"
	@echo "  $(GREEN)package$(NC)         - Create release package"
	@echo "  $(GREEN)deb$(NC)             - Create Debian package"
	@echo "  $(GREEN)rpm$(NC)             - Create RPM package"
	@echo ""
	@echo "$(YELLOW)Utilities:$(NC)"
	@echo "  $(GREEN)deps$(NC)            - Install development dependencies"
	@echo "  $(GREEN)setup$(NC)           - Setup development environment"
	@echo "  $(GREEN)setup-gui$(NC)       - Setup GUI development dependencies"
	@echo "  $(GREEN)serial-check$(NC)    - Check for available serial ports"
	@echo "  $(GREEN)gpsd-check$(NC)      - Check if gpsd is running"
	@echo "  $(GREEN)display-check$(NC)   - Check X11/Wayland display availability"

# Build targets
.PHONY: build
build:
	@echo "$(BLUE)Building debug version...$(NC)"
	$(CARGO) build $(DEBUG_FLAGS)
	@echo "$(GREEN)Build complete: $(DEBUG_DIR)/$(BINARY_NAME)$(NC)"

.PHONY: release
release:
	@echo "$(BLUE)Building release version...$(NC)"
	$(CARGO) build $(RELEASE_FLAGS)
	@echo "$(GREEN)Release build complete: $(RELEASE_DIR)/$(BINARY_NAME)$(NC)"

.PHONY: build-gui
build-gui:
	@echo "$(BLUE)Building debug version with GUI support...$(NC)"
	$(CARGO) build $(DEBUG_FLAGS) $(GUI_FLAGS)
	@echo "$(GREEN)GUI build complete: $(DEBUG_DIR)/$(BINARY_NAME)$(NC)"

.PHONY: release-gui
release-gui:
	@echo "$(BLUE)Building release version with GUI support...$(NC)"
	$(CARGO) build $(RELEASE_FLAGS) $(GUI_FLAGS)
	@echo "$(GREEN)GUI release build complete: $(RELEASE_DIR)/$(BINARY_NAME)$(NC)"

# Test targets
.PHONY: test
test:
	@echo "$(BLUE)Running tests...$(NC)"
	$(CARGO) test

.PHONY: test-verbose
test-verbose:
	@echo "$(BLUE)Running tests (verbose)...$(NC)"
	$(CARGO) test -- --nocapture

# Development targets
.PHONY: check
check:
	@echo "$(BLUE)Checking code...$(NC)"
	$(CARGO) check

.PHONY: clippy
clippy:
	@echo "$(BLUE)Running clippy...$(NC)"
	$(CARGO) clippy -- -D warnings

.PHONY: fmt
fmt:
	@echo "$(BLUE)Formatting code...$(NC)"
	$(CARGO) fmt

.PHONY: fmt-check
fmt-check:
	@echo "$(BLUE)Checking code format...$(NC)"
	$(CARGO) fmt -- --check

.PHONY: doc
doc:
	@echo "$(BLUE)Generating documentation...$(NC)"
	$(CARGO) doc --no-deps --open

.PHONY: bench
bench:
	@echo "$(BLUE)Running benchmarks...$(NC)"
	$(CARGO) bench

# Cross-compilation targets
.PHONY: build-linux
build-linux:
	@echo "$(BLUE)Building for Linux x86_64...$(NC)"
	$(CARGO) build $(RELEASE_FLAGS) --target $(LINUX_TARGET)
	@echo "$(GREEN)Linux build complete: $(TARGET_DIR)/$(LINUX_TARGET)/release/$(BINARY_NAME)$(NC)"

.PHONY: build-musl
build-musl:
	@echo "$(BLUE)Building for Linux x86_64 (musl)...$(NC)"
	$(CARGO) build $(RELEASE_FLAGS) --target $(LINUX_MUSL_TARGET)
	@echo "$(GREEN)Musl build complete: $(TARGET_DIR)/$(LINUX_MUSL_TARGET)/release/$(BINARY_NAME)$(NC)"

.PHONY: build-arm64
build-arm64:
	@echo "$(BLUE)Building for ARM64...$(NC)"
	$(CARGO) build $(RELEASE_FLAGS) --target $(ARM64_TARGET)
	@echo "$(GREEN)ARM64 build complete: $(TARGET_DIR)/$(ARM64_TARGET)/release/$(BINARY_NAME)$(NC)"

.PHONY: build-armv7
build-armv7:
	@echo "$(BLUE)Building for ARMv7...$(NC)"
	$(CARGO) build $(RELEASE_FLAGS) --target $(ARMV7_TARGET)
	@echo "$(GREEN)ARMv7 build complete: $(TARGET_DIR)/$(ARMV7_TARGET)/release/$(BINARY_NAME)$(NC)"

.PHONY: build-windows
build-windows:
	@echo "$(BLUE)Building for Windows...$(NC)"
	$(CARGO) build $(RELEASE_FLAGS) --target $(WINDOWS_TARGET)
	@echo "$(GREEN)Windows build complete: $(TARGET_DIR)/$(WINDOWS_TARGET)/release/$(BINARY_NAME).exe$(NC)"

.PHONY: build-all
build-all: build-linux build-musl build-arm64 build-armv7 build-windows
	@echo "$(GREEN)All cross-compilation targets built successfully!$(NC)"

# Installation targets
.PHONY: install
install: release
	@echo "$(BLUE)Installing $(BINARY_NAME) to $(BINDIR)...$(NC)"
	install -d $(BINDIR)
	install -m 755 $(RELEASE_DIR)/$(BINARY_NAME) $(BINDIR)/
	@echo "$(GREEN)Installation complete!$(NC)"
	@echo "$(YELLOW)Run '$(BINARY_NAME) --help' to get started$(NC)"

.PHONY: uninstall
uninstall:
	@echo "$(BLUE)Removing $(BINARY_NAME) from $(BINDIR)...$(NC)"
	rm -f $(BINDIR)/$(BINARY_NAME)
	@echo "$(GREEN)Uninstallation complete!$(NC)"

# Package targets
.PHONY: package
package: release
	@echo "$(BLUE)Creating release package...$(NC)"
	mkdir -p dist
	tar -czf dist/$(PROJECT_NAME)-$(VERSION)-linux-x86_64.tar.gz \
		-C $(RELEASE_DIR) $(BINARY_NAME) \
		-C ../../ README.md LICENSE
	@echo "$(GREEN)Package created: dist/$(PROJECT_NAME)-$(VERSION)-linux-x86_64.tar.gz$(NC)"

.PHONY: deb
deb: release
	@echo "$(BLUE)Creating Debian package...$(NC)"
	@which fpm > /dev/null || (echo "$(RED)fpm not found. Install with: gem install fpm$(NC)" && exit 1)
	fpm -s dir -t deb \
		--name $(PROJECT_NAME) \
		--version $(VERSION) \
		--description "Cross-platform GPS monitoring tool" \
		--url "https://github.com/user/gps-monitor" \
		--maintainer "Your Name <your.email@example.com>" \
		--license "MIT" \
		--depends "libc6" \
		$(RELEASE_DIR)/$(BINARY_NAME)=$(BINDIR)/$(BINARY_NAME)
	@echo "$(GREEN)Debian package created!$(NC)"

.PHONY: rpm
rpm: release
	@echo "$(BLUE)Creating RPM package...$(NC)"
	@which fpm > /dev/null || (echo "$(RED)fpm not found. Install with: gem install fpm$(NC)" && exit 1)
	fpm -s dir -t rpm \
		--name $(PROJECT_NAME) \
		--version $(VERSION) \
		--description "Cross-platform GPS monitoring tool" \
		--url "https://github.com/user/gps-monitor" \
		--maintainer "Your Name <your.email@example.com>" \
		--license "MIT" \
		$(RELEASE_DIR)/$(BINARY_NAME)=$(BINDIR)/$(BINARY_NAME)
	@echo "$(GREEN)RPM package created!$(NC)"

# Utility targets
.PHONY: deps
deps:
	@echo "$(BLUE)Installing development dependencies...$(NC)"
	@echo "$(YELLOW)Installing Rust targets for cross-compilation...$(NC)"
	rustup target add $(LINUX_TARGET)
	rustup target add $(LINUX_MUSL_TARGET)
	rustup target add $(ARM64_TARGET)
	rustup target add $(ARMV7_TARGET)
	rustup target add $(WINDOWS_TARGET)
	@echo "$(YELLOW)Installing additional tools...$(NC)"
	cargo install cargo-watch || true
	cargo install cargo-audit || true
	cargo install cargo-outdated || true
	@echo "$(GREEN)Dependencies installed!$(NC)"

.PHONY: setup
setup: deps
	@echo "$(BLUE)Setting up development environment...$(NC)"
	@echo "$(YELLOW)Installing system dependencies (requires sudo)...$(NC)"
	@if command -v apt-get > /dev/null; then \
		sudo apt-get update && \
		sudo apt-get install -y build-essential pkg-config libudev-dev; \
	elif command -v yum > /dev/null; then \
		sudo yum groupinstall -y "Development Tools" && \
		sudo yum install -y pkgconfig systemd-devel; \
	elif command -v pacman > /dev/null; then \
		sudo pacman -S --noconfirm base-devel pkgconf systemd; \
	else \
		echo "$(YELLOW)Please install build tools for your distribution$(NC)"; \
	fi
	@echo "$(GREEN)Development environment setup complete!$(NC)"

.PHONY: serial-check
serial-check:
	@echo "$(BLUE)Checking for available serial ports...$(NC)"
	@if [ -d /dev ]; then \
		echo "$(YELLOW)Serial devices found:$(NC)"; \
		ls -la /dev/tty{USB,ACM,S}* 2>/dev/null || echo "$(RED)No serial devices found$(NC)"; \
	fi
	@if command -v dmesg > /dev/null; then \
		echo "$(YELLOW)Recent USB device messages:$(NC)"; \
		dmesg | grep -i "usb\|serial\|tty" | tail -5 || true; \
	fi

.PHONY: gpsd-check
gpsd-check:
	@echo "$(BLUE)Checking gpsd status...$(NC)"
	@if command -v gpsd > /dev/null; then \
		echo "$(GREEN)gpsd is installed$(NC)"; \
		if pgrep gpsd > /dev/null; then \
			echo "$(GREEN)gpsd is running$(NC)"; \
		else \
			echo "$(YELLOW)gpsd is not running$(NC)"; \
		fi; \
	else \
		echo "$(RED)gpsd is not installed$(NC)"; \
		echo "$(YELLOW)Install with:$(NC)"; \
		echo "  Ubuntu/Debian: sudo apt-get install gpsd gpsd-clients"; \
		echo "  CentOS/RHEL:   sudo yum install gpsd"; \
		echo "  Arch:          sudo pacman -S gpsd"; \
	fi
	@if command -v netstat > /dev/null; then \
		echo "$(YELLOW)Checking gpsd port (2947):$(NC)"; \
		netstat -ln | grep :2947 || echo "$(RED)gpsd port not listening$(NC)"; \
	fi

# Cleaning targets
.PHONY: clean
clean:
	@echo "$(BLUE)Cleaning build artifacts...$(NC)"
	$(CARGO) clean
	rm -rf dist/
	@echo "$(GREEN)Clean complete!$(NC)"

.PHONY: clean-all
clean-all: clean
	@echo "$(BLUE)Cleaning all artifacts including downloads...$(NC)"
	rm -rf ~/.cargo/registry/index/
	rm -rf ~/.cargo/git/
	@echo "$(GREEN)Deep clean complete!$(NC)"

# Development workflow targets
.PHONY: dev
dev:
	@echo "$(BLUE)Starting development mode (auto-rebuild on changes)...$(NC)"
	cargo watch -x 'build' -x 'test'

.PHONY: dev-run
dev-run:
	@echo "$(BLUE)Starting development mode with auto-run...$(NC)"
	cargo watch -x 'run -- --help'

# CI/CD targets
.PHONY: ci
ci: fmt-check clippy test
	@echo "$(GREEN)All CI checks passed!$(NC)"

.PHONY: audit
audit:
	@echo "$(BLUE)Running security audit...$(NC)"
	cargo audit

.PHONY: outdated
outdated:
	@echo "$(BLUE)Checking for outdated dependencies...$(NC)"
	cargo outdated

# Information targets
.PHONY: version
version:
	@echo "$(CYAN)GPS Monitor v$(VERSION)$(NC)"

.PHONY: info
info:
	@echo "$(CYAN)Project Information$(NC)"
	@echo "==================="
	@echo "Name:         $(PROJECT_NAME)"
	@echo "Version:      $(VERSION)"
	@echo "Binary:       $(BINARY_NAME)"
	@echo "Target Dir:   $(TARGET_DIR)"
	@echo "Install Dir:  $(BINDIR)"
	@echo ""
	@echo "$(CYAN)Build Status$(NC)"
	@echo "============"
	@if [ -f "$(RELEASE_DIR)/$(BINARY_NAME)" ]; then \
		echo "Release:      $(GREEN)Built$(NC)"; \
		ls -lh "$(RELEASE_DIR)/$(BINARY_NAME)"; \
	else \
		echo "Release:      $(RED)Not built$(NC)"; \
	fi
	@if [ -f "$(DEBUG_DIR)/$(BINARY_NAME)" ]; then \
		echo "Debug:        $(GREEN)Built$(NC)"; \
		ls -lh "$(DEBUG_DIR)/$(BINARY_NAME)"; \
	else \
		echo "Debug:        $(RED)Not built$(NC)"; \
	fi

# Make sure intermediate files aren't deleted
.PRECIOUS: $(TARGET_DIR)/%/release/$(BINARY_NAME)
.PRECIOUS: $(TARGET_DIR)/%/debug/$(BINARY_NAME)
