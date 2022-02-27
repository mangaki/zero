let
  overlays = [
    (import (builtins.fetchTarball "https://github.com/oxalica/rust-overlay/archive/master.tar.gz"))

  ];
  # TODO: until we can use manylinux_2_33...
  oldNixpkgs = import (builtins.fetchTarball {
        url = "https://github.com/NixOS/nixpkgs/archive/2c162d49cd5b979eb66ff1653aecaeaa01690fcc.tar.gz";
  }) { inherit overlays; };
  newNixpkgs = import <nixpkgs> { inherit overlays; };
in
{ pythonPackageName ? "python39", ... }:
rec {
  selectPython = pkgs: pkgs.${pythonPackageName};

  shell = newNixpkgs.mkShell {
    buildInputs = with newNixpkgs; [
      rust-bin.stable.latest.default
      libressl
      pkg-config
      ((selectPython newNixpkgs).withPackages (ps: [ packages.pythonPackage ps.pytest ps.pytestcov ]))
      auditwheel
      maturin
      mypy
    ];
  };

  publishShell = oldNixpkgs.mkShell {
    buildInputs = with oldNixpkgs; [
      rust-bin.stable.latest.default
      libressl
      pkg-config
      newNixpkgs.auditwheel
      newNixpkgs.maturin
      (selectPython oldNixpkgs)
    ];
  };

  packages = newNixpkgs.callPackage ./packages.nix {
    rustToolchain = newNixpkgs.rust-bin.stable.latest.default;
    python = selectPython newNixpkgs;
  };
}
