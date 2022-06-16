{ stdenv, fetchurl, rev, lib, defaultCrateOverrides, buildRustCrate, buildPackages }@pkgs:
let
  generatedBuild = import ./Cargo.nix {
    inherit pkgs;
  };

  crate2nix = generatedBuild.workspaceMembers.toml-editor.build;

in stdenv.mkDerivation {
  pname = "toml-editor";
  version = rev;

  src = crate2nix;

  installPhase = ''
    cp -r ${crate2nix} $out
  '';
}

