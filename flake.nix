{
  inputs = {
    nixpkgs.url = "nixpkgs";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
    forester.url = "sourcehut:~jonsterling/ocaml-forester";

  };

  outputs = { self, nixpkgs, rust-overlay, forester, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };

        libraries = with pkgs; [ pkg-config bacon gdk-pixbuf openssl_3 ];

        packages = with pkgs; [ ];
        forest-server = pkgs.rustPlatform.buildRustPackage {
          pname = "forest-server";
          version = "0.2.2";
          src = ./.;
          cargoLock = {
            lockFile = ./Cargo.lock;
            allowBuiltinFetchGit = true;
          };
          nativeBuildInputs = with pkgs; [ pkg-config ];
          buildInputs = libraries;
        };
      in {
        packages.default = forest-server;
        devShell = pkgs.mkShell {
          nativeBuildInputs = [ pkgs.pkg-config ];
          buildInputs = with pkgs;
            libraries ++ [
              hut
              forester.packages.${system}.default
              (rust-bin.stable.latest.default.override {
                extensions = [ "rust-src" "rust-analyzer-preview" "rustfmt" ];
              })

            ];
        };
      });
}
