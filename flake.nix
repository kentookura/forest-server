{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };
  outputs =
    { self
    , flake-utils
    , rust-overlay
    , nixpkgs
    }@inputs:
    # Don't forget to put the package name instead of `throw':
    flake-utils.lib.eachDefaultSystem
      (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };
      in
      with pkgs;
      {
        packages.default = pkgs.rustPlatform.buildRustPackage {

          pname = "forest-server";
          version = "0.0.1";
          src = ./.;
          cargoLock = { lockFile = ./Cargo.lock; };
          buildInputs = [ ];
        };
        devShell =
          pkgs.mkShell {
            buildInputs = with pkgs; [
              rust-analyzer-unwrapped
              (rust-bin.beta.latest.default.override { extensions = [ "rust-src" "rust-analyzer-preview" ]; })
            ];
          };
      });
}
