.PHONY: cloud/docker/build web/docker/build

cloud/docker/build:
	docker build -f apps/cloud/Dockerfile -t collapse-cloud .

web/docker/build:
	docker build -f apps/web/Dockerfile -t collapse-web .
