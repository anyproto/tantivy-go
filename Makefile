all:
	@set -e;

build-verify:
	CGO_LDFLAGS='-static' GOOS="windows" GOARCH="amd64" CGO_ENABLED="1" CC="x86_64-w64-mingw32-gcc" go build -tags tantivylocal -v example/main.go
	CGO_LDFLAGS='-static' GOOS="linux" GOARCH="amd64" CGO_ENABLED="1" CC="x86_64-linux-musl-gcc" go build -tags tantivylocal -v example/main.go
	CGO_ENABLED="1" gomobile bind -tags tantivylocal -v -target=android -androidapi 26 -o lib.aar github.com/anyproto/tantivy-go/gomobile
	CGO_ENABLED="1" gomobile bind -tags tantivylocal -v -target=ios -o Lib.xcframework github.com/anyproto/tantivy-go/gomobile

build-local:
	@echo 'Building with local libraries...'
	go build -tags tantivylocal -v .

test:
	@echo 'Running tests with local libraries...'
	go test -tags tantivylocal ./...

test-remote:
	@echo 'Running tests without local libraries (simulating external usage)...'
	go test ./...

build:
	@echo 'Building without local libraries (simulating external usage)...'
	go build -v .