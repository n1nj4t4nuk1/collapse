.PHONY: api/build web/build aio/build cli/build \
       api/docker/build web/docker/build aio/docker/build

# Build targets
api/build:
	cargo build --release -p collapse-api

cli/build:
	cargo build --release -p collapse-cli

web/build:
	cd apps/web && npm ci && npm run build

aio/build: web/build
	cargo build --release -p collapse-aio

# Docker targets
api/docker/build:
	docker build -f apps/api/Dockerfile -t collapse-api .

web/docker/build:
	docker build -f apps/web/Dockerfile -t collapse-web .

aio/docker/build:
	docker build -f apps/aio/Dockerfile -t collapse-aio .
