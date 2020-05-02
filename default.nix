{ blasProvider ? "openblasCompat"
, pkgs ? import <nixpkgs> {
  config.allowUnfree = blasProvider == "mkl";
  overlays = [
    (self: super: {
        lapack = super.lapack.override {
          lapackProvider = super.${blasProvider};
        };
        blas = super.blas.override {
          blasProvider = super.${blasProvider};
        };
      })]; }
, pythonPackageName ? "python3"
, python ? pkgs.${pythonPackageName}}:

rec {
  pythonDependencies = (python.withPackages
  (ps: [
    ps.numpy
    ps.scipy
    ps.pandas
    ps.pytest
    ps.pytest_xdist
    ps.pytestcov
  ]));

  shell = pkgs.mkShell {
    buildInputs = with pkgs; [
      pythonDependencies
      poetry
    ];
  };
}
