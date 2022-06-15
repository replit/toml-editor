{ stdenv, openssl, lame, git, runCommand
, copyPathToStore, rev, pkg-config, lib, defaultCrateOverrides, buildRustCrate
, buildPackages, fetchurl }@pkgs:
let
  generatedBuild = import ./Cargo.nix {
    inherit pkgs;
    buildRustCrateForPkgs = pkgs:
      pkgs.buildRustCrate.override {
        defaultCrateOverrides = pkgs.defaultCrateOverrides // {
          "toml-editor" = attrs: {
            buildInputs = [ pkg-config ];
          };
        };
      };
  };

  crate2nix = generatedBuild.rootCrate.build;

in stdenv.mkDerivation {
  pname = "toml-editor";
  version = rev;

  src = crate2nix;

  installPhase = ''
    cp -r ${crate2nix} $out
  '';
}

