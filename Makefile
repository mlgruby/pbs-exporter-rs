.PHONY: help build test check fmt clippy audit clean run docker-build docker-run docker-stop docker-clean all

# Default target
.DEFAULT_GOAL := help

# Variables
BINARY_NAME := pbs-exporter
DOCKER_IMAGE := pbs-exporter
DOCKER_TAG := latest

help: ## Show this help message
	@echo 'Usage: make [target]'
	@echo ''
	@echo 'Available targets:'
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z_-]+:.*?## / {printf "  %-20s %s\n", $$1, $$2}' $(MAKEFILE_LIST)

build: ## Build release binary
	cargo build --release

build-dev: ## Build debug binary
	cargo build

test: ## Run all tests
	cargo test --all-features

check: ## Check code compiles
	cargo check --all-features

fmt: ## Format code
	cargo fmt --all

fmt-check: ## Check code formatting
	cargo fmt --all -- --check

clippy: ## Run clippy linter
	cargo clippy --all-features -- -D warnings

audit: ## Run security audit
	cargo audit

clean: ## Clean build artifacts
	cargo clean
	rm -rf target/

run: ## Run the exporter (requires .env file)
	cargo run --release

run-dev: ## Run the exporter in debug mode
	cargo run

watch: ## Watch for changes and rebuild
	cargo watch -x run

# Docker targets
docker-build: ## Build Docker image
	docker build -t $(DOCKER_IMAGE):$(DOCKER_TAG) .

docker-run: ## Run Docker container
	docker run -d \
		--name $(BINARY_NAME) \
		-p 9101:9101 \
		--env-file .env \
		$(DOCKER_IMAGE):$(DOCKER_TAG)

docker-stop: ## Stop Docker container
	docker stop $(BINARY_NAME) || true
	docker rm $(BINARY_NAME) || true

docker-logs: ## Show Docker container logs
	docker logs -f $(BINARY_NAME)

docker-clean: docker-stop ## Clean Docker images
	docker rmi $(DOCKER_IMAGE):$(DOCKER_TAG) || true

docker-shell: ## Open shell in running container
	docker exec -it $(BINARY_NAME) /bin/sh

# Docker Compose targets
compose-up: ## Start services with docker-compose
	docker-compose up -d

compose-down: ## Stop services with docker-compose
	docker-compose down

compose-logs: ## Show docker-compose logs
	docker-compose logs -f

compose-restart: ## Restart docker-compose services
	docker-compose restart

# CI/CD simulation
ci: fmt-check clippy test audit build docker-build ## Run all CI checks locally

# Development workflow
dev: fmt clippy test ## Run development checks (format, lint, test)

all: clean ci ## Clean and run all checks

# Release preparation
release-check: ## Check if ready for release
	@echo "Checking version in Cargo.toml..."
	@grep '^version' Cargo.toml
	@echo ""
	@echo "Checking CHANGELOG.md..."
	@head -20 CHANGELOG.md
	@echo ""
	@echo "Running all checks..."
	@make ci
	@echo ""
	@echo "✓ Ready for release!"

# Metrics testing
test-metrics: ## Test metrics endpoint (requires running instance)
	@echo "Testing health endpoint..."
	@curl -f http://localhost:9101/health || echo "Health check failed"
	@echo ""
	@echo "Testing metrics endpoint..."
	@curl -s http://localhost:9101/metrics | head -20

# Documentation
docs: ## Generate and open documentation
	cargo doc --no-deps --open

docs-build: ## Build documentation
	cargo doc --no-deps
