.DEFAULT_GOAL := help

# Configuration variables
PORT    ?= 5100
CONFIG  ?= Debug

# Colors for terminal output
CYAN    := \033[36m
GREEN   := \033[32m
YELLOW  := \033[33m
RED     := \033[31m
RESET   := \033[0m

.PHONY: help all run-backend backend watch-backend build-backend test-backend test \
        run-desktop desktop build-desktop check-desktop release-desktop \
        build-mobile mobile test-mobile \
        docker-up docker-down docker-logs clean

help: ## Show this help message
	@echo -e "$(CYAN)========================================================$(RESET)"
	@echo -e "$(CYAN)       CONFER — Real-time Video Conferencing CLI        $(RESET)"
	@echo -e "$(CYAN)========================================================$(RESET)"
	@echo -e ""
	@echo -e "Usage: make $(GREEN)<target>$(RESET)"
	@echo -e ""
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[32m%-20s\033[0m %s\n", $$1, $$2}'
	@echo -e ""

all: build-backend build-desktop build-mobile ## Build all projects (backend, desktop, mobile)

# ==============================================================================
# .NET Core Backend (ASP.NET Core / Clean Architecture)
# ==============================================================================

run-backend: ## Run .NET Core backend API on http://localhost:5100
	@echo "$(CYAN)Starting Confer .NET Core Backend on port $(PORT)...$(RESET)"
	dotnet run --project src/api/Api.csproj --urls "http://localhost:$(PORT)"

backend: run-backend ## Alias for run-backend

watch-backend: ## Run .NET Core backend in watch mode (hot reload)
	@echo "$(CYAN)Starting Confer .NET Core Backend with Hot Reload...$(RESET)"
	dotnet watch --project src/api/Api.csproj

build-backend: ## Build the .NET Core solution
	@echo "$(CYAN)Building .NET solution Confer.slnx ($(CONFIG))...$(RESET)"
	dotnet build Confer.slnx -c $(CONFIG)

test-backend: ## Run all automated tests (.NET)
	@echo "$(CYAN)Running all unit and integration tests...$(RESET)"
	dotnet test Confer.slnx --logger "console;verbosity=normal"

test: test-backend ## Alias for test-backend

# ==============================================================================
# Native Rust Desktop Client (< 40 MB RAM / egui)
# ==============================================================================

run-desktop: ## Run the native Rust desktop client
	@echo "$(CYAN)Launching Confer Native Rust Desktop Client...$(RESET)"
	cargo run --manifest-path client-desktop/Cargo.toml

desktop: run-desktop ## Alias for run-desktop

build-desktop: ## Build the Rust desktop client (Debug)
	@echo "$(CYAN)Building native desktop client (Debug)...$(RESET)"
	cargo build --manifest-path client-desktop/Cargo.toml

check-desktop: ## Typecheck the native desktop client
	@echo "$(CYAN)Checking desktop client with cargo check...$(RESET)"
	cargo check --manifest-path client-desktop/Cargo.toml

release-desktop: ## Build optimized production binary for desktop client
	@echo "$(CYAN)Building optimized release desktop client...$(RESET)"
	cargo build --release --manifest-path client-desktop/Cargo.toml

# ==============================================================================
# Native Kotlin Mobile App (Android / Jetpack Compose)
# ==============================================================================

build-mobile: ## Build the Android debug APK (Kotlin Compose)
	@echo "$(CYAN)Building Android debug APK...$(RESET)"
	cd mobile && ./gradlew assembleDebug

mobile: build-mobile ## Alias for build-mobile

test-mobile: ## Run mobile unit tests
	@echo "$(CYAN)Running Kotlin mobile unit tests...$(RESET)"
	cd mobile && ./gradlew test

# ==============================================================================
# Docker & Infrastructure
# ==============================================================================

docker-up: ## Start PostgreSQL, Redis, Coturn and Confer API in Docker
	@echo "$(CYAN)Starting Docker services in background...$(RESET)"
	docker compose up -d

docker-down: ## Stop all Docker services
	@echo "$(YELLOW)Stopping all Docker services...$(RESET)"
	docker compose down

docker-logs: ## Follow logs from all Docker services
	docker compose logs -f

# ==============================================================================
# Packaging & Distribution
# ==============================================================================

package-deb: ## Build Debian package (.deb) for Linux desktop
	@echo "$(CYAN)Building .deb package for Confer Desktop...$(RESET)"
	./packaging/linux/build-deb.sh 1.0.0

package-tarball: ## Build standalone Linux tarball (.tar.gz) with installer
	@echo "$(CYAN)Building standalone Linux tarball...$(RESET)"
	./packaging/linux/build-tarball.sh 1.0.0

package-backend: ## Build self-contained single-file backend servers (Linux & Windows)
	@echo "$(CYAN)Building self-contained backend servers...$(RESET)"
	./packaging/build-backend.sh 1.0.0

package-all: package-deb package-tarball package-backend ## Build all release packages and installers

# ==============================================================================
# Cleanup
# ==============================================================================

clean: ## Clean all build artifacts across .NET, Rust, and Gradle
	@echo "$(YELLOW)Cleaning .NET bin and obj folders...$(RESET)"
	@find . -type d -name "bin" -o -name "obj" | xargs rm -rf 2>/dev/null || true
	@echo "$(YELLOW)Cleaning Rust target directory...$(RESET)"
	@rm -rf client-desktop/target 2>/dev/null || true
	@echo "$(YELLOW)Cleaning Mobile build directories...$(RESET)"
	@rm -rf mobile/app/build mobile/.gradle 2>/dev/null || true
	@rm -rf dist 2>/dev/null || true
	@echo "$(GREEN)Clean complete!$(RESET)"

