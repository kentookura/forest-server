{
  inputs = {
    nixpkgs.url = "nixpkgs";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };

        libraries = with pkgs;[
          pkg-config
          bacon
          gdk-pixbuf
          openssl_3
        ];

        packages = with pkgs; [
        ];
      in
      {
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "forest-server";
          version = "0.2.1";
          src = ./.;
          cargoLock = { lockFile = ./Cargo.lock; };
          nativeBuildInputs = with pkgs;[ pkg-config ];
          buildInputs = libraries;
        };
        devShell = pkgs.mkShell {
          nativeBuildInputs = [ pkgs.pkg-config ];
          buildInputs = with pkgs;  libraries ++ [
            #packages.${system}.default
            (rust-bin.nightly.latest.default.override {
              targets = [ "wasm32-unknown-unknown" ];
              extensions = [ "rust-src" "rust-analyzer-preview" "rustfmt" ];
            })

          ];
        };
      });
}
