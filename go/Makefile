all:
	@set -e;

build-verify:
	CGO_LDFLAGS='-static' GOOS="windows" GOARCH="amd64" CGO_ENABLED="1" CC="x86_64-w64-mingw32-gcc" go build -v main.go
	CGO_LDFLAGS='-static' GOOS="linux" GOARCH="amd64" CGO_ENABLED="1" CC="x86_64-linux-musl-gcc" go build -v main.go
	//add 
	CGO_ENABLED="1" gomobile bind -v -target=android -androidapi 26 -o lib.aar github.com/anyproto/tantivy-go/go/tantivy/gomobile
	CGO_ENABLED="1" gomobile bind -v -target=ios -o Lib.xcframework github.com/anyproto/tantivy-go/go/tantivy/gomobile