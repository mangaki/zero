{ pkgs ? import <nixpkgs> {}, python ? pkgs.python3, useMKL ? true }:

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
