{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

    flake-parts = {
      url = "github:hercules-ci/flake-parts";
      inputs.nixpkgs-lib.follows = "nixpkgs";
    };

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = inputs:
    inputs.flake-parts.lib.mkFlake {inherit inputs;} {
      perSystem = {pkgs, system, ...}: {
        _module.args.pkgs = with inputs;
          import inputs.nixpkgs {
            inherit system;
            overlays = [
              (import rust-overlay)
            ];
          };

        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            rust-bin.stable.latest.default
          ];
        };
      };

      systems = inputs.nixpkgs.lib.systems.flakeExposed;
    };
}
