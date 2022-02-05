{ pkgs, ... }:
{
  shell = pkgs.mkShell {
    buildInputs = with pkgs; [
      libressl
      pkg-config
      (rust-bin.stable.latest.default.override {
        targets = [ "wasm32-unknown-unknown" ];
      })
      wasm-pack
    ];
  };
}
