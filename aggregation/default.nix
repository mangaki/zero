{ pkgs ? import <nixpkgs> {
  overlays = [ (import (builtins.fetchTarball "https://github.com/oxalica/rust-overlay/archive/master.tar.gz")) ];
}, lib ? pkgs.lib, ... }:
let
  filterLocalArtifacts = src: lib.cleanSourceWith {
    filter = name: type: let baseName = baseNameOf name; in ! (baseName == "target" && type == "directory");
    src = lib.cleanSource src;
  };
in
rec {
  shell = pkgs.mkShell {
    buildInputs = with pkgs; [
      pkgs.rust-bin.stable.latest.default
      # (pkgs.rust-bin.selectLatestNightlyWith (toolchain: toolchain.default))
      wasm-pack
      libressl
      pkg-config
      (python39.withPackages (ps: [ pythonPackage ]))
      python39Packages.pytest
      maturin
      llvm_13 # For llvm-symbolicator
    ];
  };

  rustPackage = pkgs.rustPlatform.buildRustPackage rec {
    pname = "rust-mangaki-zero-aggregation";
    version = "0.1.0";

    nativeBuildInputs = [
      (pkgs.rust-bin.selectLatestNightlyWith (toolchain: toolchain.minimal))
      pkgs.rustPlatform.maturinBuildHook
    ];

    src = filterLocalArtifacts ./.;
    sourceRoot = "source/rustlib";
    cargoSha256 = "sha256-W51ZHUdFYarzZThT6W9M5jszVjIQLjxpMdQCrflD0S4=";
  };

  pythonPackage = pkgs.python39.pkgs.buildPythonPackage rec {
    pname = "python-mangaki-zero-aggregation";
    version = "0.1.0";

    format = "wheel";
    sourceRoot = "source/pylib";

    nativeBuildInputs = [
      pkgs.rustPlatform.cargoSetupHook
      pkgs.rustPlatform.maturinBuildHook
      pkgs.python39
    ];

    cargoDeps = pkgs.rustPlatform.fetchCargoTarball {
      inherit src sourceRoot;
      hash = "sha256-YGpVeeRkRzR4anriV6f0kfO2acryWwBaL2m7Qshh82A=";
    };

    dontUseWheelUnpack = true;
    src = filterLocalArtifacts ./.;
  };
}
