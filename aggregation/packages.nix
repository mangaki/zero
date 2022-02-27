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
      rustPlatform.maturinBuildHook
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
      rustPlatform.cargoSetupHook
      rustPlatform.maturinBuildHook
      python
    ];

    cargoDeps = rustPlatform.fetchCargoTarball {
      inherit src sourceRoot;
      hash = "sha256-1OmGICRTpsZlUiGYWk+lGhlYnJswOJCKZos5nF97CCo=";
    };

    dontUseWheelUnpack = true;
    src = filterLocalArtifacts ./.;
  };
}
