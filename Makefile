.PHONY: api web aio cli \
       api/docker/build web/docker/build aio/docker/build

# Build targets
api:
	cargo build --release -p collapse-api

cli:
	cargo build --release -p collapse-cli

web:
	cd apps/web && npm ci && npm run build

aio: web
	cargo build --release -p collapse-aio

# Docker targets
api/docker/build:
	docker build -f apps/api/Dockerfile -t collapse-api .

web/docker/build:
	docker build -f apps/web/Dockerfile -t collapse-web .

aio/docker/build:
	docker build -f apps/aio/Dockerfile -t collapse-aio .
