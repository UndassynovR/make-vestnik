{
  description = "make-vestnik — document project manager for LaTeX publishing";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
  }:
    flake-utils.lib.eachDefaultSystem (system: let
      pkgs = import nixpkgs {inherit system;};
    in {
      packages.default = pkgs.rustPlatform.buildRustPackage {
        pname = "make-vestnik";
        version = "0.1.0";
        src = ./.;

        cargoLock.lockFile = ./Cargo.lock;

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
