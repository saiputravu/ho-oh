{
  description = "Ho-Oh";

  inputs = {
    nixpkgs.url =  "github:nixos/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachSystem [
      "x86_64-linux"
      "aarch64-darwin"
    ] (system:
        let
          pkgs = nixpkgs.legacyPackages.${system};

          sharedNativeBuildInputs = with pkgs; [];

          sharedBuildInputs = with pkgs; [
            cargo
            rustc
            rust-analyzer
          ];
        in
        {
          # Development shell: facilitates manual building and development of TuringDB
          devShells.default = pkgs.mkShell {
            nativeBuildInputs = sharedNativeBuildInputs;
            buildInputs = sharedBuildInputs;
          };
        }
      );
}
