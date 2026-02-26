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

        darwinBuildInputs = with pkgs; lib.optionals stdenv.hostPlatform.isDarwin [
          apple-sdk_15
          libiconv
        ];
      in
      {
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "ho-oh";
          version = "0.1.0";
          src = ./.;
          useFetchCargoVendor = true;
          cargoHash = "sha256-qjc5biy86qG/de+9QruscjPMbmIqm3mPI25kNJNCAsc=";

          buildInputs = darwinBuildInputs;
        };

        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs;[
            cargo
            rustc
            rust-analyzer
          ] ++ darwinBuildInputs;
        };
      }
    );
}
