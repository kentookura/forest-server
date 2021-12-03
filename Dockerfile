FROM nixpkgs/nix AS build

WORKDIR /src

# copy shell.nix and build environment first, so it will be cached
ADD shell.nix /src
RUN nix-shell --command "echo shell ready"

# copy source and execute go build with nix-shell
COPY . /src/
RUN nix-shell --command "make build"

FROM gcr.io/distroless/static AS final

# set user to nonroot user
USER nonroot:nonroot

# copy compiled app
COPY --from=build --chown=nonroot:nonroot /src/build/server /app

# run binary; use vector form
ENTRYPOINT ["/app"]
