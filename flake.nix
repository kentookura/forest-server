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
        frameworks = pkgs.darwin.apple_sdk.frameworks;
        libraries = with pkgs;[
          pkg-config
          bacon
          gdk-pixbuf
          openssl_3
          (if stdenv.hostPlatform.isDarwin then frameworks.CoreServices else null)
        ];

        packages = with pkgs; [
        ];
      in
      {
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "forest-server";
          version = "0.2.2";
          src = ./.;
          cargoLock = {
            lockFile = ./Cargo.lock;
            allowBuiltinFetchGit = true;
          };
          nativeBuildInputs = with pkgs;[ pkg-config ];
          buildInputs = libraries;
        };
        devShell = pkgs.mkShell {
          nativeBuildInputs = [ pkgs.pkg-config ];
          buildInputs = with pkgs;  libraries ++ [
            #packages.${system}.default
            (rust-bin.stable.latest.default.override {
              extensions = [ "rust-src" "rust-analyzer-preview" "rustfmt" ];
            })

          ];
        };
      });
}
