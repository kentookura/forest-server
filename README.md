# Forest Server

## Project Overview:

The project is a server that integrates with the [forester] tool, designed for crafting connected mathematical notes. It offers two binaries:

- forest-server: A simple dev server for forester
- forest-editor: An experimental desktop application.

## Installation:

To install using Nix:

```nix
{
  inputs = {
    forester.url = "sourcehut:~jonsterling/ocaml-forester";
    forest-server.url = "github:kentookura/forest-server";
    forester.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = inputs@{ self, forester, nixpkgs }:
    let
      system = "x86_64-linux"; # make sure to change this to your use case!
      pkgs = import nixpkgs { inherit system inputs; };
    in
    {
      devShells.${system}.default = pkgs.mkShell {
        buildInputs = [
          forester.packages.${system}.default
          forest-server.packages.${system}.default
        ];
      };
    };
}
```

TODO: provide builds on Github.

If you build the server from source, make sure forester is available on your
$PATH at runtime.

## Usage:

If you use the [forester-base-theme](https://git.sr.ht/~jonsterling/forester-base-theme), then run `forest-server` in that directory. Configuring folder locations and server port are not supported yet.

I plan on adding assets to `forest-server` so it can host a forest without
relying on an external folder.

## Configuration:

Configuration options will be available through command-line flags. Detailed configuration information will be added soon.

## Contributing

Contributions are welcome! Feel free to open a PR.
