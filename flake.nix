{
  description = "A toml editor that preserves formatting.";

  inputs.nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";
  inputs.nix-github-actions.url = "github:nix-community/nix-github-actions";
  inputs.nix-github-actions.inputs.nixpkgs.follows = "nixpkgs";

  outputs = {
    self,
    nixpkgs,
    nix-github-actions,
  }: let
    systems = [
      "aarch64-darwin"
      "aarch64-linux"
      "x86_64-darwin"
      "x86_64-linux"
    ];
    eachSystem = nixpkgs.lib.genAttrs systems;
    rev =
      if self ? rev
      then "0.0.0-${builtins.substring 0 7 self.rev}"
      else "0.0.0-dirty";
  in {
    packages = eachSystem (system: let
      pkgs = nixpkgs.legacyPackages.${system};
    in rec {
      default = toml-editor;
      toml-editor = pkgs.callPackage ./toml-editor.nix {
        inherit rev;
      };
      devShell = pkgs.callPackage ./nix/devshell {};
      fmt = pkgs.callPackage ./nix/fmt {};
    });
    formatter = eachSystem (system: self.packages.${system}.fmt);
    githubActions = nix-github-actions.lib.mkGithubMatrix {
      checks = nixpkgs.lib.getAttrs [ "x86_64-linux" ] self.packages;
    };
  };
}
