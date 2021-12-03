.PHONY: build
build:
	CGO_ENABLED=0 go build -o build/server server.go

.PHONY: test
test:
	go test -v ./.

.PHONY: run
run: build
	./build/server

.PHONY: docker-build
docker-build:
	docker build -t app .