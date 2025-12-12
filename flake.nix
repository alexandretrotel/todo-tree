{
  description = "A Nix(OS) Flake for todo-tree!";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    naersk.url = "github:nix-community/naersk";
  };

  outputs = {
    self,
    naersk,
    nixpkgs,
    rust-overlay,
    ...
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
      nixpkgs.lib.genAttrs supportedSystems (
        system:
          f {
            pkgs = import nixpkgs {inherit overlays system;};
            system = system;
          }
      );

    # Patch Cargo.toml content at evaluation time
    originalCargoToml = builtins.readFile ./Cargo.toml;
    patchedCargoToml =
      builtins.replaceStrings
      [''members = ["core", "cli", "extensions/zed"]'']
      [''members = ["core", "cli"]'']
      originalCargoToml;

    # Define the todo-tree package
    todoTreePackages = forEachSupportedSystem ({
      pkgs,
      system,
    }: let
      naersklib = pkgs.callPackage naersk {};

      # Filter source to exclude the zed extension (since it's a git submodule)
      filteredSrc = pkgs.lib.cleanSourceWith {
        src = self;
        filter = path: type:
          !(pkgs.lib.hasInfix "extensions/zed" path);
      };

      # Use naersk's postUnpack to patch Cargo.toml
      package = naersklib.buildPackage {
        name = "todo-tree";
        pname = "todo-tree";
        # Read version from cli/Cargo.toml since root is a workspace manifest
        version = (builtins.fromTOML (builtins.readFile ./cli/Cargo.toml)).package.version;
        src = filteredSrc;
        cargoBuildOptions = opts: opts ++ ["--package" "todo-tree"];
        nativeBuildInputs = with pkgs; [pkg-config];
        postUnpack = ''
                      cat > $sourceRoot/Cargo.toml << 'EOF'
          ${patchedCargoToml}
          EOF
        '';
        meta = {mainProgram = "todo-tree";};
      };
    in {
      todo-tree = package;
      default = package;
    });
  in {
    # Development shells
    devShells = forEachSupportedSystem ({
      pkgs,
      system,
    }: {
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
