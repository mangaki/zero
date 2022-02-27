{ lib, rustPlatform, rustToolchain, python }:
let
  filterLocalArtifacts = src: lib.cleanSourceWith {
    filter = name: type: let baseName = baseNameOf name; in ! (baseName == "target" && type == "directory");
    src = lib.cleanSource src;
  };
in
{
  rustPackage = rustPlatform.buildRustPackage rec {
    pname = "rust-mangaki-zero-aggregation";
    version = "0.1.0";

    nativeBuildInputs = [
      rustToolchain
    ];

    src = filterLocalArtifacts ./.;
    sourceRoot = "source/rustlib";
    cargoSha256 = "sha256-RMBw3qbrvrUREgisioUnwAWpcF2pEGY6rGBUjbzoDcs=";
  };

  pythonPackage = python.pkgs.buildPythonPackage rec {
    pname = "python-mangaki-zero-aggregation";
    version = "0.1.0";

    format = "wheel";
    sourceRoot = "source/pylib";

    nativeBuildInputs = [
      rustPlatform.cargoSetupHook
      rustPlatform.maturinBuildHook
      python
    ];

    cargoDeps = rustPlatform.fetchCargoTarball {
      inherit src sourceRoot;
      hash = "sha256-co4YhZEsyOcLowcR0yYrAKdIyO7fRzZ0MvY2V3lDhzo=";
    };

    dontUseWheelUnpack = true;
    src = filterLocalArtifacts ./.;
  };
}
