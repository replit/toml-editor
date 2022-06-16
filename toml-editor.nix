with import <nixpkgs> {}; let c = callPackage ./Cargo.nix {}; in c.workspaceMembers."toml-editor".build

# { stdenv, lame, git, runCommand, copyPathToStore, rev, lib, defaultCrateOverrides, buildRustCrate

# , buildPackages }@pkgs:
# let
#   generatedBuild = import ./Cargo.nix {
#     inherit pkgs;
#   };

#   crate2nix = generatedBuild.rootCrate.build;

# in stdenv.mkDerivation {
#   pname = "toml-editor";
#   version = rev;

#   src = crate2nix;

#   installPhase = ''
#     cp -r ${crate2nix} $out
#   '';
# }

