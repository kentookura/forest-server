{
  inputs = {
    opam-nix.url = "github:tweag/opam-nix";
    flake-utils.url = "github:numtide/flake-utils";
    nixpkgs.follows = "opam-nix/nixpkgs";
    rust-overlay.url = "github:oxalica/rust-overlay";
    opam-repository =
      {
        url = "github:ocaml/opam-repository";
        flake = false;
      };
    forester = {
      url = "sourcehut:~jonsterling/ocaml-forester";
      flake = false;
    };
    asai = {
      url = "github:RedPRL/asai";
      flake = false;
    };
  };
  outputs =
    { self
    , flake-utils
    , rust-overlay
    , opam-repository
    , opam-nix
    , nixpkgs
    , asai
    , forester
    }@inputs:
    # Don't forget to put the package name instead of `throw':
    flake-utils.lib.eachDefaultSystem
      (system:
      let
        my-python-packages = ps: with ps; [
          websockets
          # other python packages
        ];
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };
        on = opam-nix.lib.${system};
        overlay = final: prev:
          { };
        devPackagesQuery = {
          asai = "*";
        };
        query = devPackagesQuery // {
          ocaml-base-compiler = "*";
        };
        scope = on.buildOpamProject' { repos = [ "${opam-repository}" asai ]; } forester query;
        scope' = scope.overrideScope' overlay;
        main = scope'."forester";
        devPackages = builtins.attrValues
          (pkgs.lib.getAttrs (builtins.attrNames devPackagesQuery) scope');
      in
      {
        legacyPackages = scope.overrideScope' overlay;

        packages.default = self.legacyPackages.${system}."forester";
        devShell = with opam-nix.inputs.nixpkgs.legacyPackages.${system};
          mkShell {
            buildInputs = with pkgs; [
              (python3.withPackages my-python-packages)
              self.legacyPackages.${system}."forester"
              rust-analyzer-unwrapped
              (rust-bin.beta.latest.default.override { extensions = [ "rust-src" "rust-analyzer-preview" ]; })
            ];
          };
      });
}