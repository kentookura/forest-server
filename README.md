# Forest Server

A live-updating development server for [forester](https://git.sr.ht/~jonsterling/ocaml-forester).

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

## Usage:

To make use of the live-reload feature, you will need to add the following
javascript snippet to the root template in `forest.xsl`:

```html
...
<script type="module" src="forester.js"></script>
<script>
  const evtSource = new EventSource("reload");
  evtSource.onmessage = (event) => {
    if (event.data == "reload") {
      location.reload();
    } else {
      console.log(event.data);
    }
  };
</script>
```

Run `forest-server` in the directory containing `trees/` and `theme/`

### Rough Edges

It is now possible to specify a tree directory using the `--dir` flag, but
the application still runs `forester` in the directory that `forest-server` is called in, so
it panics when there is no `theme` directory present.

### TODO

- Add a nice overlay to the UI like in vite.

## Contributing

Contributions are welcome! Feel free to open a PR.
