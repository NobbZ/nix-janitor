{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixpkgs-unstable";

    rust-overlay.url = "github:oxalica/rust-overlay";

    parts.url = "github:hercules-ci/flake-parts";
  };

  outputs = {
    self,
    parts,
    ...
  } @ inputs:
    parts.lib.mkFlake {inherit inputs;} {
      systems = ["x86_64-linux" "aarch64-linux"];

      perSystem = {
        self',
        inputs',
        pkgs,
        system,
        ...
      }: let
        pkgsWithOverlays = inputs.nixpkgs.legacyPackages.${system}.extend inputs.rust-overlay.overlays.default;
        rustVersion = "1.75.0";
        rust = pkgs.rust-bin.stable.${rustVersion}.default;

        rustc = rust;
        cargo = rust;

        rustPlatform = pkgs.makeRustPlatform {
          inherit rustc cargo;
        };
      in {
        _module.args.pkgs = pkgsWithOverlays;

        formatter = pkgs.alejandra;

        packages.janitor = rustPlatform.buildRustPackage {
          name = "janitor";
          version = "0.3.3-unstable-${self.rev or self.dirtyRev}";

          src = ./.;

          cargoLock.lockFile = ./Cargo.lock;
        };
        packages.default = self'.packages.janitor;

        devShells.default = pkgs.mkShell {
          packages = builtins.attrValues {
            inherit (pkgs) cargo-nextest cargo-audit cargo-deny cargo-tarpaulin rust-analyzer;
            inherit (pkgs) nil pre-commit;
            inherit rust;
          };

          env.RUST_SRC_PATH = "${rust}/lib/rustlib/src/rust/library";
        };
      };
    };
}
