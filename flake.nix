{
  description = "Ho-Oh";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachSystem [
      "x86_64-linux"
      "aarch64-darwin"
    ] (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};

        darwinBuildInputs = pkgs.lib.optionals pkgs.stdenv.hostPlatform.isDarwin [
          pkgs.apple-sdk_15
          pkgs.libiconv
        ];
      in
      {
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "ho-oh";
          version = "0.1.0";
          src = ./.;
          useFetchCargoVendor = true;
          cargoHash = "";

          buildInputs = darwinBuildInputs;
        };

        devShells.default = pkgs.mkShell {
          buildInputs = [
            pkgs.cargo
            pkgs.rustc
            pkgs.rust-analyzer
          ] ++ darwinBuildInputs;
        };
      }
    );
}
