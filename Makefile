.PHONY: api/docker/build web/docker/build

api/docker/build:
	docker build -f apps/api/Dockerfile -t collapse-api .

web/docker/build:
	docker build -f apps/web/Dockerfile -t collapse-web .
