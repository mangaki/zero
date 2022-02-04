{ pkgs, ... }:
{
  shell = pkgs.mkShell {
    buildInputs = with pkgs; [
      rust-bin.stable.latest.default
      python39
      python39Packages.pip
      python39Packages.virtualenv
      maturin
    ];
    shellHook = ''
      source ./venv/bin/activate
      '';
  };
}
