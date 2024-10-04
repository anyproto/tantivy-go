{
  inputs.nixpkgs.url = "https://flakehub.com/f/NixOS/nixpkgs/0.1.0.tar.gz";
  inputs.rust-oxalica  = { url = "github:oxalica/rust-overlay"; };
  inputs.flake-utils.url = "github:numtide/flake-utils";

  outputs = { self, nixpkgs, rust-oxalica, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system: let
      musl = false;
      muslPkgsConf = {
        crossSystem = {
          config = "x86_64-unknown-linux-musl";
        };
      };
      pkgs = import nixpkgs ({
        inherit system;
        overlays = [ rust-oxalica.overlays.default ];
        config = { allowUnfree = true; };
      } // (if musl then muslPkgsConf else {}));
      tantivy-rust = pkgs.rust-bin.stable.latest.default.override {
        targets = if musl then [ "x86_64-unknown-linux-musl" ] else [];
        extensions = [ "rust-analyzer" "rust-src" ];
      };

      tantivy_compile_all_musl = pkgs.writeShellApplication {
        name = "tantivy_compile_all_musl";
        runtimeInputs = [ ];
        text = ''
         pushd .
         cd ./rust
         TARGET_CC=x86_64-linux-musl-gcc cargo build --release --target x86_64-unknown-linux-musl
         cp ./target/x86_64-unknown-linux-musl/release/libtantivy_go.a ../../anytype-heart/deps/libs/linux-amd64-musl/libtantivy_go.a
         cd ../../anytype-heart/
         rm -f linux-amd64
         GOEXPERIMENT=cgocheck2  GOOS="linux" GOARCH="amd64" CGO_ENABLED="1" go build -tags="envproduction nographviz nowatchdog nosigar nomutexdeadlockdetector noheic" -ldflags="-linkmode external -extldflags '-static -Wl,-z stack-size=1000000'" -o linux-amd64 github.com/anyproto/anytype-heart/cmd/grpcserver
         cp linux-amd64 ../anytype-ts/dist/anytypeHelper
         popd
        '';
      };

      tantivy_compile_all_gcc = pkgs.writeShellApplication {
        name = "tantivy_compile_all_gcc";
        runtimeInputs = [ ];
        text = ''
         pushd .
         cd ./rust
         cargo build --release
         cp ./target/release/libtantivy_go.a ../../anytype-heart/deps/libs/linux-amd64-musl/libtantivy_go.a
         cd ../../anytype-heart/
         rm -f linux-amd64
         go build -tags="envproduction nographviz nowatchdog nosigar nomutexdeadlockdetector noheic" -ldflags="-linkmode external" -o linux-amd64 github.com/anyproto/anytype-heart/cmd/grpcserver
         cp linux-amd64 ../anytype-ts/dist/anytypeHelper
         popd
        '';
      };

      devShell = pkgs.mkShell {
        name = "tantivy-go";
        nativeBuildInputs = with pkgs; [
          tantivy-rust
          go_1_22
        ] ++ (if musl then [ pkgs.muslCross tantivy_compile_all_musl] else [ tantivy_compile_all_gcc ]);
      };
    in {
      devShell = devShell;
    });
}
