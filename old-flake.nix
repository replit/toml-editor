{
  description = "A toml editor that preserves existing formatting and comments.";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }: 
    flake-utils.lib.eachDefaultSystem (system:
      let pkgs = nixpkgs.legacyPackages.${system};
        "toml-editor" = pkgs.callPackage ./toml-editor.nix {
          rev = if self ? rev then "0.0.0-${builtins.substring 0 7 self.rev}" else "0.0.0-dirty";
        };
      in
      {
        defaultPackage = "toml-editor";
        packages = {
          inherit toml-editor;
        };
      }
    );
}
