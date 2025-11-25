{
  description = "make-vestnik — LaTeX document project manager";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs { inherit system; };
      in
      {
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "make-vestnik";
          version = "1.0.0";
          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;

          nativeBuildInputs = [
            pkgs.pkg-config
            pkgs.makeWrapper
          ];

          # runtime deps
          propagatedBuildInputs = with pkgs; [
            pandoc
            texlive.combined.scheme-full
            imagemagickBig
            ghostscript
          ];

          postInstall = ''
            mkdir -p $out/share/make-vestnik/templates
            cp -r $src/template/* $out/share/make-vestnik/templates/

            # wrap the binary so 'gs' is on PATH for ImageMagick
            wrapProgram $out/bin/make-vestnik \
              --prefix PATH : ${pkgs.ghostscript}/bin
          '';

          meta = with pkgs.lib; {
            description = "LaTeX document manager";
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
      }
    );
}
