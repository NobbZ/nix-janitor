{
  inputs.nixpkgs.url = "github:nixos/nixpkgs?ref=nixpkgs-unstable";

  outputs = {
    self,
    nixpkgs,
    ...
  }: let
    pkgs = import nixpkgs {system = "x86_64-linux";};
    inherit (pkgs) glibcLocales;
    inherit (pkgs.stdenv) isLinux;
    inherit (pkgs.lib) optionalString;

    beamPkgs = with pkgs.beam_minimal; packagesWith interpreters.erlangR26;
    erlang = beamPkgs.erlang;
    elixir = beamPkgs.elixir_1_15;
  in {
    formatter.x86_64-linux = pkgs.alejandra;

    devShells.x86_64-linux.default = pkgs.mkShell {
      packages = builtins.attrValues {
        inherit (pkgs) nil dart pre-commit;
      };
    };
  };
}
