{
  writeScriptBin,
  alejandra,
  rustfmt,
}:
writeScriptBin "fmt" ''
  echo "Formatting Nix code..."
  ${alejandra}/bin/alejandra -q .
  echo "Formatting Rust code..."
  PATH=$PATH:${rustfmt}/bin
  ${rustfmt}/bin/cargo-fmt
''
