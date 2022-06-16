with import <nixpkgs> {}; 
let 
  c = callPackage ./Cargo.nix {}; 
in 
  c.rootCrate.build

