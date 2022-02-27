let
  overlays = [
    (import (builtins.fetchTarball "https://github.com/oxalica/rust-overlay/archive/master.tar.gz"))

  ];
  # TODO: until we can use manylinux_2_33...
  oldNixpkgs = import (builtins.fetchTarball {
        url = "https://github.com/NixOS/nixpkgs/archive/2c162d49cd5b979eb66ff1653aecaeaa01690fcc.tar.gz";
  });
  newNixpkgs = import <nixpkgs> { inherit overlays; };
in
{ pkgs ? oldNixpkgs { inherit overlays; }, lib ? pkgs.lib, pythonPackageName ? "python39", ... }:
let
  filterLocalArtifacts = src: lib.cleanSourceWith {
    filter = name: type: let baseName = baseNameOf name; in ! (baseName == "target" && type == "directory");
    src = lib.cleanSource src;
  };
  python = pkgs.${pythonPackageName};
in
rec {
  shell = pkgs.mkShell {
    buildInputs = with pkgs; [
      pkgs.rust-bin.stable.latest.default
      # (pkgs.rust-bin.selectLatestNightlyWith (toolchain: toolchain.default))
      # wasm-pack
      libressl
      pkg-config
      (python39.withPackages (ps: [ pythonPackage ps.pytest ps.pytestcov ]))
      newNixpkgs.auditwheel
      newNixpkgs.maturin
      python
      mypy
      # DEBUG: newNixpkgs.llvm_13 # For llvm-symbolicator
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

  pythonPackage = python.pkgs.buildPythonPackage rec {
    pname = "python-mangaki-zero-aggregation";
    version = "0.1.0";

    format = "wheel";
    sourceRoot = "source/pylib";

    nativeBuildInputs = [
      pkgs.rustPlatform.cargoSetupHook
      pkgs.rustPlatform.maturinBuildHook
      python
    ];

    cargoDeps = pkgs.rustPlatform.fetchCargoTarball {
      inherit src sourceRoot;
      hash = "sha256-1OmGICRTpsZlUiGYWk+lGhlYnJswOJCKZos5nF97CCo=";
    };

    dontUseWheelUnpack = true;
    src = filterLocalArtifacts ./.;
  };
}
