{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    nix-2-31-nixpkgs.url = "github:NixOS/nixpkgs/aec87d52f4d7576ec1e3587b52086701f071113c";
    systems.url = "github:nix-systems/default";
    flake-parts.url = "github:hercules-ci/flake-parts";
    flake-parts.inputs.nixpkgs-lib.follows = "nixpkgs";
    treefmt-nix.url = "github:numtide/treefmt-nix";
    treefmt-nix.inputs.nixpkgs.follows = "nixpkgs";
    rust-overlay.url = "github:oxalica/rust-overlay";
    rust-overlay.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs =
    {
      flake-parts,
      nixpkgs,
      rust-overlay,
      nix-2-31-nixpkgs,
      systems,
      ...
    }@inputs:
    flake-parts.lib.mkFlake { inherit inputs; } (
      { lib, moduleWithSystem, ... }:
      {
        imports = [
          inputs.treefmt-nix.flakeModule
        ];

        flake.nixosModules.default = moduleWithSystem (import ./modules/buildtime-secrets.nix);

        perSystem =
          { system, pkgs, ... }:
          let
            inherit (nix-2-31-nixpkgs.legacyPackages.${system}.nixVersions) nix_2_31;
          in
          {
            _module.args.pkgs = import nixpkgs {
              inherit system;
              overlays = lib.singleton rust-overlay.overlays.default;
            };

            devShells.default = pkgs.mkShell {
              packages = with pkgs; [
                rust-bin.nightly.latest.default
                pkg-config
                nix_2_31
                libarchive.dev
                nlohmann_json
                llvmPackages.clang
                boost.dev
                sops
              ];

              RUST_LOG = "debug";
            };

            treefmt.programs = {
              nixfmt.enable = true;
              rustfmt.enable = true;
              clang-format.enable = true;
            };

            packages = {
              default = pkgs.callPackage ./package.nix { inherit nix_2_31; };
              lib = pkgs.stdenv.mkDerivation {
                dontUnpack = true;
                dontBuild = true;

                passthru = {
                  fetchS3 = pkgs.callPackage ./fetch-s3.nix { };
                };
              };
            };
          };

        systems = import systems;
      }
    );
}
