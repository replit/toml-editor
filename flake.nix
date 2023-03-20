{
  description = "A toml editor that preserves formatting.";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";

    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, fenix }:
    flake-utils.lib.eachDefaultSystem
      (system:
        let
          pkgs = nixpkgs.legacyPackages.${system};
          toml-editor = pkgs.callPackage ./toml-editor.nix {
            rev = if self ? rev then "0.0.0-${builtins.substring 0 7 self.rev}" else "0.0.0-dirty";
          };
        in
        {
          packages = {
            inherit toml-editor;

            default = toml-editor;
          };
        }
      );
}
