{
  description = "A Nix(OS) Flake for todo-tree!";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    naersk.url = "github:nix-community/naersk";
  };

  outputs = { self, naersk, nixpkgs, rust-overlay, ... }:
    let
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
          f { pkgs = import nixpkgs { inherit overlays system; }; system = system; }
        );

      # Define the todo-tree package
      todoTreePackages = forEachSupportedSystem ({ pkgs, system }: let
        naersklib = pkgs.callPackage naersk {};
        package = naersklib.buildPackage {
          pname = "todo-tree";
          version = (builtins.fromTOML (builtins.readFile ./Cargo.toml)).package.version;
          src = self;
          nativeBuildInputs = with pkgs; [ pkg-config ];
          meta = { mainProgram = "todo-tree"; };
        };
      in { todo-tree = package; default = package; });

    in {
      # Development shells
      devShells = forEachSupportedSystem ({ pkgs }: {
        default = pkgs.mkShell {
          env.RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
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

      # Expose packages so they can be used in systemPackages
      packages = todoTreePackages;
    };
}
