let ourNixpkgs =
  builtins.fetchGit {
  # Descriptive name to make the store path easier to identify
  name = "nixos-21.05";
  url = "https://github.com/nixos/nixpkgs";
  # Commit hash for nixos-unstable as of 2021-10-17
  # `git ls-remote https://github.com/nixos/nixpkgs-channels nixos-unstable`
  ref = "refs/heads/nixos-21.05";
  rev = "83667ff60a88e22b76ef4b0bdf5334670b39c2b6";
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
, pythonPackageName ? "python3"
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
