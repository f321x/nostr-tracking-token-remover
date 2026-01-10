{ pkgs ? import <nixpkgs> {} }:

let
  inherit (pkgs.lib) makeLibraryPath;
  inherit (pkgs.lib) fileContents;
  inherit (pkgs) stdenv;
in
  pkgs.mkShell {
    nativeBuildInputs = with pkgs.buildPackages; [ 
        pkg-config 
        zlib 
        openssl 
        cargo
        clippy
        rustc 
        rustfmt
        gcc
     ];

    NIX_LD_LIBRARY_PATH = makeLibraryPath [
      stdenv.cc.cc
      pkgs.openssl
      pkgs.zlib
      pkgs.pkg-config
      pkgs.gcc
    ];

    RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
    NIX_LD = fileContents "${stdenv.cc}/nix-support/dynamic-linker";
}

# For VSCode and Rust Analyzer use the Extension "Nix Environment Selector"
