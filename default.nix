{ pkgs, lib, rustPlatform }:

with rustPlatform;

buildRustPackage rec {
  pname = "gbtile";
  version = "0.2.0";
  src = builtins.filterSource
    (path: type: type != "directory" || baseNameOf path != "target")
    ./.;
  cargoSha256 = "sha256-k2abebOeoa6X9W95YN3WCnwDhU0JcYhQaypDX3QHq5M=";
  doCheck = false;

  meta = with lib; {
    description = "GameBoy tile generator. Converts PNG images to GBDK or RGDDS data";
    homepage = "https://github.com/blakesmith/gbtile";
    license = with licenses; [ mit ];
  };
}
