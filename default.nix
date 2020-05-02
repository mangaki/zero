let ourNixpkgs =
  builtins.fetchGit {
  # Descriptive name to make the store path easier to identify
  name = "nixos-unstable-2020-05-02";
  url = "https://github.com/nixos/nixpkgs-channels/";
  # Commit hash for nixos-unstable as of 2020-05-02
  # `git ls-remote https://github.com/nixos/nixpkgs-channels nixos-unstable`
  ref = "refs/heads/nixos-unstable";
  rev = "fce7562cf46727fdaf801b232116bc9ce0512049";
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
