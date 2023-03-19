{
  description = "GBTile GameBoy tile generator";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/22.05";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, flake-utils, nixpkgs }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
      in
        rec {
          packages.${system}.gbtile = pkgs.callPackage ./default.nix {};
          apps.${system}.gbtile = {
            type = "app";
            program = "${packages.${system}.gbtile}/bin/gbtile";
          };
          defaultPackage = packages.${system}.gbtile;
        }
    );
}
