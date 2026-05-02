{
  description = "nix flake for installing and linking with libnetfilter_queue and openssl.";
  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };
  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
      rust-overlay,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ rust-overlay.overlays.default ];
        };
      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            rust-bin.stable.latest.default
            uv
            libnetfilter_queue
            openssl
            pkg-config
            quickemu
            spice
          ];
          shellHook = ''
            export PKG_CONFIG_PATH="${pkgs.libnetfilter_queue}/lib/pkgconfig:${pkgs.openssl}/lib/pkgconfig"
            export LD_LIBRARY_PATH="${pkgs.libnetfilter_queue}/lib:${pkgs.openssl}/lib:$LD_LIBRARY_PATH"
          '';
        };
      }
    );
}
