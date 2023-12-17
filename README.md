# Forest Server

A live-updating development server for [forester](https://git.sr.ht/~jonsterling/ocaml-forester).

## Usage

To make use of the live-reload feature, you will need to add the following
javascript snippet to the root template in `forest.xsl`:

```html
...
<script type="module" src="forester.js"></script>
<script src="reload.js"></script>
```

Run `forest watch -- "$args"`, where `$args` are the arguments you want to pass
to `forester`. For example:

`forest wach -- "build --dev --root index trees/"`

## Installation and Setup

`cargo install forest-server`

Requires `forester` to be available in `$PATH`.

To install using Nix:

```nix
{
  inputs = {
    forester.url = "sourcehut:~jonsterling/ocaml-forester";
    forest-server.url = "github:kentookura/forest-server";
    forester.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = inputs@{ self, forester, forest-server, nixpkgs }:
    let
      system = "x86_64-linux"; # Only works on linux so far, PRs welcome!
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

## Why Rust?

`forester` is written in ocaml, so why rust?
[See this discussion](https://lists.sr.ht/~jonsterling/forester-discuss/%3CCADB3NkmpLxEpoTJqv7zNoh5s8+4cTVMJJt7sKR-EwHYc_ULSqw%40mail.gmail.com%3E#%3CCADB3NkkCBB7HKdM=1kzxtJRt9YBwkY_RNHhpzOTo8uuk7crC6A@mail.gmail.com%3E).
Additionally, I have found compiling ffi code really challenging on nixos
really challenging.

I am open to deprecating this program if a good ocaml solution is possible.
However, I suspect that `forest-server` is runnable on windows, so It would be
nice that when `forester` becomes windows-compatible, the native ocaml file
watching solution also works.

### TODO

- Add a nice overlay to the UI like in vite.

## Contributing

Contributions are welcome! Feel free to open a PR.
