let ourNixpkgs =
  builtins.fetchGit {
  # Descriptive name to make the store path easier to identify
  name = "nixos-22.05";
  url = "https://github.com/nixos/nixpkgs";
  # `git ls-remote https://github.com/nixos/nixpkgs nixos-22.05`
  ref = "refs/heads/nixos-22.05";
  rev = "f21492b413295ab60f538d5e1812ab908e3e3292";
};
in
{ blasProvider ? "openblasCompat"
, pkgs ? import ourNixpkgs {
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
, pythonPackageName ? "python310"
, python ? pkgs.${pythonPackageName}}:

rec {
  pythonDependencies = (python.withPackages
  (ps: [
    ps.numpy
    ps.scipy
    ps.pandas
    ps.scikitlearn
    ps.pytest
    ps.pytest_xdist
    ps.pytestcov
  ]));

  shell = pkgs.mkShell {
    buildInputs = with pkgs; [
      pythonDependencies
      python.pkgs.sphinx
      python.pkgs.sphinxcontrib-jsmath
      poetry
    ];
  };
}
