let ourNixpkgs =
  builtins.fetchGit {
  # Descriptive name to make the store path easier to identify
  name = "nixpkgs-unstable-2020-05-02";
  url = "https://github.com/nixos/nixpkgs-channels/";
  # Commit hash for nixos-unstable as of 2020-05-02
  # `git ls-remote https://github.com/nixos/nixpkgs-channels nixpkgs-unstable`
  ref = "refs/heads/nixpkgs-unstable";
  rev = "10100a97c8964e82b30f180fda41ade8e6f69e41";
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
