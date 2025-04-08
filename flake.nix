{
  description = "A collection i3blocks for icemau";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
  };

  outputs = { self, nixpkgs }:
    let
      system = "x86_64-linux";
      pkgs = nixpkgs.legacyPackages.${system};
    in
      {
      devShells.${system}.default = pkgs.mkShell {
        buildInputs = with pkgs; [
          rustc
          cargo
        ];
      };

      packages.${system}.default = pkgs.rustPlatform.buildRustPackage{
        name = "nice-i3blocks";
        src = self;
        cargoHash = "sha256-FKP6GBUkpVvInv7Ie+vtmj2VHO4tznfsHuFY2Uc55Ag=";
      };
    };
}
