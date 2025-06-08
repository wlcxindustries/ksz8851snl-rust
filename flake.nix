{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };
  outputs = { nixpkgs, rust-overlay, ... }: let
    forAllSystems = function:
      nixpkgs.lib.genAttrs [
        "x86_64-linux"
        "aarch64-linux"
        "aarch64-darwin"
      ] (system: function (import nixpkgs {
        inherit system;
        overlays = [
          (import rust-overlay)
        ];
      }));
  in {
    devShells = forAllSystems (pkgs: {
      default = pkgs.mkShell {
        packages = with pkgs; [
          (rust-bin.nightly.latest.default.override {
            extensions = [ "rust-src" ];
            targets = [ "riscv32imc-unknown-none-elf" ];
          })
          rust-analyzer
        ];
      };
    });
  };
}
