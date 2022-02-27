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
    cargoSha256 = "sha256-IN0Iz2Dsi3W76N4sSOn52hvfGgNGqUweQ7vFtAHrZdE=";
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
      hash = "sha256-vy3kN+9Kf3GALKQEtdcXdxHe9M+XcWndcUt/iVIHAr0=";
    };

    dontUseWheelUnpack = true;
    src = filterLocalArtifacts ./.;
  };
}
