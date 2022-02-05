{ pkgs, ... }:
{
  shell = pkgs.mkShell {
    buildInputs = with pkgs; [
      rust-bin.stable.latest.default
    ];
  };
}
