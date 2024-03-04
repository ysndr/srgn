{
  inputs.crane = {
    url = "github:ipetkov/crane";
    inputs.nixpkgs.follows = "nixpkgs";
  };

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
  inputs.utils.url = "github:numtide/flake-utils";

  outputs = {
    self,
    nixpkgs,
    utils,
    crane,
  }:
    {
      # Define an overlay for `srgn` and build the package
      # with the `crane` library.
      overlays.default = final: prev: let
        craneLib = crane.mkLib final;
        craneArgs = {
          src = craneLib.path ./.;
          strictDeps = false;
          buildInputs =
            []
            ++ final.lib.optionals final.stdenv.isDarwin [
              # Additional darwin specific inputs can be set here
              final.libiconv
            ];
        };
        # Build _just_ the cargo dependencies,
        # so only the top level binary needs to be rebuilt.
        cargoArtifacts = craneLib.buildDepsOnly craneArgs;

        # Build the actual crate itself, reusing the dependency
        # artifacts from above.
        srgn = craneLib.buildPackage (craneArgs
          // {
            inherit cargoArtifacts;
          });
      in {
        inherit srgn;
      };
    }
    # Add system-specific outputs to the flake,
    # i.e. `srgn` and a `devShell` for development
    # for linux and macos.
    // (
      utils.lib.eachDefaultSystem (system: let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [self.overlays.default];
        };
      in {
        packages = {
          srgn = pkgs.srgn;
          default = pkgs.srgn;
        };
        devShell = with pkgs;
          mkShell {
            inputsFrom = [srgn];
            buildInputs = [rustPackages.clippy];
            RUST_SRC_PATH = rustPlatform.rustLibSrc;
          };
      })
    );
}
