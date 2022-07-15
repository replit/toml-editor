{ rustPlatform, rev }:

rustPlatform.buildRustPackage {
    pname = "toml-editor";
    version = rev;

    cargoLock = {
      lockFile = ./Cargo.lock;
    };

    src = ./.;
}

