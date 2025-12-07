{
  description = "A Nix(OS) Flake for Todo-tree!";

  inputs = {
    # TODO: Periodically run `nix flake update` to update nix packages (such as rust version)
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    naersk.url = "github:nix-community/naersk";
  };

  outputs = {
    self,
    naersk,
    nixpkgs,
    rust-overlay,
  }: let
    overlays = [
      rust-overlay.overlays.default
      (final: prev: {
        rustToolchain = let
          rust = prev.rust-bin;
        in
          if builtins.pathExists ./rust-toolchain.toml
          then rust.fromRustupToolchainFile ./rust-toolchain.toml
          else if builtins.pathExists ./rust-toolchain
          then rust.fromRustupToolchainFile ./rust-toolchain
          else rust.stable.latest.default;
      })
    ];
    supportedSystems = ["x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin"];
    forEachSupportedSystem = f:
      nixpkgs.lib.genAttrs supportedSystems (system:
        f {
          pkgs = import nixpkgs {inherit overlays system;};
        });
  in {
    devShells = forEachSupportedSystem ({pkgs}: {
      default = pkgs.mkShell {
        env.RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
        # NOTE: These are general development tools/helpers. Add/remove as desired
        # or at least the ones
        packages = with pkgs; [
          rustToolchain
          pkg-config
          cargo-deny
          cargo-edit
          cargo-watch
          cargo-flamegraph
          rust-analyzer
          just
          bacon
        ];
      };
    });
    packages = forEachSupportedSystem ({pkgs}: let
      naersklib = pkgs.callPackage naersk {};
      todo-tree = naersklib.buildPackage {
        pname = "todo-tree";
        # TODO: When updating your tag, update this version field to match what is in Cargo.toml
        version = "0.1.0";
        src = self;
        nativeBuildInputs = with pkgs; [pkg-config];
        meta.mainProgram = "todo-tree";
      };
    in {
      # HACK: Lets you refer to the package with either `default` or `todo-tree`
      default = todo-tree;
    });
  };
}
