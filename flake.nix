{
  description = "Rummage - project finder";

  inputs = {
    flake-parts.url = "github:hercules-ci/flake-parts";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
  };

  outputs = inputs @ {
    flake-parts,
    self,
    fenix,
    ...
  }:
    flake-parts.lib.mkFlake {inherit inputs;} {
      imports = [];
      systems = ["x86_64-linux" "aarch64-linux" "aarch64-darwin" "x86_64-darwin"];
      flake = {
        homeManagerModules.rummage = ./nix/homeManager;
        homeManagerModules.default = self.homeManagerModules.rummage;
      };
      perSystem = {
        pkgs,
        system,
        ...
      }: {
        packages.default = let
          toolchain = fenix.packages.${system}.stable.toolchain;
        in
          (pkgs.makeRustPlatform {
            cargo = toolchain;
            rustc = toolchain;
          })
          .buildRustPackage {
            pname = "rummage";
            version = "0.1.0";

            src = ./.;

            cargoLock.lockFile = ./Cargo.lock;
          };
        devShells = let
          toolchain = with fenix.packages.${system};
            combine [
              stable.rustc
              stable.cargo
              targets.x86_64-unknown-linux-musl.stable.rust-std
              targets.aarch64-apple-darwin.stable.rust-std
              targets.x86_64-apple-darwin.stable.rust-std
            ];
        in {
          default = pkgs.mkShell {
            # The Nix packages provided in the environment
            packages = with pkgs; [
              git-cliff
              zig
              toolchain
              just
            ];
          };
        };
      };
    };
}
