{
  description = "make-vestnik — LaTeX document project manager";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let pkgs = import nixpkgs { inherit system; };
      in {
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "make-vestnik";
          version = "1.0.0";
          src = ./.;
          cargoLock = ./Cargo.lock;
          buildInputs = with pkgs; [
            pandoc
            texlive.combined.scheme-full
            imagemagick
            ghostscript
          ];
          nativeBuildInputs = with pkgs; [
            pkg-config
          ];
          meta = with pkgs.lib; {
            description = "LaTeX document project manager — converts DOCX, updates projects, and compiles XeLaTeX";
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
            imagemagick
            ghostscript
            perl
            python3
          ];
        };
      });
}

