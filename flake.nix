{
  description = "make-vestnik — LaTeX document project manager";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
      in {
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "make-vestnik";
          version = "1.0.0";
          src = ./.;

          cargoLock.lockFile = ./Cargo.lock;

          nativeBuildInputs = [
            pkgs.pkg-config
          ];

          # runtime dependencies
          propagatedBuildInputs = with pkgs; [
            pandoc
            texlive.combined.scheme-full
            imagemagickBig
            ghostscript
          ];

          # install templates in $out/share/make-vestnik/templates
          postInstall = ''
            mkdir -p $out/share/make-vestnik/templates
            cp -r $src/template/* $out/share/make-vestnik/templates/
          '';

          meta = with pkgs.lib; {
            description =
              "LaTeX document project manager — converts DOCX, updates projects, and compiles XeLaTeX";
            license = licenses.mit;
          };
        };

        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            rustc
            cargo
            cargo-watch
            pandoc
            texlive.combined.scheme-full
            imagemagickBig
            ghostscript
            perl
            python3
          ];

          # devShell: templates exist in source tree
          # no environment variables needed
        };

        apps.default = {
          type = "app";
          program = ''
            ${self.packages.${system}.default}/bin/make-vestnik
          '';
        };
      });
}

