{ pkgs ? import <nixpkgs> {}, python ? pkgs.python3, useMKL ? true }:

rec {
  pythonDependencies = (python.withPackages
  (ps: [
    ps.numpy
    ps.scipy
    ps.pandas
  ]));

  shell = pkgs.mkShell {
    buildInputs = with pkgs; [
      pythonDependencies
      poetry
    ];
  };
}
