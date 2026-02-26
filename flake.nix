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

        # Wrapper around system xcrun for Metal shader compilation.
        # Nix's xcbuild xcrun can't find the proprietary metal compiler,
        # and apple-sdk's DEVELOPER_DIR causes /usr/bin/xcrun to search
        # the wrong SDK. This wrapper handles both issues.
        metalc = pkgs.writeShellScriptBin "metalc" ''
          unset DEVELOPER_DIR
          exec /usr/bin/xcrun "$@"
        '';

        metalKernels = pkgs.stdenv.mkDerivation {
          pname = "ho-oh-kernels";
          version = "0.1.0";
          src = ./src/kernels;

          buildInputs = darwinBuildInputs;
          nativeBuildInputs = [ metalc ];

          # Metal compiler lives under a cryptexd mount that the sandbox blocks by default
          sandboxProfile = ''
            (allow file-read* (subpath "/var/run/com.apple.security.cryptexd"))
          '';

          buildPhase = ''
            for f in *.metal; do
              metalc metal -c "$f" -o "''${f%.metal}.air"
            done
            metalc metallib *.air -o kernels.metallib
          '';

          installPhase = ''
            mkdir -p $out
            cp kernels.metallib $out/
          '';
        };
      in
      {
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "ho-oh";
          version = "0.1.0";
          src = ./.;
          cargoHash = "sha256-qjc5biy86qG/de+9QruscjPMbmIqm3mPI25kNJNCAsc=";

          postPatch = ''
            cp ${metalKernels}/kernels.metallib src/kernels/kernels.metallib
          '';

          buildInputs = darwinBuildInputs;
        };

        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs;[
            cargo
            rustc
            rust-analyzer
            metalc
          ] ++ darwinBuildInputs;

          shellHook = ''
            if [ ! -f src/kernels/kernels.metallib ] || \
               [ src/kernels/example.metal -nt src/kernels/kernels.metallib ]; then
              echo "Compiling Metal kernels..."
              (cd src/kernels && for f in *.metal; do
                metalc metal -c "$f" -o "''${f%.metal}.air"
              done && metalc metallib *.air -o kernels.metallib && rm -f *.air)
              echo "Metal kernels compiled."
            fi
          '';
        };
      }
    );
}
