let
  pkgs = import <nixpkgs> {
    overlays = [ (import (builtins.fetchTarball "https://github.com/oxalica/rust-overlay/archive/master.tar.gz")) ];
  };
in (import ./default.nix { inherit pkgs; }).shell
